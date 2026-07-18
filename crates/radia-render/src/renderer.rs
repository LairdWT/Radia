use std::num::NonZeroU64;
use std::sync::{Arc, Mutex};

use radia_math::{ErrorScale, UnitDualQuat, UnitQuat, Vec3};

use crate::RenderError;

const HISTORY_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const UNIFORM_SIZE: u64 = 96;
const UNIFORM_SIZE_BYTES: usize = 96;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RadiaMode {
    Off,
    Radia,
    GiOnly,
    SdfDistance,
    PrimitiveId,
    Normal,
    StepCount,
    HitState,
    Triangle,
}

impl RadiaMode {
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        match self {
            Self::Off => 0,
            Self::Radia => 1,
            Self::GiOnly => 2,
            Self::SdfDistance => 3,
            Self::PrimitiveId => 4,
            Self::Normal => 5,
            Self::StepCount => 6,
            Self::HitState => 7,
            Self::Triangle => 8,
        }
    }

    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Off => Self::Radia,
            Self::Radia => Self::GiOnly,
            Self::GiOnly => Self::SdfDistance,
            Self::SdfDistance => Self::PrimitiveId,
            Self::PrimitiveId => Self::Normal,
            Self::Normal => Self::StepCount,
            Self::StepCount => Self::HitState,
            Self::HitState => Self::Triangle,
            Self::Triangle => Self::Off,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderSettings {
    pub mode: RadiaMode,
    pub camera_to_world: UnitDualQuat,
    pub emitter_position: Vec3,
    pub emitter_radius: f32,
    pub emitter_color: Vec3,
    pub emitter_intensity: f32,
    pub maximum_distance: f32,
    pub hit_guard: f32,
    pub maximum_steps: u32,
}

/// Returns the fixed synthetic courtyard used by windowed and headless paths.
///
/// # Errors
///
/// Returns an error only if the frozen finite camera pose cannot be built.
pub fn default_render_settings(mode: RadiaMode) -> Result<RenderSettings, RenderError> {
    let error_scale = ErrorScale::new(100.0, 64)?;
    let camera_to_world = UnitDualQuat::from_rotation_translation(
        UnitQuat::identity(),
        Vec3::new(0.0, 0.6, 4.5),
        error_scale,
    )?;
    Ok(RenderSettings {
        mode,
        camera_to_world,
        emitter_position: Vec3::new(14.0, 2.2, -4.0),
        emitter_radius: 1.6,
        emitter_color: Vec3::new(1.0, 0.48, 0.16),
        emitter_intensity: 64.0,
        maximum_distance: 40.0,
        hit_guard: 0.000_5,
        maximum_steps: 128,
    })
}

#[derive(Debug)]
struct HistoryTargets {
    views: [wgpu::TextureView; 2],
}

impl HistoryTargets {
    fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let textures = [
            create_history_texture(device, width, height, "radia-history-a"),
            create_history_texture(device, width, height, "radia-history-b"),
        ];
        let views = [
            textures[0].create_view(&wgpu::TextureViewDescriptor {
                label: Some("radia-history-view-a"),
                ..Default::default()
            }),
            textures[1].create_view(&wgpu::TextureViewDescriptor {
                label: Some("radia-history-view-b"),
                ..Default::default()
            }),
        ];
        Self { views }
    }
}

#[derive(Debug, Default)]
struct GpuIssues {
    uncaptured: Vec<String>,
    device_lost: Option<String>,
}

pub struct RadiaRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    adapter: wgpu::Adapter,
    adapter_info: wgpu::AdapterInfo,
    output_format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    uniform_buffer: wgpu::Buffer,
    accumulate_bind_group_layout: wgpu::BindGroupLayout,
    present_bind_group_layout: wgpu::BindGroupLayout,
    accumulate_pipeline: wgpu::RenderPipeline,
    present_pipeline: wgpu::RenderPipeline,
    history: HistoryTargets,
    latest_history_index: usize,
    frame_index: u32,
    state_key: Option<[u32; 23]>,
    gpu_issues: Arc<Mutex<GpuIssues>>,
}

