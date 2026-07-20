# Build Week demo plan

Target length: 2 minutes 40 seconds. The final upload must be a public YouTube
video under three minutes with spoken coverage of the project, Codex, and
GPT-5.6.

## 0:00-0:20 - AEP starts the project

- Show the blank-project premise and the checked-in `AGENTS.md`, `CODE.md`, and
  accepted ADR index.
- Say that AEP exposes core routers at startup and loads only the topic spokes
  needed for each unit.

## 0:20-0:45 - Matrix-free proof

- Show `UnitQuat`, `UnitDualQuat`, explicit CPU `wxyz` to GPU `xyzw` packing,
  and `ReverseZPerspective`.
- Run `scripts/check-matrix-ban.ps1` and show `findings=0`.
- Explain that vectors still exist, but every rigid pose is a normalized dual
  quaternion and projection is analytic.

## 0:45-1:10 - Jade dragon and three-light shadows

- Launch `cargo run --release -p radia-demo`.
- Identify the three mesh-derived Stanford Chinese Dragon instances and the
  red, green, and blue emissive sources.
- Point out the 65-degree matrix-free camera: its normalized quaternion looks
  down at the shared center by 30 degrees.
- Point out the colored occlusion shadows, the three different light-orbit
  speeds, and their different sine-wave vertical motions.
- Show that the three tail sides point toward one center while all heads point
  outward. Their 2.0 meter origins leave conservatively proven disjoint field
  bounds, and the complete radial group turns slowly without breaking that layout.
- Switch briefly to albedo to name the mid-value cyan, magenta, and yellow base
  colors. State roughness `0.2`, `0.65`, and `0.9`; cyan alone is metallic 1.0.
- State that the 871,306-triangle mesh is baked once into a deterministic 128
  cubed unsigned-distance field and instanced by three normalized dual
  quaternions; runtime poses and camera remain matrix-free.

## 1:10-1:45 - Quaternion-first deferred RADIA

- Begin in `Radia`, then press `Space` for direct-only `Off`, then `GiOnly`.
- Explain the fixed G-buffer, direct-lighting, 32-tap spatially phased SSGI,
  edge-aware composite, and presentation passes. World position is
  reconstructed from reverse-Z depth and the camera dual quaternion without an
  inverse-view or projection matrix.
- Explain that guarded GGX handles direct material response. A deterministic
  sky/ground environment term is only unoccluded baseline fill; RADIA remains
  the separate current-frame, geometry-derived bounce approximation.
- State that current RADIA has no path tracing, per-frame random sample,
  temporal history, convergence grain, or learned denoiser. It gathers
  current-frame direct radiance from guarded screen-space surfaces and resolves
  it using current depth and normals.
- Cycle through albedo, normal, bounded emissive, linear depth, and ambient
  occlusion. State that normalized debug values bypass radiance tone mapping;
  depth is camera distance over the trace interval and AO uses black for
  occluded, white for open.
- Continue through primitive ID, step count, and trace state. Point out that
  production modes write depth only for hits, while the sampled UDF extends its
  boundary field with a conservative lower bound. Together they remove the
  former rotating black curtain; trace-only modes retain non-hit telemetry.
- Show the rotating dual-quaternion triangle baseline, then return to `Radia`.

## 1:45-2:10 - Controlled three-light evidence

- Show the fixed-state `Off` and `Radia` PNGs plus the TSV manifest.
- State that camera, dragon digest, all three instance poses, all three lights,
  explicit scene time, direct shadows, trace bounds, adapter, and gather taps
  are fixed; only indirect mode changes.
- Show the recorded ROI result: threshold `4/255`, observed `50/255`, and 3,846
  changed subject-and-receiver pixels.
- Show the repeated fixed-adapter hash match.

## 2:10-2:40 - Codex, GPT-5.6, and handoff

- Show `docs/hackathon/build-log.tsv` and the dated commits.
- Explain that Codex used AEP's GPT-5.6-developed math, Rust, WGPU, WGSL, and
  verification skills to scaffold, freeze decisions, implement, test, and
  package the proof.
- End on the public Radia and AEP repository links and identify Developer Tools
  as the submission category.

Record the narration in the owner's own voice or an allowed narrated voiceover.
Do not read an AI-generated Devpost description verbatim; explain the work in
the owner's own words.
