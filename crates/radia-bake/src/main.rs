use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("radia-bake: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut arguments = env::args_os().skip(1);
    let input = arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| usage("missing OBJ input"))?;
    let output = arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| usage("missing UDF output"))?;
    let resolution = arguments
        .next()
        .map(|value| {
            value
                .to_string_lossy()
                .parse::<u32>()
                .map_err(|_| usage("resolution must be an integer"))
        })
        .transpose()?
        .unwrap_or(128);
    if arguments.next().is_some() {
        return Err(usage("unexpected extra argument"));
    }

    let report = radia_bake::bake_obj_file(&input, &output, resolution)?;
    println!(
        "baked {} vertices, {} triangles into {}x{}x{} UDF; source bounds {:?}..{:?}; baked bounds {:?}..{:?}",
        report.vertex_count,
        report.triangle_count,
        report.resolution[0],
        report.resolution[1],
        report.resolution[2],
        report.source_min,
        report.source_max,
        report.volume_min,
        report.volume_max
    );
    Ok(())
}

fn usage(message: &str) -> String {
    format!("{message}; usage: radia-bake <input.obj> <output.rduf> [resolution]")
}
