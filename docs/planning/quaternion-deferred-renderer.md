# Build a quaternion-first deferred render graph

## Context

Radia currently ray-marches geometry, shades it, and updates temporal history in
one full-screen fragment pass. The next reference slice must mirror the proven
structural shape of the private Legaia deferred renderer without copying private
code: named transients, a fixed pass order, a bounded four-target G-buffer, a
full-screen lighting resolve, temporal history, and presentation. Radia retains
its stricter math contract: normalized dual-quaternion rigid poses and analytic
reverse-Z projection/reconstruction with no matrix type or value.

User decisions (2026-07-18): use a pre-existing private renderer only as
read-only architectural evidence; implement a fresh public Rust/WGPU/WGSL
deferred graph in Radia; keep the jade dragon, three colored emissive sources,
distance-field shadows, and one-bounce RADIA; do not add dependencies or copy
Legaia code, assets, identifiers, or game data.

## Evidence

- Radia stores one accumulate pipeline plus one present pipeline and only
  history targets - `crates/radia-render/src/renderer.rs:127-145`.
- Radia's first pass writes history directly and the second presents it, so no
  geometry/lighting attachment boundary exists -
  `crates/radia-render/src/renderer.rs:341-443`.
- `fs_accumulate` currently performs primary trace, shading, and history mean
  in one entry point - `crates/radia-render/src/shaders/radia.wgsl:503-529`.
- Radia already freezes dual-quaternion pose semantics, explicit `xyzw` GPU
  packing, and matrix rejection -
  `docs/adr/fb0acee4-coordinate-and-dual-quaternion-semantics.md:18-49`.
- Radia already freezes analytic WGPU `[0,1]` infinite-far reverse-Z and
  matrix-free screen rays -
  `docs/adr/a8e95262-matrix-free-reverse-z-projection.md:17-43`.
- Owner-provided private design evidence established the broad deferred shape:
  graph-owned transients, explicit read/write ordering, a bounded G-buffer,
  full-screen lighting, and a separate presentation pass. No private path,
  identifier, formula, asset, or implementation detail is part of Radia.

## Steps

### 1. Freeze the deferred graph decision (first)

Mint, populate, and owner-accept one ADR defining Radia's named transients,
formats, pass order, quaternion/analytic reconstruction boundary, clean-room
relationship to Legaia, and deferred features. Run the ADR checker before code.

### 2. Add graph-owned transient targets (blocked on step 1)

Add a small fixed graph module owning four full-resolution G-buffer color
targets, `Depth32Float`, two `Rgba16Float` history targets, resize recreation,
and named pass contracts. Validate read-before-write ordering in unit tests.

### 3. Split geometry and lighting shaders (blocked on step 2)

Extract Radia-owned shared quaternion, projection, scene-field, trace, material,
light, and sampling functions into a common WGSL source. Add a geometry entry
that writes albedo, world normal/material, emissive, trace/debug data, and
reverse-Z depth. Add a deferred entry that reconstructs world position from
pixel center plus depth analytically, performs the three-light shadowed resolve
and optional RADIA bounce, and writes one linear-HDR scene-radiance sample. Add
a separate temporal entry that combines that sample with previous history.

### 4. Encode the fixed pass sequence (blocked on step 3)

Replace the monolithic accumulation pass with explicit `gbuffer_geometry ->
deferred_lighting -> temporal_history -> presentation` resource flow. Keep the
existing public renderer API, mode cycle, reset semantics, Vulkan-only adapter
policy, typed GPU error handling, and capture/evidence entry points.

### 5. Prove the deferred boundary and rendered result (blocked on step 4)

Add host-layout, attachment-format, pass-order, shader-contract, matrix-ban,
and ignored Vulkan checks. Generate fresh fixed-state direct/RADIA captures in a
new evidence directory, validate deterministic repetition and controlled ROI
delta, and update README/demo/build-log facts without rewriting old evidence.

## Files to touch

- `docs/adr/INDEX.md` and one generated ADR (mechanical deferred decision).
- `docs/planning/quaternion-deferred-renderer.md` (this execution contract).
- `crates/radia-render/src/graph.rs` (transient ownership and pass contracts).
- `crates/radia-render/src/lib.rs` (private graph module registration).
- `crates/radia-render/src/renderer.rs` (layouts, pipelines, bind groups, pass encoding, resize).
- `crates/radia-render/src/shaders/scene.wgsl` (shared quaternion/scene contract).
- `crates/radia-render/src/shaders/gbuffer.wgsl` (geometry outputs and reverse-Z depth).
- `crates/radia-render/src/shaders/deferred.wgsl` (lighting, RADIA, and debug output).
- `crates/radia-render/src/shaders/temporal.wgsl` (running-mean history update).
- `crates/radia-render/src/shaders/radia.wgsl` (retired monolithic entry source).
- `crates/radia-render/tests/vulkan_capture.rs` (runtime graph validation assertions).
- `docs/evidence/quaternion-deferred-rtx4070/` (new controlled evidence only).
- `README.md`, `docs/hackathon/demo-script.md`, and
  `docs/hackathon/build-log.tsv` (reproduction and lineage updates).

## Verification

1. `cargo fmt --all -- --check` - expected: exit 0 and no diff.
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings` - expected: exit 0.
3. `cargo test --workspace --all-features` - expected: all CPU, graph, layout, SDF, and shader-contract tests pass.
4. `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-matrix-ban.ps1` - expected: `findings=0`.
5. `agent-code-skills adr check --target C:\Radia` - expected: `findings=0`.
6. `agent-code-skills pins --check C:\Radia --repo C:\aep-math-worktree` and
   `agent-code-skills doctor --target C:\Radia --repo C:\aep-math-worktree` - expected: zero drift/findings.
7. `cargo test -p radia-render --test vulkan_capture -- --ignored --nocapture` with `RADIA_ADAPTER_NAME=NVIDIA`, then AMD - expected: Vulkan validation clean and capture succeeds on both adapters.
8. Two fixed-state deferred RADIA captures plus Off/Radia comparison - expected: repeated decoded samples and PNG hashes match; declared ROI peak delta remains at least `4/255`; direct shadows remain in Off.
9. `cargo run --release -p radia-demo` - expected: jade dragon renders through the deferred graph; all modes, resize, camera reset, and exit work.

## Non-goals

- No Legaia code, game data, assets, identifiers, matrices, raster mesh path, or full feature parity.
- No velocity buffer, TAA, SSAO, DFAO, SSGI, bloom, transparent pass, ECS, mesh import, or new dependency in this slice.
- No Devpost submission, Git commit, push, or baseline deletion.

## Risks

- Four floating-point G-buffer targets may exceed an adapter capability -> use
  formats already validated on both target adapters and fail setup explicitly.
- Writing `frag_depth` from a full-screen ray march can expose invalid near or
  miss values -> write only finite `(0,1]` hit depth and preserve zero clear for misses.
- Shader factoring can drift shared CPU/GPU semantics -> keep one common WGSL
  source per pipeline composition and retain numeric/layout tests.
- Existing release demo may hold the Windows executable -> stop only that exact
  Radia process before the release rebuild, then relaunch after gates.

## Open questions
