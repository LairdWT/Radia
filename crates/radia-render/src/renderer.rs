use std::num::NonZeroU64;
use std::sync::{Arc, Mutex};

use radia_math::{ErrorScale, UnitDualQuat, UnitQuat, Vec3};

use crate::RenderError;
use crate::dragon::{DragonFieldAsset, DragonFieldMetadata};
use crate::graph::{
    DEPTH_CLEAR, DEPTH_COMPARE, DEPTH_FORMAT, FrameTargets, GBUFFER_ALBEDO_FORMAT,
    GBUFFER_EMISSIVE_FORMAT, GBUFFER_NORMAL_MATERIAL_FORMAT, GBUFFER_TRACE_FORMAT,
    INDIRECT_RADIANCE_FORMAT, SCENE_RADIANCE_FORMAT, validate_pass_contracts,
};

pub const DRAGON_COUNT: usize = 3;
pub const LIGHT_COUNT: usize = 3;

const UNIFORM_SIZE: u64 = 304;
const UNIFORM_SIZE_BYTES: usize = 304;
const DRAGON_CLUSTER_CENTER: Vec3 = Vec3::new(0.0, -1.0, -4.6);
const DRAGON_CLUSTER_RADIUS: f32 = 2.0;
const DRAGON_TURN_RATE_RADIANS_PER_SECOND: f32 = 0.12;
const INITIAL_CLUSTER_PHASE: f32 = std::f32::consts::FRAC_PI_2;
const DEFAULT_CAMERA_POSITION: Vec3 = Vec3::new(0.0, 3.0, 2.328_203_2);
const DEFAULT_CAMERA_PITCH_RADIANS: f32 = -std::f32::consts::PI / 6.0;
const DEFAULT_VERTICAL_FOV_RADIANS: f32 = 65.0_f32.to_radians();

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneLight {
    pub position: Vec3,
    pub radius: f32,
    pub color: Vec3,
    pub intensity: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneMotion {
    pub dragons_to_world: [UnitDualQuat; DRAGON_COUNT],
    pub lights: [SceneLight; LIGHT_COUNT],
}

#[derive(Clone, Copy)]
struct LightOrbit {
    radius: f32,
    angular_speed: f32,
    phase: f32,
    base_height: f32,
    bob_amplitude: f32,
    bob_speed: f32,
    bob_phase: f32,
    light_radius: f32,
    color: Vec3,
    intensity: f32,
}

const LIGHT_ORBITS: [LightOrbit; LIGHT_COUNT] = [
    LightOrbit {
        radius: 3.1,
        angular_speed: 0.32,
        phase: std::f32::consts::PI,
        base_height: 1.30,
        bob_amplitude: 0.55,
        bob_speed: 0.77,
        bob_phase: 0.0,
        light_radius: 0.24,
        color: Vec3::new(1.0, 0.035, 0.015),
        intensity: 26.0,
    },
    LightOrbit {
        radius: 3.0,
        angular_speed: -0.23,
        phase: 0.0,
        base_height: 1.65,
        bob_amplitude: 0.65,
        bob_speed: 0.63,
        bob_phase: std::f32::consts::TAU / 3.0,
        light_radius: 0.25,
        color: Vec3::new(0.02, 1.0, 0.08),
        intensity: 22.0,
    },
    LightOrbit {
        radius: 2.5,
        angular_speed: 0.17,
        phase: std::f32::consts::FRAC_PI_2,
        base_height: 2.40,
        bob_amplitude: 0.75,
        bob_speed: 0.49,
        bob_phase: 2.0 * std::f32::consts::TAU / 3.0,
        light_radius: 0.27,
        color: Vec3::new(0.025, 0.12, 1.0),
        intensity: 24.0,
    },
];

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
    Albedo,
    Emissive,
    LinearDepth,
    AmbientOcclusion,
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
            Self::Albedo => 9,
            Self::Emissive => 10,
            Self::LinearDepth => 11,
            Self::AmbientOcclusion => 12,
        }
    }

    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Off => Self::GiOnly,
            Self::Radia => Self::Off,
            Self::GiOnly => Self::Albedo,
            Self::Albedo => Self::Normal,
            Self::Normal => Self::Emissive,
            Self::Emissive => Self::LinearDepth,
            Self::LinearDepth => Self::AmbientOcclusion,
            Self::AmbientOcclusion => Self::SdfDistance,
            Self::SdfDistance => Self::PrimitiveId,
            Self::PrimitiveId => Self::StepCount,
            Self::StepCount => Self::HitState,
            Self::HitState => Self::Triangle,
            Self::Triangle => Self::Radia,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderSettings {
    pub mode: RadiaMode,
    pub camera_to_world: UnitDualQuat,
    pub dragons_to_world: [UnitDualQuat; DRAGON_COUNT],
    pub lights: [SceneLight; LIGHT_COUNT],
    pub vertical_fov: f32,
    pub near: f32,
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
    let camera_rotation =
        UnitQuat::try_from_axis_angle(Vec3::X, DEFAULT_CAMERA_PITCH_RADIANS, error_scale)?;
    let camera_to_world = UnitDualQuat::from_rotation_translation(
        camera_rotation,
        DEFAULT_CAMERA_POSITION,
        error_scale,
    )?;
    let motion = scene_motion(0.0)?;
    Ok(RenderSettings {
        mode,
        camera_to_world,
        dragons_to_world: motion.dragons_to_world,
        lights: motion.lights,
        vertical_fov: DEFAULT_VERTICAL_FOV_RADIANS,
        near: 0.1,
        maximum_distance: 40.0,
        hit_guard: 0.000_5,
        maximum_steps: 128,
    })
}

