use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use radia_math::{ErrorScale, Vec3};
use radia_render::{
    CaptureConfig, EvidenceConfig, RadiaMode, RadiaRenderer, RenderError, RenderSettings,
    apply_scene_motion, block_on, capture_controlled_delta, capture_png, default_render_settings,
};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

fn main() -> Result<(), DemoError> {
    match parse_command(std::env::args().skip(1))? {
        Command::Window => run_window(),
        Command::Capture(config) => run_capture(&config),
        Command::Evidence(config) => run_evidence(&config),
    }
}

#[derive(Debug)]
enum DemoError {
    Arguments(String),
    Render(RenderError),
    EventLoop(String),
    Window(String),
    Surface(String),
}

impl fmt::Display for DemoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arguments(message) => write!(formatter, "argument error: {message}"),
            Self::Render(error) => write!(formatter, "render error: {error}"),
            Self::EventLoop(message) => write!(formatter, "event loop error: {message}"),
            Self::Window(message) => write!(formatter, "window error: {message}"),
            Self::Surface(message) => write!(formatter, "surface error: {message}"),
        }
    }
}

impl Error for DemoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Render(error) => Some(error),
            _ => None,
        }
    }
}

impl From<RenderError> for DemoError {
    fn from(error: RenderError) -> Self {
        Self::Render(error)
    }
}

enum Command {
    Window,
    Capture(CaptureConfig),
    Evidence(EvidenceConfig),
}

fn parse_command(mut arguments: impl Iterator<Item = String>) -> Result<Command, DemoError> {
    let Some(command) = arguments.next() else {
        return Ok(Command::Window);
    };
    if command == "evidence" {
        return parse_evidence_command(arguments);
    }
    if command != "capture" {
        return Err(DemoError::Arguments(format!(
            "unknown command '{command}'; expected 'capture', 'evidence', or no command"
        )));
    }

    let mut width = 640_u32;
    let mut height = 360_u32;
    let mut mode = RadiaMode::Radia;
    let mut scene_time_seconds = 0.0_f32;
    let mut dragon_angle_radians = None;
    let mut output_path = PathBuf::from("Temp/captures/radia.png");
    while let Some(flag) = arguments.next() {
        let value = arguments.next().ok_or_else(|| {
            DemoError::Arguments(format!("flag '{flag}' requires a following value"))
        })?;
        match flag.as_str() {
            "--width" => width = parse_u32("width", &value)?,
            "--height" => height = parse_u32("height", &value)?,
            "--mode" => mode = parse_mode(&value)?,
            "--scene-time-seconds" => {
                scene_time_seconds = parse_f32("scene time", &value)?;
            }
            "--dragon-angle-degrees" => {
                dragon_angle_radians = Some(parse_f32("dragon angle", &value)?.to_radians());
            }
            "--output" => output_path = PathBuf::from(value),
            _ => {
                return Err(DemoError::Arguments(format!(
                    "unknown capture flag '{flag}'"
                )));
            }
        }
    }
    Ok(Command::Capture(CaptureConfig {
        width,
        height,
        mode,
        scene_time_seconds,
        dragon_angle_radians,
        output_path,
    }))
}

fn parse_evidence_command(
    mut arguments: impl Iterator<Item = String>,
) -> Result<Command, DemoError> {
    let mut width = 320_u32;
    let mut height = 180_u32;
    let mut output_directory = PathBuf::from("Temp/evidence");
    while let Some(flag) = arguments.next() {
        let value = arguments.next().ok_or_else(|| {
            DemoError::Arguments(format!("flag '{flag}' requires a following value"))
        })?;
        match flag.as_str() {
            "--width" => width = parse_u32("width", &value)?,
            "--height" => height = parse_u32("height", &value)?,
            "--output-dir" => output_directory = PathBuf::from(value),
            _ => {
                return Err(DemoError::Arguments(format!(
                    "unknown evidence flag '{flag}'"
                )));
            }
        }
    }
    Ok(Command::Evidence(EvidenceConfig {
        width,
        height,
        output_directory,
    }))
}

