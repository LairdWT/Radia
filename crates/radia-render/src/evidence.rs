use std::fs;
use std::io::BufReader;
use std::path::PathBuf;

use radia_math::ReverseZPerspective;

use crate::renderer::{RadiaMode, RenderSettings, default_render_settings};
use crate::sha256::digest_hex;
use crate::{CaptureConfig, CaptureReport, RenderError, capture_png};

#[derive(Clone, Debug, PartialEq)]
pub struct EvidenceConfig {
    pub width: u32,
    pub height: u32,
    pub samples: u32,
    pub output_directory: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReceiverRoi {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EvidenceReport {
    pub off_capture: CaptureReport,
    pub radia_capture: CaptureReport,
    pub manifest_path: PathBuf,
    pub receiver_roi: ReceiverRoi,
    pub maximum_channel_delta: u8,
    pub mean_channel_delta: f64,
    pub changed_pixels: u64,
    pub control_signature: String,
    pub emitter_offscreen: bool,
}

struct ManifestData<'a> {
    config: &'a EvidenceConfig,
    off_capture: &'a CaptureReport,
    radia_capture: &'a CaptureReport,
    control_signature: &'a str,
    emitter_offscreen: bool,
    receiver_roi: ReceiverRoi,
    maximum_channel_delta: u8,
    mean_channel_delta: f64,
    changed_pixels: u64,
}

/// Captures Off and Radia modes and enforces the controlled receiver delta.
///
/// # Errors
///
/// Returns capture, decode, control-mismatch, offscreen, or delta failures.
pub fn capture_controlled_delta(config: &EvidenceConfig) -> Result<EvidenceReport, RenderError> {
    if config.width < 16 || config.height < 16 || config.samples == 0 {
        return Err(RenderError::InvalidConfig(
            "evidence extent must be at least 16x16 and samples must be positive".to_owned(),
        ));
    }
    fs::create_dir_all(&config.output_directory)?;
    let off_path = config.output_directory.join("radia-off.png");
    let radia_path = config.output_directory.join("radia-on.png");
    let off_capture = capture_png(&CaptureConfig {
        width: config.width,
        height: config.height,
        samples: config.samples,
        mode: RadiaMode::Off,
        output_path: off_path,
    })?;
    let radia_capture = capture_png(&CaptureConfig {
        width: config.width,
        height: config.height,
        samples: config.samples,
        mode: RadiaMode::Radia,
        output_path: radia_path,
    })?;
    if off_capture.adapter_name != radia_capture.adapter_name
        || off_capture.backend != radia_capture.backend
        || off_capture.driver != radia_capture.driver
    {
        return Err(RenderError::InvalidConfig(
            "controlled captures used different GPU adapters or drivers".to_owned(),
        ));
    }

    let off_settings = default_render_settings(RadiaMode::Off)?;
    let radia_settings = default_render_settings(RadiaMode::Radia)?;
    let off_signature = control_signature(off_settings);
    let radia_signature = control_signature(radia_settings);
    if off_signature != radia_signature {
        return Err(RenderError::InvalidConfig(
            "receiver, direct-light, depth, or emitter controls changed".to_owned(),
        ));
    }
    let emitter_offscreen = emitter_is_offscreen(off_settings, config.width, config.height)?;
    if !emitter_offscreen {
        return Err(RenderError::InvalidConfig(
            "emitter intersects the camera frustum".to_owned(),
        ));
    }

    let (off_width, off_height, off_pixels) = decode_rgba8(&off_capture.output_path)?;
    let (radia_width, radia_height, radia_pixels) = decode_rgba8(&radia_capture.output_path)?;
    if off_width != config.width
        || off_height != config.height
        || radia_width != config.width
        || radia_height != config.height
        || off_pixels.len() != radia_pixels.len()
    {
        return Err(RenderError::InvalidConfig(
            "decoded evidence extent or byte length differs".to_owned(),
        ));
    }
    let receiver_roi = ReceiverRoi {
        left: config.width / 8,
        top: config.height / 2,
        right: config.width - config.width / 8,
        bottom: config.height,
    };
    let (maximum_channel_delta, mean_channel_delta, changed_pixels) =
        compare_receiver_roi(&off_pixels, &radia_pixels, config.width, receiver_roi)?;
    let manifest_path = write_manifest(&ManifestData {
        config,
        off_capture: &off_capture,
        radia_capture: &radia_capture,
        control_signature: &off_signature,
        emitter_offscreen,
        receiver_roi,
        maximum_channel_delta,
        mean_channel_delta,
        changed_pixels,
    })?;
    if maximum_channel_delta < 4 {
        return Err(RenderError::InvalidConfig(format!(
            "receiver ROI maximum channel delta {maximum_channel_delta} is below 4/255"
        )));
    }
    Ok(EvidenceReport {
        off_capture,
        radia_capture,
        manifest_path,
        receiver_roi,
        maximum_channel_delta,
        mean_channel_delta,
        changed_pixels,
        control_signature: off_signature,
        emitter_offscreen,
    })
}

fn write_manifest(data: &ManifestData<'_>) -> Result<PathBuf, RenderError> {
    let manifest_path = data.config.output_directory.join("visual-evidence.tsv");
    let manifest = format!(
        "field\tvalue\ncontract\tadr:deterministic-visual-evidence-contract\nbackend\t{}\nadapter\t{}\ndriver\t{}\nsize\t{}x{}\nsamples\t{}\nsequence\thammersley-cp-v1-1024\noff_sha256\t{}\nradia_sha256\t{}\ncontrol_signature\t{}\nemitter_offscreen\t{}\nreceiver_roi\t{},{},{},{}\nmaximum_channel_delta\t{}\nmean_channel_delta\t{:.6}\nchanged_pixels\t{}\nthreshold\t4\n",
        data.off_capture.backend,
        data.off_capture.adapter_name,
        data.off_capture.driver,
        data.config.width,
        data.config.height,
        data.config.samples,
        data.off_capture.sha256,
        data.radia_capture.sha256,
        data.control_signature,
        data.emitter_offscreen,
        data.receiver_roi.left,
        data.receiver_roi.top,
        data.receiver_roi.right,
        data.receiver_roi.bottom,
        data.maximum_channel_delta,
        data.mean_channel_delta,
        data.changed_pixels,
    );
    fs::write(&manifest_path, manifest)?;
    Ok(manifest_path)
}

fn control_signature(settings: RenderSettings) -> String {
    let camera = settings.camera_to_world.to_gpu_xyzw();
    let values = [
        camera[0],
        camera[1],
        camera[2],
        camera[3],
        camera[4],
        camera[5],
        camera[6],
        camera[7],
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
    let mut bytes = Vec::with_capacity(values.len() * 4 + 4);
    for value in values {
        bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    bytes.extend_from_slice(&settings.maximum_steps.to_le_bytes());
    digest_hex(&bytes)
}

fn emitter_is_offscreen(
    settings: RenderSettings,
    width: u32,
    height: u32,
) -> Result<bool, RenderError> {
    let aspect = f32::from(u16::try_from(width).map_err(|_| {
        RenderError::InvalidConfig("evidence width exceeds exact projection range".to_owned())
    })?) / f32::from(u16::try_from(height).map_err(|_| {
        RenderError::InvalidConfig("evidence height exceeds exact projection range".to_owned())
    })?);
    let projection = ReverseZPerspective::new(std::f32::consts::FRAC_PI_3, aspect, 0.1)?;
    let screen = projection.project_world(
        settings.camera_to_world,
        settings.emitter_position,
        width,
        height,
    )?;
    let camera_point = settings
        .camera_to_world
        .inverse()
        .transform_point(settings.emitter_position);
    if camera_point.z >= 0.0 {
        return Ok(true);
    }
    let height_f32 = f32::from(u16::try_from(height).map_err(|_| {
        RenderError::InvalidConfig("evidence height exceeds exact projection range".to_owned())
    })?);
    let projected_radius = settings.emitter_radius * height_f32
        / (2.0 * (std::f32::consts::FRAC_PI_6).tan() * -camera_point.z);
    let width_f32 = f32::from(u16::try_from(width).map_err(|_| {
        RenderError::InvalidConfig("evidence width exceeds exact projection range".to_owned())
    })?);
    Ok(screen.position.x - projected_radius >= width_f32
        || screen.position.x + projected_radius <= 0.0
        || screen.position.y - projected_radius >= height_f32
        || screen.position.y + projected_radius <= 0.0)
}

fn decode_rgba8(path: &PathBuf) -> Result<(u32, u32, Vec<u8>), RenderError> {
    let file = fs::File::open(path)?;
    let decoder = png::Decoder::new(BufReader::new(file));
    let mut reader = decoder
        .read_info()
        .map_err(|error| RenderError::PngEncoding(error.to_string()))?;
    let buffer_size = reader.output_buffer_size().ok_or_else(|| {
        RenderError::PngEncoding("decoded PNG output buffer size is unknown".to_owned())
    })?;
    let mut pixels = vec![0_u8; buffer_size];
    let info = reader
        .next_frame(&mut pixels)
        .map_err(|error| RenderError::PngEncoding(error.to_string()))?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return Err(RenderError::PngEncoding(format!(
            "expected RGBA8, got {:?} {:?}",
            info.color_type, info.bit_depth
        )));
    }
    pixels.truncate(info.buffer_size());
    Ok((info.width, info.height, pixels))
}

fn compare_receiver_roi(
    off: &[u8],
    radia: &[u8],
    width: u32,
    roi: ReceiverRoi,
) -> Result<(u8, f64, u64), RenderError> {
    let width_usize = usize::try_from(width)
        .map_err(|_| RenderError::InvalidConfig("ROI width exceeded usize".to_owned()))?;
    let mut maximum = 0_u8;
    let mut total = 0.0_f64;
    let mut channel_count = 0.0_f64;
    let mut changed_pixels = 0_u64;
    for row in roi.top..roi.bottom {
        for column in roi.left..roi.right {
            let pixel_index = usize::try_from(row)
                .and_then(|row_value| {
                    usize::try_from(column)
                        .map(|column_value| row_value * width_usize + column_value)
                })
                .map_err(|_| RenderError::InvalidConfig("ROI index exceeded usize".to_owned()))?;
            let byte_index = pixel_index * 4;
            let mut pixel_maximum = 0_u8;
            for channel in 0..3 {
                let delta = off[byte_index + channel].abs_diff(radia[byte_index + channel]);
                maximum = maximum.max(delta);
                pixel_maximum = pixel_maximum.max(delta);
                total += f64::from(delta);
                channel_count += 1.0;
            }
            if pixel_maximum >= 4 {
                changed_pixels += 1;
            }
        }
    }
    let mean = if channel_count == 0.0 {
        0.0
    } else {
        total / channel_count
    };
    Ok((maximum, mean, changed_pixels))
}

#[cfg(test)]
mod tests {
    use super::{ReceiverRoi, compare_receiver_roi, control_signature};
    use crate::{RadiaMode, default_render_settings};

    #[test]
    fn mode_does_not_change_control_signature() {
        let off = default_render_settings(RadiaMode::Off).expect("frozen scene is valid");
        let radia = default_render_settings(RadiaMode::Radia).expect("frozen scene is valid");
        assert_eq!(control_signature(off), control_signature(radia));
    }

    #[test]
    fn receiver_comparison_ignores_alpha_and_reports_threshold() {
        let off = [10_u8, 20, 30, 0, 40, 50, 60, 0];
        let radia = [14_u8, 21, 29, 255, 40, 50, 60, 255];
        let result = compare_receiver_roi(
            &off,
            &radia,
            2,
            ReceiverRoi {
                left: 0,
                top: 0,
                right: 2,
                bottom: 1,
            },
        )
        .expect("ROI is valid");
        assert_eq!(result.0, 4);
        assert_eq!(result.2, 1);
    }
}