/// Evaluates every rigid pose and light from one finite analytic scene clock.
///
/// # Errors
///
/// Returns an error when time is non-finite or a normalized pose cannot be
/// constructed.
pub fn scene_motion(elapsed_seconds: f32) -> Result<SceneMotion, RenderError> {
    let cluster_phase = scene_cluster_phase(elapsed_seconds)?;
    Ok(SceneMotion {
        dragons_to_world: dragon_triad_pose(cluster_phase)?,
        lights: LIGHT_ORBITS.map(|orbit| light_at_time(orbit, elapsed_seconds)),
    })
}

/// Returns the wrapped analytic cluster phase for a finite scene time.
///
/// # Errors
///
/// Returns an error when elapsed time is non-finite.
pub fn scene_cluster_phase(elapsed_seconds: f32) -> Result<f32, RenderError> {
    if !elapsed_seconds.is_finite() {
        return Err(RenderError::InvalidConfig(
            "scene time must be finite".to_owned(),
        ));
    }
    Ok((INITIAL_CLUSTER_PHASE
        + elapsed_seconds.rem_euclid(std::f32::consts::TAU / DRAGON_TURN_RATE_RADIANS_PER_SECOND)
            * DRAGON_TURN_RATE_RADIANS_PER_SECOND)
        .rem_euclid(std::f32::consts::TAU))
}

/// Builds the three outward-facing dragon poses at an explicit cluster angle.
///
/// # Errors
///
/// Returns an error when the angle is non-finite or a normalized rigid pose
/// cannot be constructed.
pub fn dragon_triad_pose(
    cluster_angle_radians: f32,
) -> Result<[UnitDualQuat; DRAGON_COUNT], RenderError> {
    if !cluster_angle_radians.is_finite() {
        return Err(RenderError::InvalidConfig(
            "dragon cluster angle must be finite".to_owned(),
        ));
    }
    let error_scale = ErrorScale::new(100.0, 64)?;
    let phase = cluster_angle_radians.rem_euclid(std::f32::consts::TAU);
    let separation = std::f32::consts::TAU / 3.0;
    Ok([
        dragon_instance_pose(phase, error_scale)?,
        dragon_instance_pose(phase + separation, error_scale)?,
        dragon_instance_pose(phase + 2.0 * separation, error_scale)?,
    ])
}

/// Replaces only the animated scene state, preserving camera and render mode.
///
/// # Errors
///
/// Returns an error when elapsed time is non-finite or pose construction fails.
pub fn apply_scene_motion(
    settings: &mut RenderSettings,
    elapsed_seconds: f32,
) -> Result<(), RenderError> {
    let motion = scene_motion(elapsed_seconds)?;
    settings.dragons_to_world = motion.dragons_to_world;
    settings.lights = motion.lights;
    Ok(())
}

fn dragon_instance_pose(
    radial_angle: f32,
    error_scale: ErrorScale,
) -> Result<UnitDualQuat, RenderError> {
    let outward = Vec3::new(radial_angle.cos(), 0.0, radial_angle.sin());
    let translation = DRAGON_CLUSTER_CENTER + outward * DRAGON_CLUSTER_RADIUS;
    let yaw = (-radial_angle - std::f32::consts::FRAC_PI_2).rem_euclid(std::f32::consts::TAU);
    let rotation = UnitQuat::try_from_axis_angle(Vec3::Y, yaw, error_scale)?;
    UnitDualQuat::from_rotation_translation(rotation, translation, error_scale)
        .map_err(RenderError::from)
}

fn light_at_time(orbit: LightOrbit, elapsed_seconds: f32) -> SceneLight {
    let orbit_period = std::f32::consts::TAU / orbit.angular_speed.abs();
    let bob_period = std::f32::consts::TAU / orbit.bob_speed;
    let angle = orbit.phase + orbit.angular_speed * elapsed_seconds.rem_euclid(orbit_period);
    let height = orbit.base_height
        + orbit.bob_amplitude
            * (orbit.bob_phase + orbit.bob_speed * elapsed_seconds.rem_euclid(bob_period)).sin();
    SceneLight {
        position: Vec3::new(
            orbit.radius * angle.cos(),
            height,
            DRAGON_CLUSTER_CENTER.z + orbit.radius * angle.sin(),
        ),
        radius: orbit.light_radius,
        color: orbit.color,
        intensity: orbit.intensity,
    }
}

#[derive(Debug, Default)]
struct GpuIssues {
    uncaptured: Vec<String>,
    device_lost: Option<String>,
}

struct FrameBindGroups {
    gbuffer: wgpu::BindGroup,
    direct: wgpu::BindGroup,
    indirect: wgpu::BindGroup,
    composite: wgpu::BindGroup,
    presentation: wgpu::BindGroup,
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
    dragon_buffer: wgpu::Buffer,
    dragon_metadata: DragonFieldMetadata,
    gbuffer_bind_group_layout: wgpu::BindGroupLayout,
    deferred_bind_group_layout: wgpu::BindGroupLayout,
    indirect_bind_group_layout: wgpu::BindGroupLayout,
    composite_bind_group_layout: wgpu::BindGroupLayout,
    present_bind_group_layout: wgpu::BindGroupLayout,
    gbuffer_pipeline: wgpu::RenderPipeline,
    deferred_pipeline: wgpu::RenderPipeline,
    indirect_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,
    present_pipeline: wgpu::RenderPipeline,
    targets: FrameTargets,
    frame_index: u32,
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
        validate_pass_contracts()?;
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
        let dragon_asset = load_dragon_asset(&adapter)?;
        let dragon_buffer_size = u64::try_from(dragon_asset.payload.len()).map_err(|_| {
            RenderError::InvalidConfig("dragon storage buffer size exceeds u64".to_owned())
        })?;

