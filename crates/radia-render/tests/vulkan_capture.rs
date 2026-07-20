use std::{fs, io::BufReader, path::Path};

use radia_render::{CaptureConfig, RadiaMode, capture_png};

#[test]
#[ignore = "requires a Vulkan adapter and driver"]
fn captures_identical_fixed_state_pngs() -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::env::temp_dir().join(format!("radia-gpu-test-{}", std::process::id()));
    fs::create_dir_all(&directory)?;
    let first = capture_png(&CaptureConfig {
        width: 64,
        height: 64,
        mode: RadiaMode::Radia,
        scene_time_seconds: 5.0,
        dragon_angle_radians: Some(33_f32.to_radians()),
        output_path: directory.join("first.png"),
    })?;
    let second = capture_png(&CaptureConfig {
        width: 64,
        height: 64,
        mode: RadiaMode::Radia,
        scene_time_seconds: 5.0,
        dragon_angle_radians: Some(33_f32.to_radians()),
        output_path: directory.join("second.png"),
    })?;
    assert_eq!(first.sha256, second.sha256);
    for mode in [
        RadiaMode::HitState,
        RadiaMode::LinearDepth,
        RadiaMode::AmbientOcclusion,
        RadiaMode::Albedo,
        RadiaMode::Normal,
        RadiaMode::Emissive,
    ] {
        let report = capture_png(&CaptureConfig {
            width: 64,
            height: 64,
            mode,
            scene_time_seconds: 5.0,
            dragon_angle_radians: Some(33_f32.to_radians()),
            output_path: directory.join(format!("{mode:?}.png")),
        })?;
        assert!(fs::metadata(report.output_path)?.len() > 128);
    }
    for angle_degrees in [170.0_f32, 190.0_f32] {
        let report = capture_png(&CaptureConfig {
            width: 320,
            height: 180,
            mode: RadiaMode::HitState,
            scene_time_seconds: 0.0,
            dragon_angle_radians: Some(angle_degrees.to_radians()),
            output_path: directory.join(format!("HitState-{angle_degrees}.png")),
        })?;
        let pixels = decode_rgba8(&report.output_path)?;
        let indeterminate_pixels = pixels
            .chunks_exact(4)
            .filter(|pixel| pixel[0] > 180 && pixel[1] < 32 && pixel[2] > 180)
            .count();
        assert_eq!(
            indeterminate_pixels, 0,
            "rotated UDF bounds must not exhaust the primary trace at {angle_degrees} degrees"
        );
    }
    fs::remove_dir_all(directory)?;
    Ok(())
}

fn decode_rgba8(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let decoder = png::Decoder::new(BufReader::new(fs::File::open(path)?));
    let mut reader = decoder.read_info()?;
    let buffer_size = reader
        .output_buffer_size()
        .ok_or_else(|| std::io::Error::other("PNG output size is unknown"))?;
    let mut pixels = vec![0_u8; buffer_size];
    let info = reader.next_frame(&mut pixels)?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return Err(std::io::Error::other("expected RGBA8 capture").into());
    }
    pixels.truncate(info.buffer_size());
    Ok(pixels)
}
