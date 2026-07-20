# Remove indeterminate depth curtains and expose deferred buffers

## Context

User decisions (2026-07-18): remove the black plane visible beside the dragon
at some turntable angles; add human-readable 0..1 views for depth, AO, and
other useful deferred data; update the existing AEP Devpost project after the
implementation is proved. Keep the quaternion-first, matrix-free, Vulkan-only
renderer and do not submit the hackathon entry.

Decisions: `adr:inspectable-deferred-buffer-and-telemetry-contract` and
`adr:spatially-phased-and-edge-resolved-deferred-radia`.

## Evidence

- The bounded primary trace returns `TRACE_INDETERMINATE` at either the
  configured step limit or the hard loop limit -
  `crates/radia-render/src/shaders/scene.wgsl:235`.
- The G-buffer writes nonzero reverse-Z depth before returning an indeterminate
  result - `crates/radia-render/src/shaders/gbuffer.wgsl:31`.
- Production deferred shading maps that state to dark background, so the false
  depth becomes a black tangent curtain -
  `crates/radia-render/src/shaders/deferred.wgsl:20`.
- The two supplied window captures show a thin dark vertical surface only at a
  grazing dragon angle - observed runtime captures
  `Screenshot 2026-07-18 231029.png` and
  `Screenshot 2026-07-18 231038.png`.
- The fixed graph already exposes albedo, normal/material, emissive, trace,
  reverse-Z depth, direct radiance, and indirect radiance without exceeding
  four simultaneous color targets - `crates/radia-render/src/graph.rs:52`.
- The deterministic indirect pass already uses sixteen current-frame integer
  texel offsets and leaves target alpha unused -
  `crates/radia-render/src/shaders/ssgi.wgsl:36` and
  `crates/radia-render/src/shaders/ssgi.wgsl:99`.
- The supplied AO and Radia captures show repeated horizontal bands. An
  identical-angle Off capture retains broad direct-shadow shapes without the
  repeated striping; the indirect shader places all sixteen taps on only two
  radii and clamps out-of-frame taps, so planar depth crossings change on the
  same small set of framebuffer rows - observed runtime comparison and
  `crates/radia-render/src/shaders/ssgi.wgsl:41`.
- A later 170 degree capture proves a second curtain mechanism: HitState marks
  the strip indeterminate, and the UDF outside-domain branch discards clamped
  boundary samples in favor of an error-sized constant step. The 2.94 meter
  local depth can therefore exhaust 128 steps - observed runtime captures and
  `crates/radia-render/src/shaders/scene.wgsl:161`.
- Presentation currently tone-maps every value, which maps a normalized debug
  value of 1 to 0.5 - `crates/radia-render/src/shaders/present.wgsl:20`.
- Live Devpost project 1345054 is published but not submitted; its Radia section
  still describes the superseded Hammersley path - observed Devpost project
  read on 2026-07-18.

## Steps

1. Accept the inspectable-buffer decision before code changes, including the
   debug-only telemetry sentinel required by greater reverse-Z comparison.
2. Change the G-buffer state boundary so only hits write depth and surface
   attributes; keep trace state and steps for misses and indeterminate rays.
   Add a source-contract test for the invariant.
3. Extend `RadiaMode` and CLI parsing with Albedo, Emissive, LinearDepth, and
   AmbientOcclusion. Make all mode values, cycle order, and names testable.
4. Refactor deferred debug selection so trace-state views work without depth,
   surface views require a hit, and all scalar/vector debug outputs follow the
   accepted 0..1 mappings.
5. Store deterministic screen-space accessibility in indirect alpha, expose it
   through composite AO mode, and leave production lighting unchanged.
6. Bind frame mode to presentation so physical radiance is tone-mapped once and
   normalized debug values pass through unchanged.
7. Add a capture-only dragon angle override so the reported grazing angle and
   every buffer mode can be reproduced headlessly without changing fixed
   evidence defaults.
8. Update README, demo script, build log, and visual evidence. Relaunch the
   release window after all local gates pass.