fn parse_u32(name: &str, value: &str) -> Result<u32, DemoError> {
    value
        .parse::<u32>()
        .map_err(|error| DemoError::Arguments(format!("invalid {name} '{value}': {error}")))
}

fn parse_f32(name: &str, value: &str) -> Result<f32, DemoError> {
    let parsed = value
        .parse::<f32>()
        .map_err(|error| DemoError::Arguments(format!("invalid {name} '{value}': {error}")))?;
    if !parsed.is_finite() {
        return Err(DemoError::Arguments(format!(
            "invalid {name} '{value}': value must be finite"
        )));
    }
    Ok(parsed)
}

fn parse_mode(value: &str) -> Result<RadiaMode, DemoError> {
    match value {
        "off" => Ok(RadiaMode::Off),
        "radia" => Ok(RadiaMode::Radia),
        "gi" => Ok(RadiaMode::GiOnly),
        "sdf" => Ok(RadiaMode::SdfDistance),
        "primitive" => Ok(RadiaMode::PrimitiveId),
        "normal" => Ok(RadiaMode::Normal),
        "steps" => Ok(RadiaMode::StepCount),
        "hit" => Ok(RadiaMode::HitState),
        "triangle" => Ok(RadiaMode::Triangle),
        "albedo" => Ok(RadiaMode::Albedo),
        "emissive" => Ok(RadiaMode::Emissive),
        "depth" => Ok(RadiaMode::LinearDepth),
        "ao" => Ok(RadiaMode::AmbientOcclusion),
        _ => Err(DemoError::Arguments(format!(
            "unknown mode '{value}'; expected triangle|off|radia|gi|albedo|normal|emissive|depth|ao|sdf|primitive|steps|hit"
        ))),
    }
}

fn run_capture(config: &CaptureConfig) -> Result<(), DemoError> {
    let report = capture_png(config)?;
    println!(
        "capture={} sha256={} adapter={} backend={} driver={} size={}x{} mode={:?} scene_time_seconds={} dragon_angle_radians={}",
        report.output_path.display(),
        report.sha256,
        report.adapter_name,
        report.backend,
        report.driver,
        report.width,
        report.height,
        report.mode,
        report.scene_time_seconds,
        report.dragon_angle_radians
    );
    Ok(())
}

fn run_evidence(config: &EvidenceConfig) -> Result<(), DemoError> {
    let report = capture_controlled_delta(config)?;
    println!(
        "evidence={} off_sha256={} radia_sha256={} repeat_sha256={} control={} lights={} roi={},{},{},{} max_delta={} mean_delta={:.6} changed_pixels={}",
        report.manifest_path.display(),
        report.off_capture.sha256,
        report.radia_capture.sha256,
        report.radia_repeat_capture.sha256,
        report.control_signature,
        report.light_count,
        report.receiver_roi.left,
        report.receiver_roi.top,
        report.receiver_roi.right,
        report.receiver_roi.bottom,
        report.maximum_channel_delta,
        report.mean_channel_delta,
        report.changed_pixels
    );
    Ok(())
}

fn run_window() -> Result<(), DemoError> {
    let event_loop = EventLoop::new().map_err(|error| DemoError::EventLoop(error.to_string()))?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut application = DemoApplication::default();
    event_loop
        .run_app(&mut application)
        .map_err(|error| DemoError::EventLoop(error.to_string()))?;
    if let Some(failure) = application.failure {
        return Err(DemoError::Render(RenderError::GpuValidation(failure)));
    }
    Ok(())
}

#[derive(Default)]
struct DemoApplication {
    running: Option<RunningApplication>,
    failure: Option<String>,
}

struct RunningApplication {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    renderer: RadiaRenderer,
    settings: RenderSettings,
    started_at: Instant,
    suspended: bool,
}

