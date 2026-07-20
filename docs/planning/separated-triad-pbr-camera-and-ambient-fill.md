# Separate the dragon triad and add PBR presentation lighting

## Context

The owner requires non-clipping dragon placement, a 65-degree quaternion-only
camera looking down at the shared center by approximately 30 degrees, three
distinct cyan/magenta/yellow material studies, and ambient or bounce light.
This unit preserves the matrix-free deferred renderer, analytic moving lights,
deterministic capture, sampled UDF, and current-frame non-path-traced RADIA.

## Evidence

- `DRAGON_CLUSTER_RADIUS` is 0.78 meters while all three rigid origins share the
  radial arrangement - `crates/radia-render/src/renderer.rs:19-20`,
  `crates/radia-render/src/renderer.rs:251-266`.
- The default camera uses identity rotation, translation `(0,0.15,0.5)`, and a
  60-degree vertical FOV - `crates/radia-render/src/renderer.rs:164-180`.
- Scene tracing assigns one material identifier to all three instances -
  `crates/radia-render/src/shaders/scene.wgsl:39-45`,
  `crates/radia-render/src/shaders/scene.wgsl:196-208`.
- Direct lighting uses a fixed gloss exponent and a 0.006 albedo baseline -
  `crates/radia-render/src/shaders/scene.wgsl:339-377`.
- The deferred primitive palette contains only six entries and the RADIA gather
  identifies only one dragon material -
  `crates/radia-render/src/shaders/deferred.wgsl:31-42`,
  `crates/radia-render/src/shaders/ssgi.wgsl:115-122`.

## Steps

### 1. Freeze the separated presentation contract (first action)

Populate and accept
`adr:separated-dragon-triad-pbr-camera-and-ambient-fill`, superseding the radial
motion ADR only for radius and presentation. Freeze exact placement, camera,
material, BRDF, environment-fill, and RADIA ownership boundaries.

### 2. Implement rigid placement and camera (blocked on step 1)

Raise the origin radius to 2.0 meters under
`adr:conservatively-disjoint-dragon-field-bounds`, with a metadata-derived
separating-axis test for every pair. Build the default camera from a -30
degree active +X `UnitQuat` and translation `(0,3.0,2.3282032)` with a 65-degree
vertical FOV. Add tests for target alignment, pitch, field of view, radial
clearance against embedded asset metadata, and pairwise origin separation.

### 3. Add per-instance PBR material contracts (blocked on step 2)

Assign three consecutive dragon material identifiers by instance index. Add
scene-linear cyan, magenta, and yellow albedos; roughness 0.2, 0.65, and 0.9;
and metallic 1.0 only for cyan. Update normal dispatch, the indirect gather,
and debug palette. Replace the dragon gloss branch with guarded GGX,
Smith-Schlick, and Schlick Fresnel evaluation.

### 4. Add bounded environment fill (blocked on step 3)

Seed direct radiance with deterministic sky/ground irradiance, diffuse response,
and a roughness/Fresnel-bounded specular term. Keep it explicitly unoccluded
and analytic; do not modify the geometry-derived RADIA gather or shadow rays.

### 5. Document and prove the scene (blocked on step 4)

Update README demo instructions and the hackathon demo script. Extend shader
source and camera/placement tests, run both Vulkan adapters, capture fixed-time
physical and diagnostic views, inspect for clipping or field artifacts, validate
repeat determinism and evidence manifests, then rebuild and launch the demo.

## Files to touch

- `docs/adr/77c5dd21-separated-dragon-triad-pbr-camera-and-ambient-fill.md`
- `docs/adr/4c297497-conservatively-disjoint-dragon-field-bounds.md`
- `docs/adr/07789df4-radial-dragon-triad-and-orbital-light-motion.md`
- `docs/adr/INDEX.md`
- `crates/radia-render/src/renderer.rs`
- `crates/radia-render/src/shaders/scene.wgsl`
- `crates/radia-render/src/shaders/deferred.wgsl`
- `crates/radia-render/src/shaders/ssgi.wgsl`
- `crates/radia-render/tests/vulkan_capture.rs`
- `README.md`
- `docs/hackathon/demo-script.md`
- `docs/hackathon/build-log.tsv`
- `docs/evidence/separated-pbr-triad-rtx4070/`

## Verification

1. `agent-code-skills adr check --target C:\Radia` - expected: exit 0 with no ADR findings.
2. `agent-code-skills plan check docs\planning\separated-triad-pbr-camera-and-ambient-fill.md` - expected: exit 0.
3. `cargo fmt --all -- --check` - expected: exit 0 with no diff.
4. `cargo clippy --workspace --all-targets --all-features -- -D warnings` - expected: exit 0.
5. `cargo test --workspace --all-features` - expected: all CPU, placement, camera, shader-contract, and layout tests pass.
6. `powershell -File scripts\matrix-ban.ps1` - expected: exit 0 and no project matrix declaration.
7. `cargo test -p radia-render --test vulkan_capture -- --ignored --nocapture` on RTX 4070 and AMD 610M - expected: captures complete without validation issues.
8. Fixed-state `Radia`, `Off`, `Albedo`, `PrimitiveId`, `LinearDepth`, and `AmbientOcclusion` captures at representative phases - expected: three separated dragons, readable distinct materials, no indeterminate curtain/box, and byte-identical repeats.
9. Validate and compare the controlled-delta and determinism manifests under
   `docs\evidence\separated-pbr-triad-rtx4070` - expected: zero validation
   findings, controlled comparison pass, and exact repeat comparison pass.
10. `cargo run --release -p radia-demo` - expected: the elevated triad and orbiting/bobbing lights remain visible and responsive.

## Non-goals

- No scale, skinning, deformation, mesh replacement, or private Legaia content.
- No path tracing, temporal accumulation, new stochastic process, probe system,
  environment map, or second geometry-derived indirect producer.
- No editor controls, new dependencies, commit, push, Devpost mutation, or publication.

## Risks

- A larger radius can widen the composition beyond the camera frame -> validate
  multiple fixed phases and tune only through this accepted presentation record.
- Metallic GGX can expose numeric or tone-map extremes -> use bounded parameters,
  guarded denominators, HDR intermediates, and final-pass tone mapping.
- Analytic fill can flatten shadows -> keep it low-energy and preserve direct
  shadow visibility plus RADIA ambient accessibility.
- Three new material identifiers can drift across shader passes -> centralize
  helpers and assert all consumers in source-contract tests.

## Open questions