9. Supersede the sparse two-ring gather contract with thirty-two deterministic
   antipodal radial strata, a fixed 4x4 spatial phase, and a bounded 5x5
   depth/normal-aware resolve. Reject rather than clamp edge taps and prove the
   AO/Radia bands are absent against an identical Off control at 640x360 and
   1280x720.
10. Read the live Devpost project, replace only the Radia progress section with
   current verified facts, and update project 1345054. Do not call a submission
   tool.
11. Replace the sampled UDF's outside-domain error shell with the conservative
    AABB plus boundary-sample lower bound, prove boundary clearance, and repeat
    the 170 degree debug suite plus full-turn scan.

## Files to touch

- `crates/radia-render/src/shaders/gbuffer.wgsl` (hit-only depth invariant)
- `crates/radia-render/src/shaders/deferred.wgsl` (normalized G-buffer views)
- `crates/radia-render/src/shaders/ssgi.wgsl` (ambient accessibility alpha)
- `crates/radia-render/src/shaders/composite.wgsl` (AO selection)
- `crates/radia-render/src/shaders/present.wgsl` (mode-aware presentation)
- `crates/radia-render/src/renderer.rs` (mode API, pose helper, binding/test updates)
- `crates/radia-render/src/capture.rs` (capture pose override)
- `crates/radia-demo/src/main.rs` (CLI modes, angle option, controls/tests)
- `crates/radia-render/tests/vulkan_capture.rs` (GPU mode coverage)
- `README.md` (buffer contracts and demo commands)
- `docs/hackathon/demo-script.md` (demo sequence)
- `docs/hackathon/devpost-update-draft.md` (owner-readable published copy)
- `docs/hackathon/build-log.tsv` (dated gate evidence)
- `docs/evidence/inspectable-buffers-rtx4070/` (raw deterministic captures)

## Verification

1. `cargo fmt --all -- --check` -> exit 0 with no diff.
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings` ->
   exit 0 with no warnings.
3. `cargo test --workspace --all-features` -> all CPU, shader-contract, mode,
   and capture-config tests pass.
4. `powershell -File scripts/check-matrix-ban.ps1` -> zero project matrix
   findings.
5. `agent-code-skills adr check --target C:\Radia` and
   `agent-code-skills plan check C:\Radia\docs\planning\inspectable-deferred-buffers.md`
   -> exit 0 with no findings.
6. `agent-code-skills pins --check C:\Radia --repo C:\aep-math-worktree` and
   `agent-code-skills doctor --target C:\Radia --repo C:\aep-math-worktree` ->
   zero drift and zero findings.
7. NVIDIA and AMD ignored Vulkan capture tests -> validation clean on both
   adapters.
8. Headless Radia, HitState, LinearDepth, AmbientOcclusion, Albedo, Normal, and
   Emissive captures at the reported grazing angle -> PNGs decode, contain no
   non-finite values, and normalized debug captures stay within display range.
9. Repeated fixed-state Radia capture -> identical decoded samples and SHA-256.
10. Windowed release run -> dragon rotates, reported black curtain is absent,
    Space reaches every named buffer view, and window remains responsive.
11. Live Devpost readback -> project 1345054 contains the verified deterministic
    deferred/turntable/buffer progress and remains unsubmitted.

## Non-goals

- No path tracing, temporal history, standalone/temporal/learned denoiser,
  probe volume, new dependency, matrix transform, mesh deformation, or
  production AO modulation.
- No Git commit, push, release, Devpost submission, video upload, or thumbnail
  change.

## Risks

- Screen-space AO is incomplete at edges and occluded surfaces; label it as
  accessibility and keep it diagnostic-only.
- Sparse aligned screen-space taps create coherent bands at planar depth
  boundaries; retain deterministic stratification and compare AO/Radia against
  an identical Off control.
- Bypassing tone mapping for debug modes changes display transfer; keep physical
  modes on the existing tone-map path and verify both branches.
- A stale running executable can hide the fix; stop only the exact Radia demo
  process launched by this task, rebuild release, then relaunch.

## Open questions
