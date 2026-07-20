use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use crate::renderer::{
    RadiaMode, RadiaRenderer, apply_scene_motion, default_render_settings, dragon_triad_pose,
    scene_cluster_phase,
};
use crate::sha256::digest_hex;
use crate::{RenderError, block_on};

#[derive(Clone, Debug, PartialEq)]
pub struct CaptureConfig {
    pub width: u32,
    pub height: u32,
    pub mode: RadiaMode,
    pub scene_time_seconds: f32,
    pub dragon_angle_radians: Option<f32>,
    pub output_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CaptureReport {
    pub output_path: PathBuf,
    pub sha256: String,
    pub adapter_name: String,
    pub backend: String,
    pub driver: String,
    pub width: u32,
    pub height: u32,
    pub mode: RadiaMode,
    pub scene_time_seconds: f32,
    pub dragon_angle_radians: f32,
}

/// Renders a deterministic offscreen image and writes an RGBA PNG.
///
/// # Errors
///
/// Returns config, Vulkan/WGPU, readback, I/O, or PNG encoding failures.
pub fn capture_png(config: &CaptureConfig) -> Result<CaptureReport, RenderError> {
    validate_capture_config(config)?;
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..Default::default()
    });
    let mut renderer = block_on(RadiaRenderer::new(
        &instance,
        None,
        Some(wgpu::TextureFormat::Rgba8UnormSrgb),
        config.width,
        config.height,
    ))?;
    let mut settings = default_render_settings(config.mode)?;
    apply_scene_motion(&mut settings, config.scene_time_seconds)?;
    if let Some(angle_radians) = config.dragon_angle_radians {
        settings.dragons_to_world = dragon_triad_pose(angle_radians)?;
    }
    let output_texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
        label: Some("radia-headless-output"),
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: renderer.output_format(),
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("radia-headless-output-view"),
        ..Default::default()
    });
    renderer.render_to_view(&output_view, settings)?;
    let rgba = read_rgba8(&renderer, &output_texture, config.width, config.height)?;
    let png_bytes = encode_png(&rgba, config.width, config.height)?;
    if let Some(parent) = config.output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&config.output_path, &png_bytes)?;
    let adapter = renderer.adapter_info();
    Ok(CaptureReport {
        output_path: config.output_path.clone(),
        sha256: digest_hex(&png_bytes),
        adapter_name: adapter.name.clone(),
        backend: format!("{:?}", adapter.backend),
        driver: adapter.driver.clone(),
        width: config.width,
        height: config.height,
        mode: config.mode,
        scene_time_seconds: config.scene_time_seconds,
        dragon_angle_radians: config
            .dragon_angle_radians
            .unwrap_or(scene_cluster_phase(config.scene_time_seconds)?),
    })
}

fn validate_capture_config(config: &CaptureConfig) -> Result<(), RenderError> {
    if config.width == 0 || config.height == 0 {
        return Err(RenderError::InvalidConfig(
            "capture width and height must be positive".to_owned(),
        ));
    }
    if config.output_path == Path::new("") {
        return Err(RenderError::InvalidConfig(
            "capture output path must not be empty".to_owned(),
        ));
    }
    if config
        .dragon_angle_radians
        .is_some_and(|angle| !angle.is_finite())
    {
        return Err(RenderError::InvalidConfig(
            "capture dragon angle must be finite".to_owned(),
        ));
    }
    if !config.scene_time_seconds.is_finite() {
        return Err(RenderError::InvalidConfig(
            "capture scene time must be finite".to_owned(),
        ));
    }
    Ok(())
}

fn read_rgba8(
    renderer: &RadiaRenderer,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, RenderError> {
    let row_bytes = width.checked_mul(4).ok_or_else(|| {
        RenderError::InvalidConfig("capture row byte count overflowed".to_owned())
    })?;
    let padded_row_bytes = row_bytes
        .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        .checked_mul(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        .ok_or_else(|| RenderError::InvalidConfig("padded row byte count overflowed".to_owned()))?;
    let buffer_size = u64::from(padded_row_bytes)
        .checked_mul(u64::from(height))
        .ok_or_else(|| RenderError::InvalidConfig("readback buffer size overflowed".to_owned()))?;
    let readback = renderer.device().create_buffer(&wgpu::BufferDescriptor {
        label: Some("radia-headless-readback"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = renderer
        .device()
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("radia-headless-readback-encoder"),
        });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_row_bytes),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    renderer.queue().submit(Some(encoder.finish()));

    let slice = readback.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        drop(sender.send(result));
    });
    renderer
        .device()
        .poll(wgpu::PollType::wait())
        .map_err(|error| RenderError::DevicePoll(error.to_string()))?;
    receiver
        .recv()
        .map_err(|error| RenderError::BufferMap(error.to_string()))?
        .map_err(|error| RenderError::BufferMap(error.to_string()))?;

    let mapped = slice.get_mapped_range();
    let tight_length = usize::try_from(u64::from(row_bytes) * u64::from(height))
        .map_err(|_| RenderError::InvalidConfig("tight readback size exceeded usize".to_owned()))?;
    let mut tight = vec![0_u8; tight_length];
    let source_stride = usize::try_from(padded_row_bytes)
        .map_err(|_| RenderError::InvalidConfig("source stride exceeded usize".to_owned()))?;
    let destination_stride = usize::try_from(row_bytes)
        .map_err(|_| RenderError::InvalidConfig("destination stride exceeded usize".to_owned()))?;
    for row in 0..usize::try_from(height)
        .map_err(|_| RenderError::InvalidConfig("height exceeded usize".to_owned()))?
    {
        let source_start = row * source_stride;
        let destination_start = row * destination_stride;
        tight[destination_start..destination_start + destination_stride]
            .copy_from_slice(&mapped[source_start..source_start + destination_stride]);
    }
    drop(mapped);
    readback.unmap();
    Ok(tight)
}

fn encode_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, RenderError> {
    let mut encoded = Vec::new();
    {
        let mut png_encoder = png::Encoder::new(&mut encoded, width, height);
        png_encoder.set_color(png::ColorType::Rgba);
        png_encoder.set_depth(png::BitDepth::Eight);
        let mut writer = png_encoder
            .write_header()
            .map_err(|error| RenderError::PngEncoding(error.to_string()))?;
        writer
            .write_image_data(rgba)
            .map_err(|error| RenderError::PngEncoding(error.to_string()))?;
    }
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{CaptureConfig, capture_png};
    use crate::RadiaMode;

    #[test]
    fn capture_config_rejects_zero_domains_before_gpu_work() {
        let config = CaptureConfig {
            width: 0,
            height: 64,
            mode: RadiaMode::Off,
            scene_time_seconds: 0.0,
            dragon_angle_radians: None,
            output_path: PathBuf::from("capture.png"),
        };
        assert!(capture_png(&config).is_err());
        let non_finite = CaptureConfig {
            width: 64,
            height: 64,
            mode: RadiaMode::Off,
            scene_time_seconds: 0.0,
            dragon_angle_radians: Some(f32::NAN),
            output_path: PathBuf::from("capture.png"),
        };
        assert!(capture_png(&non_finite).is_err());
        let non_finite_time = CaptureConfig {
            width: 64,
            height: 64,
            mode: RadiaMode::Off,
            scene_time_seconds: f32::INFINITY,
            dragon_angle_radians: None,
            output_path: PathBuf::from("capture.png"),
        };
        assert!(capture_png(&non_finite_time).is_err());
    }
}