        validate_deferred_formats(&adapter, output_format)?;

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
        let dragon_buffer = create_dragon_buffer(&device, &dragon_asset, dragon_buffer_size);
        let gbuffer_bind_group_layout =
            create_gbuffer_bind_group_layout(&device, dragon_buffer_size);
        let deferred_bind_group_layout =
            create_deferred_bind_group_layout(&device, dragon_buffer_size);
        let indirect_bind_group_layout =
            create_indirect_bind_group_layout(&device, dragon_buffer_size);
        let composite_bind_group_layout = create_composite_bind_group_layout(&device);
        let present_bind_group_layout = create_present_bind_group_layout(&device);
        let gbuffer_pipeline = create_gbuffer_pipeline(&device, &gbuffer_bind_group_layout);
        let deferred_pipeline = create_deferred_pipeline(&device, &deferred_bind_group_layout);
        let indirect_pipeline = create_indirect_pipeline(&device, &indirect_bind_group_layout);
        let composite_pipeline = create_composite_pipeline(&device, &composite_bind_group_layout);
        let present_pipeline =
            create_present_pipeline(&device, &present_bind_group_layout, output_format);
        let targets = FrameTargets::new(&device, width, height);

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
            dragon_buffer,
            dragon_metadata: dragon_asset.metadata,
            gbuffer_bind_group_layout,
            deferred_bind_group_layout,
            indirect_bind_group_layout,
            composite_bind_group_layout,
            present_bind_group_layout,
            gbuffer_pipeline,
            deferred_pipeline,
            indirect_pipeline,
            composite_pipeline,
            present_pipeline,
            targets,
            frame_index: 0,
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

    /// Recreates graph-owned extent-dependent targets and resets frame sequencing.
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
        self.targets = FrameTargets::new(&self.device, width, height);
        self.reset_frame_sequence();
        Ok(())
    }

    pub fn reset_frame_sequence(&mut self) {
        self.frame_index = 0;
    }

    /// Renders one deterministic current-frame deferred result.
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
        let uniform_bytes = encode_uniforms(
            settings,
            self.width,
            self.height,
            self.frame_index,
            self.dragon_metadata,
        );
        self.queue
            .write_buffer(&self.uniform_buffer, 0, &uniform_bytes);

        let bind_groups = self.create_frame_bind_groups();

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("radia-frame-encoder"),
            });
        self.encode_frame(output_view, &bind_groups, &mut encoder);
        self.queue.submit(Some(encoder.finish()));
        self.frame_index = self.frame_index.saturating_add(1);
        self.device
            .poll(wgpu::PollType::Poll)
            .map_err(|error| RenderError::DevicePoll(error.to_string()))?;
        self.check_gpu_issues()
    }

    fn create_frame_bind_groups(&self) -> FrameBindGroups {
        let gbuffer = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("radia-gbuffer-bind-group"),
            layout: &self.gbuffer_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.dragon_buffer.as_entire_binding(),
                },
            ],
        });
        let direct = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("radia-deferred-bind-group"),
            layout: &self.deferred_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.dragon_buffer.as_entire_binding(),
                },
                texture_bind_entry(2, &self.targets.gbuffer_albedo),
                texture_bind_entry(3, &self.targets.gbuffer_normal_material),
                texture_bind_entry(4, &self.targets.gbuffer_emissive),
                texture_bind_entry(5, &self.targets.gbuffer_trace),
                texture_bind_entry(6, &self.targets.depth),
            ],
        });
        let indirect = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("radia-indirect-bind-group"),
            layout: &self.indirect_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.dragon_buffer.as_entire_binding(),
                },
                texture_bind_entry(2, &self.targets.gbuffer_albedo),
                texture_bind_entry(3, &self.targets.gbuffer_normal_material),
                texture_bind_entry(4, &self.targets.gbuffer_trace),
                texture_bind_entry(5, &self.targets.depth),
                texture_bind_entry(6, &self.targets.direct_radiance),
            ],
        });
        let composite = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("radia-composite-bind-group"),
            layout: &self.composite_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                texture_bind_entry(1, &self.targets.direct_radiance),
                texture_bind_entry(2, &self.targets.indirect_radiance),
                texture_bind_entry(3, &self.targets.gbuffer_normal_material),
                texture_bind_entry(4, &self.targets.depth),
            ],
        });
        let presentation = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("radia-present-bind-group"),
            layout: &self.present_bind_group_layout,
            entries: &[
                texture_bind_entry(0, &self.targets.scene_radiance),
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
            ],
        });
        FrameBindGroups {
            gbuffer,
            direct,
            indirect,
            composite,
            presentation,
        }
    }

    fn encode_frame(
        &self,
        output_view: &wgpu::TextureView,
        bind_groups: &FrameBindGroups,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        self.encode_gbuffer(bind_groups, encoder);
        Self::encode_single_target(
            "radia-deferred-lighting-pass",
            &self.targets.direct_radiance,
            &self.deferred_pipeline,
            &bind_groups.direct,
            encoder,
        );
        Self::encode_single_target(
            "radia-screen-space-indirect-pass",
            &self.targets.indirect_radiance,
            &self.indirect_pipeline,
            &bind_groups.indirect,
            encoder,
        );
        Self::encode_single_target(
            "radia-deferred-composite-pass",
            &self.targets.scene_radiance,
            &self.composite_pipeline,
            &bind_groups.composite,
            encoder,
        );
        Self::encode_single_target(
            "radia-presentation-pass",
            output_view,
            &self.present_pipeline,
            &bind_groups.presentation,
            encoder,
        );
    }

    fn encode_gbuffer(&self, bind_groups: &FrameBindGroups, encoder: &mut wgpu::CommandEncoder) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("radia-gbuffer-geometry-pass"),
            color_attachments: &[
                Some(clear_attachment(&self.targets.gbuffer_albedo)),
                Some(clear_attachment(&self.targets.gbuffer_normal_material)),
                Some(clear_attachment(&self.targets.gbuffer_emissive)),
                Some(clear_attachment(&self.targets.gbuffer_trace)),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.targets.depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(DEPTH_CLEAR),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&self.gbuffer_pipeline);
        pass.set_bind_group(0, &bind_groups.gbuffer, &[]);
        pass.draw(0..3, 0..1);
    }

    fn encode_single_target(
        label: &'static str,
        target: &wgpu::TextureView,
        pipeline: &wgpu::RenderPipeline,
        bind_group: &wgpu::BindGroup,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[Some(clear_attachment(target))],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);
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

