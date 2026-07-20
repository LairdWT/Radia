//! Vulkan-only WGPU renderer for Radia.

mod capture;
mod dragon;
mod error;
mod evidence;
mod executor;
mod graph;
mod renderer;
mod sdf;
mod sha256;

pub use capture::{CaptureConfig, CaptureReport, capture_png};
pub use error::RenderError;
pub use evidence::{EvidenceConfig, EvidenceReport, ReceiverRoi, capture_controlled_delta};
pub use executor::block_on;
pub use renderer::{
    DRAGON_COUNT, LIGHT_COUNT, RadiaMode, RadiaRenderer, RenderSettings, SceneLight, SceneMotion,
    apply_scene_motion, default_render_settings, dragon_triad_pose, scene_cluster_phase,
    scene_motion,
};
pub use sdf::{
    CpuSdfScene, CpuTraceConfig, CpuTraceResult, SdfMaterial, SdfSample, TraceTermination,
};