impl RunningApplication {
    fn new(event_loop: &ActiveEventLoop) -> Result<Self, DemoError> {
        let attributes = Window::default_attributes()
            .with_title("Radia - Jade Dragon Triad - mode Radia")
            .with_inner_size(LogicalSize::new(1280.0, 720.0));
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .map_err(|error| DemoError::Window(error.to_string()))?,
        );
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return Err(DemoError::Window(
                "new window reported a zero framebuffer extent".to_owned(),
            ));
        }
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });
        let surface = instance
            .create_surface(Arc::clone(&window))
            .map_err(|error| DemoError::Surface(error.to_string()))?;
        let renderer = block_on(RadiaRenderer::new(
            &instance,
            Some(&surface),
            None,
            size.width,
            size.height,
        ))?;
        let surface_config = renderer.surface_configuration(&surface, size.width, size.height)?;
        surface.configure(renderer.device(), &surface_config);
        let settings = default_render_settings(RadiaMode::Radia)?;
        Ok(Self {
            window,
            surface,
            surface_config,
            renderer,
            settings,
            started_at: Instant::now(),
            suspended: false,
        })
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), DemoError> {
        if width == 0 || height == 0 {
            self.suspended = true;
            return Ok(());
        }
        self.renderer.resize(width, height)?;
        self.surface_config = self
            .renderer
            .surface_configuration(&self.surface, width, height)?;
        self.surface
            .configure(self.renderer.device(), &self.surface_config);
        self.suspended = false;
        Ok(())
    }

    fn render(&mut self) -> Result<(), DemoError> {
        if self.suspended {
            return Ok(());
        }
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface
                    .configure(self.renderer.device(), &self.surface_config);
                return Ok(());
            }
            Err(wgpu::SurfaceError::Timeout) => return Ok(()),
            Err(wgpu::SurfaceError::OutOfMemory) => {
                return Err(DemoError::Surface("surface ran out of memory".to_owned()));
            }
            Err(wgpu::SurfaceError::Other) => {
                return Err(DemoError::Surface("surface acquisition failed".to_owned()));
            }
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("radia-surface-view"),
            ..Default::default()
        });
        apply_scene_motion(&mut self.settings, self.started_at.elapsed().as_secs_f32())?;
        self.renderer.render_to_view(&view, self.settings)?;
        frame.present();
        Ok(())
    }

    fn cycle_mode(&mut self) {
        self.settings.mode = self.settings.mode.next();
        self.renderer.reset_frame_sequence();
        self.window.set_title(&format!(
            "Radia - Jade Dragon Triad - mode {:?}",
            self.settings.mode
        ));
    }

    fn move_camera(&mut self, local_step: Vec3) -> Result<(), DemoError> {
        let error_scale = ErrorScale::new(100.0, 64).map_err(RenderError::from)?;
        let pose = self.settings.camera_to_world;
        let world_step = pose.transform_direction(local_step);
        self.settings.camera_to_world = radia_math::UnitDualQuat::from_rotation_translation(
            pose.rotation(),
            pose.translation() + world_step,
            error_scale,
        )
        .map_err(RenderError::from)?;
        self.renderer.reset_frame_sequence();
        Ok(())
    }
}

