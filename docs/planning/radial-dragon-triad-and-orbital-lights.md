# Add the radial dragon triad and orbital light motion

## Context

The renderer must present three copies of the Jade Dragon with their tail sides
at a shared center and their heads pointing outward. The red, green, and blue
finite emitters must orbit that group at distinct speeds and bob vertically at
distinct sinusoidal rates. User decision (2026-07-19): preserve the existing
quaternion-first, matrix-free deferred renderer and current-frame RADIA method;
animate this composition in the live demo and keep headless evidence fixed by
an explicit scene time.

## Evidence

- `RenderSettings` owns one dragon pose and one host-authored emitter -
  `crates/radia-render/src/renderer.rs:74-88`.
- The default dragon is a single dual-quaternion pose at `(0,-1,-4.6)` and the
  interactive demo replaces it from elapsed time -
  `crates/radia-render/src/renderer.rs:95-136`,
  `crates/radia-demo/src/main.rs:310-369`.
- WGSL stores one dragon pose while green and blue light records are literals -
  `crates/radia-render/src/shaders/scene.wgsl:1-15`,
  `crates/radia-render/src/shaders/scene.wgsl:99-125`.
- Scene tracing currently loses dragon-instance identity because `SceneSample`
  and `TraceHit` carry material only -
  `crates/radia-render/src/shaders/scene.wgsl:21-32`,
  `crates/radia-render/src/shaders/scene.wgsl:201-267`.
- Host packing is a hand-authored 176-byte layout with byte-offset tests -
  `crates/radia-render/src/renderer.rs:1190-1329`.
- Deterministic capture already owns an optional exact dragon angle and defaults
  to the initial pose - `crates/radia-render/src/capture.rs:9-113`.

## Steps

### 1. Freeze motion and placement (first action)

Accept `adr:radial-dragon-triad-and-orbital-light-motion` and supersede
`adr:rigid-dragon-turntable-presentation`. The new record owns the radial pose
derivation, orbit and bob equations, constants, uniform boundary, deterministic
time contract, and performance consequence.

### 2. Add analytic scene motion (blocked on step 1)

Replace the single pose/emitter settings seam in
`crates/radia-render/src/renderer.rs` with three typed lights and three
normalized dual-quaternion poses. Add finite-time construction for the triad
and emitter paths. Retain an explicit cluster-angle adapter for regression
captures. Unit tests prove radial separation, outward local `-Z`, inward local
`+Z`, 120-degree spacing, bounded bob heights, distinct orbit speeds,
repeatability, and non-finite refusal.

### 3. Mirror the expanded frame contract in WGSL (blocked on step 2)

Expand the host/WGSL uniform to three position/radius records, three
color/intensity records, and three real/dual quaternion pairs. Loop over dragon
instances in scene distance, carry the winning instance index through
`TraceHit`, and use it for UDF normal reconstruction. Keep the existing
material IDs, debug modes, conservative UDF extension, shadow contract, and
matrix ban. Assert every host byte offset and shader source seam.

### 4. Thread deterministic time through live and capture paths (blocked on step 3)

Update `crates/radia-demo/src/main.rs` so one monotonic elapsed-time value
updates the triad and all emitters before each frame. Add
`--scene-time-seconds` to headless capture, retain
`--dragon-angle-degrees` as a cluster-angle override, and record both in the
capture report and evidence signature. Update CPU reference tests to consume
the typed primary light.

### 5. Document and visually verify (blocked on step 4)

Update `README.md` and the hackathon demo script with the three-dragon staging,
orbit/bob behavior, reset control, and deterministic capture command. Produce
fresh direct render-target captures at fixed times, inspect a contact sheet for
tail-center composition and field artifacts, then launch the release demo.

## Files to touch

- `docs/adr/07789df4-radial-dragon-triad-and-orbital-light-motion.md` (accepted decision)
- `docs/adr/76bc3596-rigid-dragon-turntable-presentation.md` (mechanical supersede metadata)
- `docs/adr/INDEX.md` (tool-owned decision index)
- `crates/radia-render/src/renderer.rs` (scene types, motion, packing, tests)
- `crates/radia-render/src/shaders/scene.wgsl` (arrays, instance tracing)
- `crates/radia-render/src/shaders/gbuffer.wgsl` (instance-correct normal)
- `crates/radia-render/src/capture.rs` (fixed scene time contract)
- `crates/radia-render/src/evidence.rs` (complete scene signature/manifest)
- `crates/radia-render/src/sdf.rs` (typed primary-light adapter)
- `crates/radia-render/src/lib.rs` (public scene-motion surface)
- `crates/radia-demo/src/main.rs` (live motion and capture CLI)
- `crates/radia-render/tests/vulkan_capture.rs` (Vulkan fixed-time proof)
- `README.md` (demo instructions and architecture)
- `docs/hackathon/demo-script.md` (showcase narration)

## Verification

1. `agent-code-skills adr check --target C:\Radia` - expected: exit 0 with no ADR findings.
2. `agent-code-skills plan check docs\planning\radial-dragon-triad-and-orbital-lights.md` - expected: exit 0.
3. `cargo fmt --all -- --check` - expected: exit 0 with no diff.
4. `cargo clippy --workspace --all-targets --all-features -- -D warnings` - expected: exit 0.
5. `cargo test --workspace --all-features` - expected: all CPU, layout, motion, shader-contract, and CLI tests pass.
6. `powershell -File scripts\matrix-ban.ps1` - expected: exit 0 and no project matrix type or WGSL matrix declaration.
7. `cargo test -p radia-render --test vulkan_capture -- --ignored --nocapture` on each Vulkan adapter - expected: fixed-time captures complete with no validation issue.
8. Capture `Radia`, `Off`, `HitState`, `PrimitiveId`, and `LinearDepth` at fixed times `0`, `5`, and `10` seconds - expected: three outward-facing instances, centered tail sides, three distinct light positions, no false curtain/box, and identical repeat hashes for identical state.
9. `cargo run --release -p radia-demo` - expected: all three dragons turn as one radial group while the red, green, and blue spheres orbit at different speeds and bob at different vertical rates.

## Non-goals

- No path tracing, temporal accumulation, stochastic animation, or new GI producer.
- No dragon scaling, skinning, deformation, new asset, or copied private Legaia code.
- No editor controls, collision response, or light-path spline authoring.
- No commit, push, Devpost mutation, or publication in this unit.

## Risks

- Three field instances increase shader cost -> retain bounded loops, measure the release demo, and report rather than hide an unsupported adapter.
- Overlapping UDF surfaces can make the tail center visually dense -> tune only the frozen rigid radius through a superseding decision if fixed-time captures fail.
- Uniform-array offsets can silently disagree -> use explicit byte packing, offset tests, exact WGPU validation, and deterministic render-target capture.

## Open questions