impl RadiaRenderer {
    /// Requests a Vulkan adapter/device compatible with an optional surface.
    ///
    /// # Errors
    ///
    /// Returns typed adapter, device, format, or validation failures.
    pub async fn new(
        instance: &wgpu::Instance,
        compatible_surface: Option<&wgpu::Surface<'_>>,
        requested_output_format: Option<wgpu::TextureFormat>,
        width: u32,
        height: u32,
    ) -> Result<Self, RenderError> {
        validate_extent(width, height)?;
        let adapter = select_vulkan_adapter(instance, compatible_surface).await?;
        let adapter_info = adapter.get_info();
        if adapter_info.backend != wgpu::Backend::Vulkan {
            return Err(RenderError::UnsupportedAdapter(format!(
                "expected Vulkan, got {:?} ({})",
                adapter_info.backend, adapter_info.name
            )));
        }

        let output_format =
            select_output_format(&adapter, compatible_surface, requested_output_format)?;

        let history_features = adapter.get_texture_format_features(HISTORY_FORMAT);
        let required_history_usages =
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        if !history_features
            .allowed_usages
            .contains(required_history_usages)
        {
            return Err(RenderError::UnsupportedFormat(format!(
                "{HISTORY_FORMAT:?} lacks {required_history_usages:?} on {}",
                adapter_info.name
            )));
        }
        let output_features = adapter.get_texture_format_features(output_format);
        if !output_features
            .allowed_usages
            .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
        {
            return Err(RenderError::UnsupportedFormat(format!(
                "{output_format:?} is not renderable on {}",
                adapter_info.name
            )));
        }

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("radia-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|error| RenderError::DeviceRequest(error.to_string()))?;

        let gpu_issues = Arc::new(Mutex::new(GpuIssues::default()));
        install_error_handlers(&device, Arc::clone(&gpu_issues));
        device.push_error_scope(wgpu::ErrorFilter::Validation);

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("radia-frame-uniform"),
            size: UNIFORM_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let accumulate_bind_group_layout = create_accumulate_bind_group_layout(&device);
        let present_bind_group_layout = create_present_bind_group_layout(&device);
        let accumulate_pipeline =
            create_accumulate_pipeline(&device, &accumulate_bind_group_layout);
        let present_pipeline =
            create_present_pipeline(&device, &present_bind_group_layout, output_format);
        let history = HistoryTargets::new(&device, width, height);

        if let Some(error) = device.pop_error_scope().await {
            return Err(RenderError::GpuValidation(error.to_string()));
        }

        Ok(Self {
            device,
            queue,
            adapter,
            adapter_info,
            output_format,
            width,
            height,
            uniform_buffer,
            accumulate_bind_group_layout,
            present_bind_group_layout,
            accumulate_pipeline,
            present_pipeline,
            history,
            latest_history_index: 0,
            frame_index: 0,
            state_key: None,
            gpu_issues,
        })
    }

    #[must_use]
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub(crate) fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    #[must_use]
    pub fn adapter_info(&self) -> &wgpu::AdapterInfo {
        &self.adapter_info
    }

    #[must_use]
    pub fn output_format(&self) -> wgpu::TextureFormat {
        self.output_format
    }

    /// Builds a supported FIFO surface configuration for this adapter.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid extent or capability drift.
    pub fn surface_configuration(
        &self,
        surface: &wgpu::Surface<'_>,
        width: u32,
        height: u32,
    ) -> Result<wgpu::SurfaceConfiguration, RenderError> {
        validate_extent(width, height)?;
        let capabilities = surface.get_capabilities(&self.adapter);
        if !capabilities.formats.contains(&self.output_format) {
            return Err(RenderError::UnsupportedFormat(format!(
                "surface no longer advertises {:?}",
                self.output_format
            )));
        }
        let alpha_mode = capabilities.alpha_modes.first().copied().ok_or_else(|| {
            RenderError::UnsupportedFormat("surface has no alpha mode".to_owned())
        })?;
        Ok(wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.output_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: Vec::new(),
        })
    }

