# Replace stochastic RADIA with deterministic deferred GI and rotate the dragon

## Context

The owner reports visible path-tracing grain and artifacts and requires a slowly
rotating dragon. Replace the stochastic secondary-bounce path with a stable,
current-frame deferred screen-space irradiance gather. Keep direct shadows,
dual-quaternion rigid transforms, analytic projection, and matrix-free code.

User decisions (2026-07-18): no path tracing or Monte Carlo accumulation;
mirror Legaia's deferred separation without copying private code; rotate the
dragon slowly as a rigid pose.

## Evidence

- `crates/radia-render/src/shaders/deferred.wgsl:38` selects one
  cosine-weighted bounce direction per pixel and frame.
- `crates/radia-render/src/shaders/temporal.wgsl:38` averages that stochastic
  sample into history, causing visible convergence grain.
- `crates/radia-render/src/shaders/scene.wgsl:84-89` hard-codes a static dragon
  quaternion and translation.
- `crates/radia-render/src/graph.rs:68-88` owns a separate temporal history pass
  after deferred lighting.
- `crates/radia-demo/src/main.rs:223-230` stores no monotonic animation epoch;
  `render` forwards an unchanged pose at `:313`.
- Owner-provided private design evidence separates indirect lighting into a
  named graph pass. It remains read-only evidence under the clean-room
  boundary; Radia carries no private paths or implementation details.

## Steps

### 1. Freeze replacement contracts

Accept one ADR replacing stochastic transport/history with deterministic
current-frame deferred irradiance, and one ADR fixing a rigid +Y turntable at
0.12 radians per second with a fixed headless pose.

### 2. Make dragon pose explicit

Add `dragon_to_world: UnitDualQuat` to `RenderSettings`, expand the explicit
uniform encoder, and replace shader constants with GPU `xyzw` real/dual lanes.
Derive windowed orientation analytically from monotonic elapsed time; do not
integrate quaternion products frame by frame.

### 3. Remove stochastic transport and history

Delete Hammersley/hash/cosine sampling and the secondary scene trace. Evaluate
indirect diffuse light with a fixed symmetric screen-space G-buffer gather over
the current frame. Reject missing depth, back-facing pairs, and invalid distance
domains. Remove temporal targets, bind groups, pipeline, shader, and pass.

### 4. Remove sample-count semantics

Render each headless capture once. Remove `--samples` and sample/sequence fields
from current capture reports and manifests. Continue producing two identical
fixed-state RADIA captures for determinism.

### 5. Rebuild visual proof and docs

Generate new NVIDIA captures, inspect them diagnostically, verify exact repeat
hashes and controlled Off/Radia delta, then update README/demo/build log. Label
the GI method as deterministic screen-space deferred approximation, not path
tracing. Document slow rigid rotation and fixed headless pose.

## Files to touch

- `crates/radia-math/src/rotation.rs` (only if a focused pose test needs API support)
- `crates/radia-render/src/renderer.rs` (pose uniform and graph/pipeline wiring)
- `crates/radia-render/src/graph.rs` (remove history transient/pass)
- `crates/radia-render/src/shaders/scene.wgsl` (uniform dragon pose)
- `crates/radia-render/src/shaders/deferred.wgsl` (deterministic irradiance gather)
- `crates/radia-render/src/shaders/temporal.wgsl` (delete)
- `crates/radia-render/src/shaders/present.wgsl` (scene-radiance naming)
- `crates/radia-render/src/capture.rs` (single-frame capture contract)
- `crates/radia-render/src/evidence.rs` (deterministic manifest)
- `crates/radia-demo/src/main.rs` (monotonic turntable and CLI)
- `README.md`, `assets/stanford-dragon/README.md`, `docs/hackathon/*` (truthful docs)
- `docs/adr/`, `docs/evidence/` (new contracts and proof)

## Verification

1. `cargo fmt --all -- --check` -> exit 0.
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings` -> exit 0.
3. `cargo test --workspace --all-features` -> all CPU/layout/graph tests pass.
4. `scripts/check-matrix-ban.ps1` -> `findings=0`.
5. `rg "cosine_direction|radical_inverse|previous_history|temporal_accumulation" crates/radia-render/src` -> no matches.
6. NVIDIA and AMD ignored Vulkan capture tests -> pass with no validation error.
7. Two fixed-state NVIDIA RADIA captures -> identical decoded bytes and PNG hashes.
8. Controlled Off/Radia evidence -> declared ROI delta is at least `4/255`.
9. Windowed release run -> responsive; dragon pose changes over time while camera,
   lights, and background remain fixed.
10. `agent-code-skills adr check`, plan check, pins check, and doctor -> exit 0.

## Non-goals

- No path tracing, Monte Carlo history, TAA, denoiser, mesh rasterizer, or new dependency.
- No copy of Legaia source, shader formulas, identifiers, assets, or matrices.
- No deformation, morphing, or skeletal animation of the dragon.

## Risks

- Screen-space GI is view-dependent. Bound taps to valid current-frame surfaces
  and document off-screen loss; do not claim reference-quality path transport.
- Dynamic UDF tracing can still expose field-resolution limits. Production
  modes hide indeterminate diagnostics; the explicit hit-state view retains them.
- Continuous redraw can run uncapped. Use elapsed-time orientation so speed is
  frame-rate independent.

## Open questions