fn load_dragon_asset(adapter: &wgpu::Adapter) -> Result<DragonFieldAsset, RenderError> {
    let asset = DragonFieldAsset::embedded()?;
    let size = u64::try_from(asset.payload.len()).map_err(|_| {
        RenderError::InvalidConfig("dragon storage buffer size exceeds u64".to_owned())
    })?;
    if size > u64::from(adapter.limits().max_storage_buffer_binding_size) {
        return Err(RenderError::UnsupportedAdapter(format!(
            "dragon field needs {size} storage bytes, adapter limit is {}",
            adapter.limits().max_storage_buffer_binding_size
        )));
    }
    Ok(asset)
}

fn create_dragon_buffer(
    device: &wgpu::Device,
    asset: &DragonFieldAsset,
    size: u64,
) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("radia-dragon-udf-storage"),
        size,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: true,
    });
    buffer
        .slice(..)
        .get_mapped_range_mut()
        .copy_from_slice(&asset.payload);
    buffer.unmap();
    buffer
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

fn clear_attachment(view: &wgpu::TextureView) -> wgpu::RenderPassColorAttachment<'_> {
    wgpu::RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            store: wgpu::StoreOp::Store,
        },
    }
}

fn validate_deferred_formats(
    adapter: &wgpu::Adapter,
    output_format: wgpu::TextureFormat,
) -> Result<(), RenderError> {
    let required_color_usages =
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
    for format in [
        GBUFFER_ALBEDO_FORMAT,
        GBUFFER_NORMAL_MATERIAL_FORMAT,
        GBUFFER_EMISSIVE_FORMAT,
        GBUFFER_TRACE_FORMAT,
        SCENE_RADIANCE_FORMAT,
        INDIRECT_RADIANCE_FORMAT,
    ] {
        let features = adapter.get_texture_format_features(format);
        if !features.allowed_usages.contains(required_color_usages) {
            return Err(RenderError::UnsupportedFormat(format!(
                "{format:?} lacks {required_color_usages:?} on {}",
                adapter.get_info().name
            )));
        }
    }
    let depth_usages =
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
    let depth_features = adapter.get_texture_format_features(DEPTH_FORMAT);
    if !depth_features.allowed_usages.contains(depth_usages) {
        return Err(RenderError::UnsupportedFormat(format!(
            "{DEPTH_FORMAT:?} lacks {depth_usages:?} on {}",
            adapter.get_info().name
        )));
    }
    let output_features = adapter.get_texture_format_features(output_format);
    if !output_features
        .allowed_usages
        .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
    {
        return Err(RenderError::UnsupportedFormat(format!(
            "{output_format:?} is not renderable on {}",
            adapter.get_info().name
        )));
    }
    Ok(())
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

fn create_gbuffer_bind_group_layout(
    device: &wgpu::Device,
    dragon_buffer_size: u64,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("radia-gbuffer-bind-group-layout"),
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
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(dragon_buffer_size),
                },
                count: None,
            },
        ],
    })
}

fn create_deferred_bind_group_layout(
    device: &wgpu::Device,
    dragon_buffer_size: u64,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("radia-deferred-bind-group-layout"),
        entries: &[
            uniform_layout_entry(0),
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(dragon_buffer_size),
                },
                count: None,
            },
            float_texture_layout_entry(2),
            float_texture_layout_entry(3),
            float_texture_layout_entry(4),
            float_texture_layout_entry(5),
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
    })
}

fn create_indirect_bind_group_layout(
    device: &wgpu::Device,
    dragon_buffer_size: u64,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("radia-indirect-bind-group-layout"),
        entries: &[
            uniform_layout_entry(0),
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(dragon_buffer_size),
                },
                count: None,
            },
            float_texture_layout_entry(2),
            float_texture_layout_entry(3),
            float_texture_layout_entry(4),
            depth_texture_layout_entry(5),
            float_texture_layout_entry(6),
        ],
    })
}