    #[must_use]
    pub fn extent(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Recreates extent-dependent history and resets accumulation.
    ///
    /// # Errors
    ///
    /// Returns an error for zero extent.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), RenderError> {
        validate_extent(width, height)?;
        if self.width == width && self.height == height {
            return Ok(());
        }
        self.width = width;
        self.height = height;
        self.history = HistoryTargets::new(&self.device, width, height);
        self.reset_history();
        Ok(())
    }

    pub fn reset_history(&mut self) {
        self.frame_index = 0;
        self.latest_history_index = 0;
        self.state_key = None;
    }

    /// Renders one deterministic sample and presents the running mean.
    ///
    /// # Errors
    ///
    /// Returns a config, device-loss, validation, or poll failure.
    pub fn render_to_view(
        &mut self,
        output_view: &wgpu::TextureView,
        settings: RenderSettings,
    ) -> Result<(), RenderError> {
        validate_settings(settings)?;
        let state_key = settings_state_key(settings, self.width, self.height);
        if self.state_key != Some(state_key) {
            self.frame_index = 0;
            self.state_key = Some(state_key);
        }

        let previous_index = self.latest_history_index;
        let current_index = 1 - previous_index;
        let uniform_bytes = encode_uniforms(settings, self.width, self.height, self.frame_index);
        self.queue
            .write_buffer(&self.uniform_buffer, 0, &uniform_bytes);

        let accumulate_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("radia-accumulate-bind-group"),
            layout: &self.accumulate_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &self.history.views[previous_index],
                    ),
                },
            ],
        });
        let present_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("radia-present-bind-group"),
            layout: &self.present_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&self.history.views[current_index]),
            }],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("radia-frame-encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("radia-accumulate-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.history.views[current_index],
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.accumulate_pipeline);
            pass.set_bind_group(0, &accumulate_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("radia-present-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.present_pipeline);
            pass.set_bind_group(0, &present_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        self.latest_history_index = current_index;
        self.frame_index = self.frame_index.saturating_add(1);
        self.device
            .poll(wgpu::PollType::Poll)
            .map_err(|error| RenderError::DevicePoll(error.to_string()))?;
        self.check_gpu_issues()
    }

    fn check_gpu_issues(&self) -> Result<(), RenderError> {
        let issues = self
            .gpu_issues
            .lock()
            .map_err(|_| RenderError::GpuValidation("GPU issue lock poisoned".to_owned()))?;
        if let Some(message) = &issues.device_lost {
            return Err(RenderError::DeviceLost(message.clone()));
        }
        if !issues.uncaptured.is_empty() {
            return Err(RenderError::GpuValidation(issues.uncaptured.join(" | ")));
        }
        Ok(())
    }
}

async fn select_vulkan_adapter(
    instance: &wgpu::Instance,
    compatible_surface: Option<&wgpu::Surface<'_>>,
) -> Result<wgpu::Adapter, RenderError> {
    if let Some(requested_name) = std::env::var_os("RADIA_ADAPTER_NAME") {
        let requested_name = requested_name.to_string_lossy().trim().to_ascii_lowercase();
        if requested_name.is_empty() {
            return Err(RenderError::InvalidConfig(
                "RADIA_ADAPTER_NAME must not be empty when set".to_owned(),
            ));
        }

        let mut available_names = Vec::new();
        let mut matches = Vec::new();
        for adapter in instance.enumerate_adapters(wgpu::Backends::VULKAN) {
            if compatible_surface.is_some_and(|surface| !adapter.is_surface_supported(surface)) {
                continue;
            }
            let name = adapter.get_info().name;
            available_names.push(name.clone());
            if name.to_ascii_lowercase().contains(&requested_name) {
                matches.push(adapter);
            }
        }
        return match matches.len() {
            1 => Ok(matches.remove(0)),
            0 => Err(RenderError::UnsupportedAdapter(format!(
                "RADIA_ADAPTER_NAME={requested_name:?} matched no compatible Vulkan adapter; available: {}",
                available_names.join(", ")
            ))),
            count => Err(RenderError::UnsupportedAdapter(format!(
                "RADIA_ADAPTER_NAME={requested_name:?} matched {count} compatible Vulkan adapters; use a more specific name"
            ))),
        };
    }

    instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface,
        })
        .await
        .map_err(|error| RenderError::AdapterRequest(error.to_string()))
}

