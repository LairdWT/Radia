# Jade dragon RADIA upgrade

## Context

Radia currently renders an analytic SDF courtyard. The owner requested a
recognizable jade dragon illuminated by three colored sources with shadows and
RADIA indirect light. The implementation must preserve the accepted
quaternion-first, matrix-free contracts and exact dependency set.

## Evidence

- `docs/adr/c69013f1-analytic-radia-mvp-boundary.md:46` defers mesh SDF baking and
  must be superseded before this work lands.
- `crates/radia-render/src/shaders/radia.wgsl:29` owns camera tracing, material
  evaluation, one-bounce sampling, and temporal accumulation in one pass.
- `crates/radia-render/src/renderer.rs:126` owns the explicit WGPU bind-group and
  host-uniform contracts.
- Stanford's repository permits attributed research use and free
  redistribution, but excludes commercial use without permission:
  https://graphics.stanford.edu/data/3Dscanrep/
- McGuire's archive identifies its Chinese Dragon OBJ as a Stanford Scan and
  documents its conversion provenance:
  https://casual-effects.com/g3d/data10/research/model/dragon/info.js

## Steps

1. [decision gate] Accept and mechanically supersede the analytic-only ADR.
2. [source gate] Download the named archive into `Temp/`, verify a pinned
   SHA-256, and record archive/member metadata plus license boundaries.
3. [CPU gate] Add a dependency-free deterministic OBJ-to-UDF baker with BVH
   nearest-triangle queries, numeric guards, golden metadata, and field tests.
4. [GPU gate] Bind the fixed volume as a read-only storage buffer and add
   conservative sampling, normals, dragon material identity, and trace tests.
5. [lighting gate] Add three finite colored emitters, direct visibility shadows,
   bounded jade shading, and one-bounce RADIA contribution in linear light.
6. [product gate] Update controls, README demo instructions, asset attribution,
   limitations, and a new headless evidence command.
7. [release gate] Run format, clippy, workspace tests, matrix ban, AEP gates,
   Vulkan captures on both adapters, inspect the image, build release, and
   launch the upgraded windowed demo.

## Files

- `docs/adr/`, `docs/planning/jade-dragon.md`
- `assets/stanford-dragon/`
- `crates/radia-bake/`
- `crates/radia-render/src/renderer.rs`
- `crates/radia-render/src/sdf.rs`
- `crates/radia-render/src/shaders/radia.wgsl`
- `crates/radia-demo/src/main.rs`
- `README.md`, `docs/hackathon/demo-script.md`

## Verification

- `cargo fmt --all -- --check` exits 0.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` exits 0.
- `cargo test --workspace --all-features` exits 0.
- `powershell -File scripts/check-matrix-ban.ps1` exits 0.
- `agent-code-skills adr check --target C:\Radia` exits 0.
- `agent-code-skills pins --check --target C:\Radia` exits 0.
- `agent-code-skills doctor --target C:\Radia` exits 0.
- Fixed-state Vulkan captures validate on the RTX 4070 and AMD 610M.
- A release window visibly shows the dragon, three colored sources, occlusion
  shadows, and a distinct RADIA on/off response.

## Non-goals

- Runtime glTF/OBJ import, mesh editing, clipmaps, skinning, subsurface
  scattering, multiple indirect bounces, or general-purpose signed volume CSG.

## Risks

- Stanford's asset terms are not Apache-2.0; isolate and label the artifact.
- Source holes prevent exact sign classification; use and name an unsigned
  distance field.
- Voxel resolution can erase thin details; validate silhouette before release.
- Large storage buffers can expose adapter-limit drift; check limits before
  creating the resource.
