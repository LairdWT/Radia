use std::fs;
use std::io::BufReader;
use std::path::PathBuf;

use crate::dragon::DRAGON_FIELD_SHA256;
use crate::renderer::{RadiaMode, RenderSettings, default_render_settings};
use crate::sha256::digest_hex;
use crate::{CaptureConfig, CaptureReport, RenderError, capture_png};

#[derive(Clone, Debug, PartialEq)]
pub struct EvidenceConfig {
    pub width: u32,
    pub height: u32,
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
    pub radia_repeat_capture: CaptureReport,
    pub manifest_path: PathBuf,
    pub receiver_roi: ReceiverRoi,
    pub maximum_channel_delta: u8,
    pub mean_channel_delta: f64,
    pub changed_pixels: u64,
    pub control_signature: String,
    pub light_count: u32,
}

struct ManifestData<'a> {
    config: &'a EvidenceConfig,
    off_capture: &'a CaptureReport,
    radia_capture: &'a CaptureReport,
    radia_repeat_capture: &'a CaptureReport,
    control_signature: &'a str,
    receiver_roi: ReceiverRoi,
    maximum_channel_delta: u8,
    mean_channel_delta: f64,
    changed_pixels: u64,
}

/// Captures Off and Radia modes and enforces the controlled receiver delta.
///
/// # Errors
///
/// Returns capture, decode, control-mismatch, or delta failures.
pub fn capture_controlled_delta(config: &EvidenceConfig) -> Result<EvidenceReport, RenderError> {
    if config.width < 16 || config.height < 16 {
        return Err(RenderError::InvalidConfig(
            "evidence extent must be at least 16x16".to_owned(),
        ));
    }
    let (off_capture, radia_capture, radia_repeat_capture) = capture_set(config)?;
    if off_capture.adapter_name != radia_capture.adapter_name
        || off_capture.adapter_name != radia_repeat_capture.adapter_name
        || off_capture.backend != radia_capture.backend
        || off_capture.backend != radia_repeat_capture.backend
        || off_capture.driver != radia_capture.driver
        || off_capture.driver != radia_repeat_capture.driver
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
    let (off_width, off_height, off_pixels) = decode_rgba8(&off_capture.output_path)?;
    let (radia_width, radia_height, radia_pixels) = decode_rgba8(&radia_capture.output_path)?;
    let (repeat_width, repeat_height, repeat_pixels) =
        decode_rgba8(&radia_repeat_capture.output_path)?;
    if off_width != config.width
        || off_height != config.height
        || radia_width != config.width
        || radia_height != config.height
        || repeat_width != config.width
        || repeat_height != config.height
        || off_pixels.len() != radia_pixels.len()
        || radia_pixels != repeat_pixels
        || radia_capture.sha256 != radia_repeat_capture.sha256
    {
        return Err(RenderError::InvalidConfig(
            "decoded evidence extent, byte length, or fixed-state RADIA output differs".to_owned(),
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
        radia_repeat_capture: &radia_repeat_capture,
        control_signature: &off_signature,
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
        radia_repeat_capture,
        manifest_path,
        receiver_roi,
        maximum_channel_delta,
        mean_channel_delta,
        changed_pixels,
        control_signature: off_signature,
        light_count: 3,
    })
}

fn capture_set(
    config: &EvidenceConfig,
) -> Result<(CaptureReport, CaptureReport, CaptureReport), RenderError> {
    fs::create_dir_all(&config.output_directory)?;
    let capture = |name: &str, mode| {
        capture_png(&CaptureConfig {
            width: config.width,
            height: config.height,
            mode,
            scene_time_seconds: 0.0,
            dragon_angle_radians: None,
            output_path: config.output_directory.join(name),
        })
    };
    Ok((
        capture("radia-off.png", RadiaMode::Off)?,
        capture("radia-on.png", RadiaMode::Radia)?,
        capture("radia-on-repeat.png", RadiaMode::Radia)?,
    ))
}

fn write_manifest(data: &ManifestData<'_>) -> Result<PathBuf, RenderError> {
    let manifest_path = data.config.output_directory.join("visual-evidence.tsv");
    let settings = default_render_settings(RadiaMode::Radia)?;
    let lights = settings
        .lights
        .iter()
        .enumerate()
        .map(|(index, light)| {
            format!(
                "{index}=({:.6},{:.6},{:.6};r={:.6})",
                light.position.x, light.position.y, light.position.z, light.radius
            )
        })
        .collect::<Vec<_>>()
        .join(";");
    let manifest = format!(
        "field\tvalue\ncontract\tadr:spatially-phased-and-edge-resolved-deferred-radia\nscene_presentation_contract\tadr:conservatively-disjoint-dragon-field-bounds\nudf_domain_contract\tadr:conservative-sampled-udf-domain-extension\nbackend\t{}\nadapter\t{}\ndriver\t{}\nsize\t{}x{}\nscene_time_seconds\t0\ncamera_vertical_fov_degrees\t65\ncamera_position\t0,3.0,2.3282032\ncamera_pitch_degrees\t-30\ndragon_origin_radius\t2.0\ndragon_materials\tcyan:r=0.2:m=1.0;magenta:r=0.65:m=0.0;yellow:r=0.9:m=0.0\ndirect_brdf\tggx-smith-schlick\nenvironment_fill\tanalytic-unoccluded-sky-ground\ngi_method\tscreen-space-irradiance-v3\ngather_taps\t32\nphase_tile\t4x4\nresolve_kernel\t5x5-edge-aware\ndragon_field_sha256\t{}\ndragon_count\t3\nlight_count\t3\nlights\t{}\noff_sha256\t{}\nradia_sha256\t{}\nradia_repeat_sha256\t{}\nfixed_state_hashes_match\ttrue\ncontrol_signature\t{}\nreceiver_roi\t{},{},{},{}\nmaximum_channel_delta\t{}\nmean_channel_delta\t{:.6}\nchanged_pixels\t{}\nthreshold\t4\n",
        data.off_capture.backend,
        data.off_capture.adapter_name,
        data.off_capture.driver,
        data.config.width,
        data.config.height,
        DRAGON_FIELD_SHA256,
        lights,
        data.off_capture.sha256,
        data.radia_capture.sha256,
        data.radia_repeat_capture.sha256,
        data.control_signature,
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
    let mut values = Vec::with_capacity(8 + settings.lights.len() * 8 + 4);
    values.extend_from_slice(&camera);
    for light in settings.lights {
        values.extend_from_slice(&[
            light.position.x,
            light.position.y,
            light.position.z,
            light.radius,
            light.color.x,
            light.color.y,
            light.color.z,
            light.intensity,
        ]);
    }
    values.extend_from_slice(&[
        settings.vertical_fov,
        settings.near,
        settings.maximum_distance,
        settings.hit_guard,
    ]);
    let mut bytes =
        Vec::with_capacity((values.len() + settings.dragons_to_world.len() * 8) * 4 + 4);
    for value in values {
        bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    for dragon in settings.dragons_to_world {
        for value in dragon.to_gpu_xyzw() {
            bytes.extend_from_slice(&value.to_bits().to_le_bytes());
        }
    }
    bytes.extend_from_slice(&settings.maximum_steps.to_le_bytes());
    digest_hex(&bytes)
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