fn create_history_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    label: &'static str,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: HISTORY_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

fn select_output_format(
    adapter: &wgpu::Adapter,
    compatible_surface: Option<&wgpu::Surface<'_>>,
    requested: Option<wgpu::TextureFormat>,
) -> Result<wgpu::TextureFormat, RenderError> {
    let Some(surface) = compatible_surface else {
        return requested.ok_or_else(|| {
            RenderError::UnsupportedFormat("offscreen output format was not declared".to_owned())
        });
    };
    let capabilities = surface.get_capabilities(adapter);
    if let Some(format) = requested {
        if capabilities.formats.contains(&format) {
            return Ok(format);
        }
        return Err(RenderError::UnsupportedFormat(format!(
            "surface does not advertise requested {format:?}"
        )));
    }
    capabilities
        .formats
        .iter()
        .copied()
        .find(wgpu::TextureFormat::is_srgb)
        .or_else(|| capabilities.formats.first().copied())
        .ok_or_else(|| RenderError::UnsupportedFormat("surface has no color format".to_owned()))
}

fn create_accumulate_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("radia-accumulate-bind-group-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(UNIFORM_SIZE),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
    })
}

fn create_present_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("radia-present-bind-group-layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }],
    })
}

fn create_accumulate_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("radia-analytic-sdf-wgsl"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/radia.wgsl").into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("radia-accumulate-pipeline-layout"),
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("radia-accumulate-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_accumulate"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: HISTORY_FORMAT,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
        cache: None,
    })
}

fn create_present_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    output_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("radia-present-wgsl"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/present.wgsl").into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("radia-present-pipeline-layout"),
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("radia-present-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_present"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: output_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
        cache: None,
    })
}

fn install_error_handlers(device: &wgpu::Device, gpu_issues: Arc<Mutex<GpuIssues>>) {
    let uncaptured_issues = Arc::clone(&gpu_issues);
    device.on_uncaptured_error(Box::new(move |error| {
        if let Ok(mut issues) = uncaptured_issues.lock() {
            issues.uncaptured.push(error.to_string());
        }
    }));
    device.set_device_lost_callback(move |reason, message| {
        if let Ok(mut issues) = gpu_issues.lock() {
            issues.device_lost = Some(format!("{reason:?}: {message}"));
        }
    });
}

fn validate_extent(width: u32, height: u32) -> Result<(), RenderError> {
    if width == 0 || height == 0 {
        return Err(RenderError::InvalidConfig(
            "framebuffer width and height must be positive".to_owned(),
        ));
    }
    if width > u32::from(u16::MAX) || height > u32::from(u16::MAX) {
        return Err(RenderError::InvalidConfig(
            "framebuffer extent exceeds exact f32 adapter range".to_owned(),
        ));
    }
    Ok(())
}

fn validate_settings(settings: RenderSettings) -> Result<(), RenderError> {
    let finite_values = [
        settings.emitter_position.x,
        settings.emitter_position.y,
        settings.emitter_position.z,
        settings.emitter_radius,
        settings.emitter_color.x,
        settings.emitter_color.y,
        settings.emitter_color.z,
        settings.emitter_intensity,
        settings.maximum_distance,
        settings.hit_guard,
    ];
    if finite_values.iter().any(|value| !value.is_finite()) {
        return Err(RenderError::InvalidConfig(
            "render settings must be finite".to_owned(),
        ));
    }
    if settings.emitter_radius <= 0.0
        || settings.emitter_intensity < 0.0
        || settings.maximum_distance <= 0.0
        || settings.hit_guard <= 0.0
        || !(1..=128).contains(&settings.maximum_steps)
    {
        return Err(RenderError::InvalidConfig(
            "radius, distance, guard, intensity, or step bound is outside its domain".to_owned(),
        ));
    }
    Ok(())
}