fn create_composite_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("radia-composite-bind-group-layout"),
        entries: &[
            uniform_layout_entry(0),
            float_texture_layout_entry(1),
            float_texture_layout_entry(2),
            float_texture_layout_entry(3),
            depth_texture_layout_entry(4),
        ],
    })
}

const fn uniform_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: NonZeroU64::new(UNIFORM_SIZE),
        },
        count: None,
    }
}

const fn float_texture_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

const fn depth_texture_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Depth,
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn create_present_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("radia-present-bind-group-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            uniform_layout_entry(1),
        ],
    })
}

fn create_gbuffer_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("radia-gbuffer-wgsl"),
        source: wgpu::ShaderSource::Wgsl(
            concat!(
                include_str!("shaders/scene.wgsl"),
                "\n",
                include_str!("shaders/gbuffer.wgsl")
            )
            .into(),
        ),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("radia-gbuffer-pipeline-layout"),
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("radia-gbuffer-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_gbuffer"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: DEPTH_COMPARE,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_gbuffer"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[
                Some(color_target(GBUFFER_ALBEDO_FORMAT)),
                Some(color_target(GBUFFER_NORMAL_MATERIAL_FORMAT)),
                Some(color_target(GBUFFER_EMISSIVE_FORMAT)),
                Some(color_target(GBUFFER_TRACE_FORMAT)),
            ],
        }),
        multiview: None,
        cache: None,
    })
}

fn create_deferred_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("radia-deferred-wgsl"),
        source: wgpu::ShaderSource::Wgsl(
            concat!(
                include_str!("shaders/scene.wgsl"),
                "\n",
                include_str!("shaders/deferred.wgsl")
            )
            .into(),
        ),
    });
    create_single_target_pipeline(
        device,
        "radia-deferred-pipeline",
        bind_group_layout,
        &shader,
        "vs_deferred",
        "fs_deferred",
        SCENE_RADIANCE_FORMAT,
    )
}

fn create_indirect_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("radia-ssgi-wgsl"),
        source: wgpu::ShaderSource::Wgsl(
            concat!(
                include_str!("shaders/scene.wgsl"),
                "\n",
                include_str!("shaders/ssgi.wgsl")
            )
            .into(),
        ),
    });
    create_single_target_pipeline(
        device,
        "radia-ssgi-pipeline",
        bind_group_layout,
        &shader,
        "vs_ssgi",
        "fs_ssgi",
        INDIRECT_RADIANCE_FORMAT,
    )
}

fn create_composite_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("radia-composite-wgsl"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/composite.wgsl").into()),
    });
    create_single_target_pipeline(
        device,
        "radia-composite-pipeline",
        bind_group_layout,
        &shader,
        "vs_composite",
        "fs_composite",
        SCENE_RADIANCE_FORMAT,
    )
}

fn create_single_target_pipeline(
    device: &wgpu::Device,
    label: &'static str,
    bind_group_layout: &wgpu::BindGroupLayout,
    shader: &wgpu::ShaderModule,
    vertex_entry: &'static str,
    fragment_entry: &'static str,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some(vertex_entry),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(fragment_entry),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(color_target(format))],
        }),
        multiview: None,
        cache: None,
    })
}

const fn color_target(format: wgpu::TextureFormat) -> wgpu::ColorTargetState {
    wgpu::ColorTargetState {
        format,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    }
}

fn texture_bind_entry(binding: u32, view: &wgpu::TextureView) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(view),
    }
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
        settings.vertical_fov,
        settings.near,
        settings.maximum_distance,
        settings.hit_guard,
    ];
    if finite_values.iter().any(|value| !value.is_finite()) {
        return Err(RenderError::InvalidConfig(
            "render settings must be finite".to_owned(),
        ));
    }
    if settings.lights.iter().any(|light| {
        !light.position.is_finite()
            || !light.radius.is_finite()
            || light.radius <= 0.0
            || !light.color.is_finite()
            || light.color.x < 0.0
            || light.color.y < 0.0
            || light.color.z < 0.0
            || !light.intensity.is_finite()
            || light.intensity < 0.0
    }) || settings.vertical_fov <= 0.0
        || settings.vertical_fov >= std::f32::consts::PI
        || settings.near <= 0.0
        || settings.maximum_distance <= 0.0
        || settings.hit_guard <= 0.0
        || !(1..=128).contains(&settings.maximum_steps)
    {
        return Err(RenderError::InvalidConfig(
            "radius, projection, distance, guard, intensity, or step bound is outside its domain"
                .to_owned(),
        ));
    }
    Ok(())
}