impl ApplicationHandler for DemoApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.running.is_some() {
            return;
        }
        match RunningApplication::new(event_loop) {
            Ok(running) => self.running = Some(running),
            Err(error) => self.stop_with_failure(event_loop, error.to_string()),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(running) = self.running.as_mut() else {
            return;
        };
        if running.window.id() != window_id {
            return;
        }
        let result = match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                Ok(())
            }
            WindowEvent::Resized(size) => running.resize(size.width, size.height),
            WindowEvent::RedrawRequested => running.render(),
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && !event.repeat =>
            {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Escape) => {
                        event_loop.exit();
                        Ok(())
                    }
                    PhysicalKey::Code(KeyCode::Space) => {
                        running.cycle_mode();
                        Ok(())
                    }
                    PhysicalKey::Code(KeyCode::KeyR) => {
                        running.started_at = Instant::now();
                        running.renderer.reset_frame_sequence();
                        Ok(())
                    }
                    PhysicalKey::Code(KeyCode::KeyW) => {
                        running.move_camera(Vec3::new(0.0, 0.0, -0.25))
                    }
                    PhysicalKey::Code(KeyCode::KeyS) => {
                        running.move_camera(Vec3::new(0.0, 0.0, 0.25))
                    }
                    PhysicalKey::Code(KeyCode::KeyA) => {
                        running.move_camera(Vec3::new(-0.25, 0.0, 0.0))
                    }
                    PhysicalKey::Code(KeyCode::KeyD) => {
                        running.move_camera(Vec3::new(0.25, 0.0, 0.0))
                    }
                    PhysicalKey::Code(KeyCode::KeyQ) => {
                        running.move_camera(Vec3::new(0.0, -0.25, 0.0))
                    }
                    PhysicalKey::Code(KeyCode::KeyE) => {
                        running.move_camera(Vec3::new(0.0, 0.25, 0.0))
                    }
                    _ => Ok(()),
                }
            }
            _ => Ok(()),
        };
        if let Err(error) = result {
            self.stop_with_failure(event_loop, error.to_string());
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(running) = &self.running
            && !running.suspended
        {
            running.window.request_redraw();
        }
    }
}

impl DemoApplication {
    fn stop_with_failure(&mut self, event_loop: &ActiveEventLoop, message: String) {
        eprintln!("Radia stopped: {message}");
        self.failure = Some(message);
        event_loop.exit();
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{Command, DemoError, parse_command, parse_mode};

    #[test]
    fn parses_headless_capture_contract() -> Result<(), DemoError> {
        let arguments = [
            "capture",
            "--width",
            "320",
            "--height",
            "180",
            "--mode",
            "ao",
            "--scene-time-seconds",
            "5.5",
            "--dragon-angle-degrees",
            "33",
            "--output",
            "out.png",
        ]
        .into_iter()
        .map(str::to_owned);
        let Command::Capture(config) = parse_command(arguments)? else {
            return Err(DemoError::Arguments("expected capture command".to_owned()));
        };
        assert_eq!(config.width, 320);
        assert_eq!(config.height, 180);
        assert_eq!(config.mode, radia_render::RadiaMode::AmbientOcclusion);
        assert!((config.scene_time_seconds - 5.5).abs() <= f32::EPSILON);
        let angle = config
            .dragon_angle_radians
            .ok_or_else(|| DemoError::Arguments("expected capture dragon angle".to_owned()))?;
        assert!((angle - 33_f32.to_radians()).abs() <= f32::EPSILON * 4.0);
        assert_eq!(config.output_path, PathBuf::from("out.png"));
        Ok(())
    }

    #[test]
    fn scene_motion_updates_dragons_and_all_lights_from_elapsed_time() -> Result<(), DemoError> {
        let start = radia_render::scene_motion(0.0)?;
        let later = radia_render::scene_motion(1.0)?;
        for index in 0..radia_render::DRAGON_COUNT {
            assert_ne!(
                start.dragons_to_world[index].translation(),
                later.dragons_to_world[index].translation()
            );
            assert!(
                start.dragons_to_world[index]
                    .rotation()
                    .orientation_dot(later.dragons_to_world[index].rotation())
                    .abs()
                    < 1.0
            );
        }
        for index in 0..radia_render::LIGHT_COUNT {
            assert_ne!(start.lights[index].position, later.lights[index].position);
        }
        Ok(())
    }

    #[test]
    fn parses_every_human_readable_buffer_mode() -> Result<(), DemoError> {
        for (name, expected) in [
            ("albedo", radia_render::RadiaMode::Albedo),
            ("normal", radia_render::RadiaMode::Normal),
            ("emissive", radia_render::RadiaMode::Emissive),
            ("depth", radia_render::RadiaMode::LinearDepth),
            ("ao", radia_render::RadiaMode::AmbientOcclusion),
        ] {
            assert_eq!(parse_mode(name)?, expected);
        }
        Ok(())
    }
}