fn settings_state_key(settings: RenderSettings, width: u32, height: u32) -> [u32; 23] {
    let camera = settings.camera_to_world.to_gpu_xyzw();
    [
        width,
        height,
        settings.mode.as_u32(),
        camera[0].to_bits(),
        camera[1].to_bits(),
        camera[2].to_bits(),
        camera[3].to_bits(),
        camera[4].to_bits(),
        camera[5].to_bits(),
        camera[6].to_bits(),
        camera[7].to_bits(),
        settings.emitter_position.x.to_bits(),
        settings.emitter_position.y.to_bits(),
        settings.emitter_position.z.to_bits(),
        settings.emitter_radius.to_bits(),
        settings.emitter_color.x.to_bits(),
        settings.emitter_color.y.to_bits(),
        settings.emitter_color.z.to_bits(),
        settings.emitter_intensity.to_bits(),
        settings.maximum_distance.to_bits(),
        settings.hit_guard.to_bits(),
        settings.maximum_steps,
        1,
    ]
}

fn encode_uniforms(
    settings: RenderSettings,
    width: u32,
    height: u32,
    frame_index: u32,
) -> [u8; UNIFORM_SIZE_BYTES] {
    let mut bytes = [0_u8; UNIFORM_SIZE_BYTES];
    write_f32(
        &mut bytes,
        0,
        f32::from(u16::try_from(width).unwrap_or(u16::MAX)),
    );
    write_f32(
        &mut bytes,
        4,
        f32::from(u16::try_from(height).unwrap_or(u16::MAX)),
    );
    write_u32(&mut bytes, 8, frame_index);
    write_u32(&mut bytes, 12, settings.mode.as_u32());
    let camera = settings.camera_to_world.to_gpu_xyzw();
    for (index, value) in camera.iter().enumerate() {
        write_f32(&mut bytes, 16 + index * 4, *value);
    }
    write_vec4(
        &mut bytes,
        48,
        [
            settings.emitter_position.x,
            settings.emitter_position.y,
            settings.emitter_position.z,
            settings.emitter_radius,
        ],
    );
    write_vec4(
        &mut bytes,
        64,
        [
            settings.emitter_color.x,
            settings.emitter_color.y,
            settings.emitter_color.z,
            settings.emitter_intensity,
        ],
    );
    write_vec4(
        &mut bytes,
        80,
        [
            settings.maximum_distance,
            settings.hit_guard,
            f32::from(u8::try_from(settings.maximum_steps).unwrap_or(u8::MAX)),
            0.0,
        ],
    );
    bytes
}

fn write_vec4(bytes: &mut [u8], offset: usize, value: [f32; 4]) {
    for (index, component) in value.iter().enumerate() {
        write_f32(bytes, offset + index * 4, *component);
    }
}

fn write_f32(bytes: &mut [u8], offset: usize, value: f32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_ne_bytes());
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_ne_bytes());
}

#[cfg(test)]
mod tests {
    use super::{RadiaMode, UNIFORM_SIZE_BYTES, default_render_settings, encode_uniforms};

    #[test]
    fn uniform_layout_is_exact_and_gpu_quaternions_are_xyzw() {
        let settings = default_render_settings(RadiaMode::Radia).expect("frozen scene is valid");
        let bytes = encode_uniforms(settings, 640, 360, 7);
        assert_eq!(bytes.len(), UNIFORM_SIZE_BYTES);
        assert_eq!(
            u32::from_ne_bytes(bytes[8..12].try_into().expect("four bytes")),
            7
        );
        assert_eq!(
            u32::from_ne_bytes(bytes[12..16].try_into().expect("four bytes")),
            1
        );
        let real_w = f32::from_ne_bytes(bytes[28..32].try_into().expect("four bytes"));
        assert!((real_w - 1.0).abs() <= f32::EPSILON);
    }

    #[test]
    fn changing_mode_changes_accumulation_key() {
        let off = default_render_settings(RadiaMode::Off).expect("frozen scene is valid");
        let radia = default_render_settings(RadiaMode::Radia).expect("frozen scene is valid");
        assert_ne!(
            super::settings_state_key(off, 64, 64),
            super::settings_state_key(radia, 64, 64)
        );
    }
}