fn encode_uniforms(
    settings: RenderSettings,
    width: u32,
    height: u32,
    frame_index: u32,
    dragon: DragonFieldMetadata,
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
    for (index, light) in settings.lights.iter().enumerate() {
        write_vec4(
            &mut bytes,
            48 + index * 16,
            [
                light.position.x,
                light.position.y,
                light.position.z,
                light.radius,
            ],
        );
        write_vec4(
            &mut bytes,
            96 + index * 16,
            [light.color.x, light.color.y, light.color.z, light.intensity],
        );
    }
    write_vec4(
        &mut bytes,
        144,
        [
            settings.maximum_distance,
            settings.hit_guard,
            f32::from(u8::try_from(settings.maximum_steps).unwrap_or(u8::MAX)),
            0.0,
        ],
    );
    write_vec4(
        &mut bytes,
        160,
        [
            dragon.minimum[0],
            dragon.minimum[1],
            dragon.minimum[2],
            dragon.conservative_error,
        ],
    );
    write_vec4(
        &mut bytes,
        176,
        [
            dragon.maximum[0],
            dragon.maximum[1],
            dragon.maximum[2],
            f32::from(u16::try_from(dragon.resolution[0]).unwrap_or(u16::MAX)),
        ],
    );
    write_vec4(
        &mut bytes,
        192,
        [(settings.vertical_fov * 0.5).tan(), settings.near, 0.0, 0.0],
    );
    for (dragon_index, pose) in settings.dragons_to_world.iter().enumerate() {
        let packed = pose.to_gpu_xyzw();
        write_vec4(
            &mut bytes,
            208 + dragon_index * 16,
            [packed[0], packed[1], packed[2], packed[3]],
        );
        write_vec4(
            &mut bytes,
            256 + dragon_index * 16,
            [packed[4], packed[5], packed[6], packed[7]],
        );
    }
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
    use crate::dragon::DragonFieldAsset;
    use radia_math::Vec3;

    use super::{
        DEFAULT_CAMERA_PITCH_RADIANS, DEFAULT_CAMERA_POSITION, DEFAULT_VERTICAL_FOV_RADIANS,
        DRAGON_CLUSTER_CENTER, DRAGON_CLUSTER_RADIUS, DRAGON_COUNT, LIGHT_COUNT, RadiaMode,
        UNIFORM_SIZE_BYTES, default_render_settings, dragon_triad_pose, encode_uniforms,
        scene_motion, validate_settings,
    };

    #[test]
    fn uniform_layout_is_exact_and_gpu_quaternions_are_xyzw() {
        let settings = default_render_settings(RadiaMode::Radia).expect("frozen scene is valid");
        let dragon = DragonFieldAsset::embedded()
            .expect("embedded field is valid")
            .metadata;
        let bytes = encode_uniforms(settings, 640, 360, 7, dragon);
        assert_eq!(bytes.len(), UNIFORM_SIZE_BYTES);
        assert_eq!(
            u32::from_ne_bytes(bytes[8..12].try_into().expect("four bytes")),
            7
        );
        assert_eq!(
            u32::from_ne_bytes(bytes[12..16].try_into().expect("four bytes")),
            1
        );
        let packed_camera = settings.camera_to_world.to_gpu_xyzw();
        for component in 0..4 {
            let observed_real = f32::from_ne_bytes(
                bytes[16 + component * 4..20 + component * 4]
                    .try_into()
                    .expect("four bytes"),
            );
            let observed_dual = f32::from_ne_bytes(
                bytes[32 + component * 4..36 + component * 4]
                    .try_into()
                    .expect("four bytes"),
            );
            assert!((observed_real - packed_camera[component]).abs() <= f32::EPSILON);
            assert!((observed_dual - packed_camera[component + 4]).abs() <= f32::EPSILON);
        }
        let dragon_resolution = f32::from_ne_bytes(bytes[188..192].try_into().expect("four bytes"));
        assert!((dragon_resolution - 128.0).abs() <= f32::EPSILON);
        let tangent_half_fov = f32::from_ne_bytes(bytes[192..196].try_into().expect("four bytes"));
        let near = f32::from_ne_bytes(bytes[196..200].try_into().expect("four bytes"));
        assert!((tangent_half_fov - (settings.vertical_fov * 0.5).tan()).abs() <= f32::EPSILON);
        assert!((near - settings.near).abs() <= f32::EPSILON);
        for (light_index, light) in settings.lights.iter().enumerate() {
            let position_offset = 48 + light_index * 16;
            let color_offset = 96 + light_index * 16;
            for (offset, expected) in [
                (position_offset, light.position.x),
                (position_offset + 4, light.position.y),
                (position_offset + 8, light.position.z),
                (position_offset + 12, light.radius),
                (color_offset, light.color.x),
                (color_offset + 4, light.color.y),
                (color_offset + 8, light.color.z),
                (color_offset + 12, light.intensity),
            ] {
                let observed =
                    f32::from_ne_bytes(bytes[offset..offset + 4].try_into().expect("four bytes"));
                assert!((observed - expected).abs() <= f32::EPSILON);
            }
        }
        for (dragon_index, pose) in settings.dragons_to_world.iter().enumerate() {
            let packed = pose.to_gpu_xyzw();
            for component in 0..4 {
                let real_offset = 208 + dragon_index * 16 + component * 4;
                let dual_offset = 256 + dragon_index * 16 + component * 4;
                let observed_real = f32::from_ne_bytes(
                    bytes[real_offset..real_offset + 4]
                        .try_into()
                        .expect("four bytes"),
                );
                let observed_dual = f32::from_ne_bytes(
                    bytes[dual_offset..dual_offset + 4]
                        .try_into()
                        .expect("four bytes"),
                );
                assert!((observed_real - packed[component]).abs() <= f32::EPSILON);
                assert!((observed_dual - packed[component + 4]).abs() <= f32::EPSILON);
            }
        }
    }

    #[test]
    fn projection_settings_are_finite_and_inside_the_declared_domain() {
        let mut settings = default_render_settings(RadiaMode::Off).expect("frozen scene is valid");
        settings.near = 0.0;
        assert!(validate_settings(settings).is_err());
        settings.near = 0.1;
        settings.vertical_fov = std::f32::consts::PI;
        assert!(validate_settings(settings).is_err());
    }

    #[test]
    fn default_camera_is_quaternion_only_and_targets_the_cluster_from_thirty_degrees() {
        let settings = default_render_settings(RadiaMode::Off).expect("frozen scene is valid");
        let camera_position = settings.camera_to_world.translation();
        let forward = settings.camera_to_world.transform_direction(-Vec3::Z);
        let to_target = DRAGON_CLUSTER_CENTER - camera_position;
        let target_direction = to_target / to_target.length();
        let guard = f32::EPSILON * 100.0 * 64.0;

        assert!((camera_position - DEFAULT_CAMERA_POSITION).length() <= guard);
        assert!((forward.dot(target_direction) - 1.0).abs() <= guard);
        assert!((forward.y + 0.5).abs() <= guard);
        assert!((DEFAULT_CAMERA_PITCH_RADIANS.to_degrees() + 30.0).abs() <= guard);
        assert!((settings.vertical_fov - DEFAULT_VERTICAL_FOV_RADIANS).abs() <= f32::EPSILON);
        assert!((settings.vertical_fov.to_degrees() - 65.0).abs() <= f32::EPSILON * 64.0);
        assert!(settings.camera_to_world.study_dot().abs() <= guard);
    }

    #[test]
    fn display_modes_have_stable_values_and_cycle_once() {
        let expected = [
            (RadiaMode::Off, 0),
            (RadiaMode::Radia, 1),
            (RadiaMode::GiOnly, 2),
            (RadiaMode::SdfDistance, 3),
            (RadiaMode::PrimitiveId, 4),
            (RadiaMode::Normal, 5),
            (RadiaMode::StepCount, 6),
            (RadiaMode::HitState, 7),
            (RadiaMode::Triangle, 8),
            (RadiaMode::Albedo, 9),
            (RadiaMode::Emissive, 10),
            (RadiaMode::LinearDepth, 11),
            (RadiaMode::AmbientOcclusion, 12),
        ];
        for (mode, value) in expected {
            assert_eq!(mode.as_u32(), value);
        }

        let mut mode = RadiaMode::Radia;
        let mut visited = Vec::new();
        loop {
            assert!(!visited.contains(&mode));
            visited.push(mode);
            mode = mode.next();
            if mode == RadiaMode::Radia {
                break;
            }
        }
        assert_eq!(visited.len(), expected.len());
    }

    #[test]
    fn scene_motion_rejects_non_finite_inputs() {
        assert!(scene_motion(f32::NAN).is_err());
        assert!(scene_motion(f32::INFINITY).is_err());
        assert!(dragon_triad_pose(f32::NEG_INFINITY).is_err());
    }

    #[test]
    fn dragon_triad_is_radial_outward_and_tail_side_is_inward() {
        let poses =
            dragon_triad_pose(std::f32::consts::FRAC_PI_2).expect("frozen cluster angle is valid");
        assert_eq!(poses.len(), DRAGON_COUNT);
        let radial_guard = f32::EPSILON * 100.0 * 64.0;
        for pose in poses {
            let radial = pose.translation() - DRAGON_CLUSTER_CENTER;
            assert!((radial.length() - DRAGON_CLUSTER_RADIUS).abs() <= radial_guard);
            let outward = radial / radial.length();
            let head = pose.transform_direction(-Vec3::Z);
            let tail = pose.transform_direction(Vec3::Z);
            assert!((head.dot(outward) - 1.0).abs() <= radial_guard);
            assert!((tail.dot(outward) + 1.0).abs() <= radial_guard);
            assert!(pose.study_dot().abs() <= radial_guard);
        }
        let first = (poses[0].translation() - DRAGON_CLUSTER_CENTER) / DRAGON_CLUSTER_RADIUS;
        let second = (poses[1].translation() - DRAGON_CLUSTER_CENTER) / DRAGON_CLUSTER_RADIUS;
        let third = (poses[2].translation() - DRAGON_CLUSTER_CENTER) / DRAGON_CLUSTER_RADIUS;
        assert!((first.dot(second) + 0.5).abs() <= radial_guard);
        assert!((second.dot(third) + 0.5).abs() <= radial_guard);
        assert!((third.dot(first) + 0.5).abs() <= radial_guard);

        let dragon = DragonFieldAsset::embedded().expect("embedded field is valid");
        let tail_extent = dragon.metadata.maximum[2];
        assert!(DRAGON_CLUSTER_RADIUS - tail_extent >= 0.53);
        let expected_pairwise_distance = 3.0_f32.sqrt() * DRAGON_CLUSTER_RADIUS;
        for first_index in 0..DRAGON_COUNT {
            for second_index in first_index + 1..DRAGON_COUNT {
                let distance =
                    (poses[first_index].translation() - poses[second_index].translation()).length();
                assert!((distance - expected_pairwise_distance).abs() <= radial_guard);

                let separating_axis = (poses[first_index].translation() - DRAGON_CLUSTER_CENTER)
                    / DRAGON_CLUSTER_RADIUS;
                let x_extent = dragon.metadata.minimum[0]
                    .abs()
                    .max(dragon.metadata.maximum[0].abs());
                let z_extent = dragon.metadata.minimum[2]
                    .abs()
                    .max(dragon.metadata.maximum[2].abs());
                let projected_radius = |pose: radia_math::UnitDualQuat| {
                    x_extent * separating_axis.dot(pose.transform_direction(Vec3::X)).abs()
                        + z_extent * separating_axis.dot(pose.transform_direction(Vec3::Z)).abs()
                };
                let projected_center_distance = (poses[second_index].translation()
                    - poses[first_index].translation())
                .dot(separating_axis)
                .abs();
                let separating_clearance = projected_center_distance
                    - projected_radius(poses[first_index])
                    - projected_radius(poses[second_index]);
                assert!(separating_clearance >= 0.19);
            }
        }
    }

    #[test]
    fn orbital_lights_are_repeatable_distinct_and_bounded() {
        let start = scene_motion(0.0).expect("frozen time is valid");
        let repeat = scene_motion(0.0).expect("frozen time is valid");
        let later = scene_motion(1.0).expect("frozen time is valid");
        assert_eq!(start, repeat);
        assert_eq!(start.lights.len(), LIGHT_COUNT);
        for index in 0..LIGHT_COUNT {
            assert_ne!(start.lights[index].position, later.lights[index].position);
            let orbit = super::LIGHT_ORBITS[index];
            let horizontal = Vec3::new(
                start.lights[index].position.x,
                0.0,
                start.lights[index].position.z - DRAGON_CLUSTER_CENTER.z,
            );
            assert!((horizontal.length() - orbit.radius).abs() <= f32::EPSILON * 32.0);
            assert!(
                start.lights[index].position.y
                    >= orbit.base_height - orbit.bob_amplitude - f32::EPSILON
            );
            assert!(
                start.lights[index].position.y
                    <= orbit.base_height + orbit.bob_amplitude + f32::EPSILON
            );
        }
        assert!(
            (super::LIGHT_ORBITS[0].angular_speed - super::LIGHT_ORBITS[1].angular_speed).abs()
                > f32::EPSILON
        );
        assert!(
            (super::LIGHT_ORBITS[1].angular_speed - super::LIGHT_ORBITS[2].angular_speed).abs()
                > f32::EPSILON
        );
    }

    #[test]
    fn shader_sources_expose_the_deferred_resource_boundary() {
        let scene = include_str!("shaders/scene.wgsl");
        let gbuffer = include_str!("shaders/gbuffer.wgsl");
        let deferred = include_str!("shaders/deferred.wgsl");
        let ssgi = include_str!("shaders/ssgi.wgsl");
        let composite = include_str!("shaders/composite.wgsl");
        let present = include_str!("shaders/present.wgsl");
        assert!(scene.contains("camera_real_xyzw"));
        assert!(scene.contains("dragon_real_xyzw: array<vec4<f32>, 3>"));
        assert!(scene.contains("light_position_radius: array<vec4<f32>, 3>"));
        assert!(scene.contains("dragon_index"));
        assert!(scene.contains("dragon_index < 3u"));
        assert!(scene.contains("const MATERIAL_DRAGON_CYAN: u32 = 5u"));
        assert!(scene.contains("const MATERIAL_DRAGON_MAGENTA: u32 = 6u"));
        assert!(scene.contains("const MATERIAL_DRAGON_YELLOW: u32 = 7u"));
        assert!(scene.contains("fn dragon_material"));
        assert!(scene.contains("fn is_dragon"));
        assert!(scene.contains("fn distribution_ggx"));
        assert!(scene.contains("fn geometry_smith"));
        assert!(scene.contains("fn fresnel_schlick"));
        assert!(scene.contains("fn environment_lighting"));
        assert!(scene.contains("return 0.2"));
        assert!(scene.contains("return 0.65"));
        assert!(scene.contains("return 0.9"));
        assert!(scene.contains("return 1.0"));
        assert!(!scene.contains("let gloss ="));
        assert!(!scene.contains("material == MATERIAL_DRAGON {"));
        assert!(!scene.contains("return vec4<f32>(2.6, 1.75, -3.6"));
        assert!(scene.contains("reconstruct_world_position"));
        assert_eq!(gbuffer.matches("@location(").count(), 4);
        assert!(gbuffer.contains("@builtin(frag_depth)"));
        assert!(gbuffer.contains("if hit.state != TRACE_HIT"));
        assert!(gbuffer.contains("diagnostic_trace_mode"));
        assert!(deferred.contains("var gbuffer_depth: texture_depth_2d"));
        assert!(deferred.contains("textureLoad(gbuffer_normal_material"));
        assert!(deferred.contains("MODE_LINEAR_DEPTH"));
        assert!(ssgi.contains("var direct_radiance: texture_2d<f32>"));
        assert!(ssgi.contains("array<vec2<i32>, 32>"));
        assert!(ssgi.contains("let phase_order = array<u32, 16>"));
        assert!(ssgi.contains("any(tap < vec2<i32>(0))"));
        assert!(!ssgi.contains("let tap = clamp("));
        assert!(ssgi.contains("accessibility"));
        assert!(ssgi.contains("is_dragon(tap_material)"));
        assert!(composite.contains("fn resolved_indirect"));
        assert!(composite.contains("texture_depth_2d"));
        assert!(composite.contains("var indirect_radiance: texture_2d<f32>"));
        assert!(present.contains("@binding(1) var<uniform> frame"));
        for source in [scene, gbuffer, deferred, ssgi, composite, present] {
            assert!(!source.contains("mat2x"));
            assert!(!source.contains("mat3x"));
            assert!(!source.contains("mat4x"));
        }
    }
}
