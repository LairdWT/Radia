use std::fs;

use radia_render::{CaptureConfig, RadiaMode, capture_png};

#[test]
#[ignore = "requires a Vulkan adapter and driver"]
fn captures_identical_fixed_state_pngs() -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::env::temp_dir().join(format!("radia-gpu-test-{}", std::process::id()));
    fs::create_dir_all(&directory)?;
    let first = capture_png(&CaptureConfig {
        width: 64,
        height: 64,
        samples: 4,
        mode: RadiaMode::Radia,
        output_path: directory.join("first.png"),
    })?;
    let second = capture_png(&CaptureConfig {
        width: 64,
        height: 64,
        samples: 4,
        mode: RadiaMode::Radia,
        output_path: directory.join("second.png"),
    })?;
    assert_eq!(first.sha256, second.sha256);
    fs::remove_dir_all(directory)?;
    Ok(())
}
