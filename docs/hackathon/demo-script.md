# Build Week demo plan

Target length: 2 minutes 35 seconds. The final upload must be a public YouTube
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

## 0:45-1:05 - Renderer baseline

- Launch `cargo run --release -p radia-demo`.
- Show the rotating colored triangle.
- Move the camera once, then press `Space` into the analytic courtyard.

## 1:05-1:40 - RADIA off and on

- Show `Off`, then `Radia`, then `GiOnly`.
- Explain the deterministic cosine-weighted sample, one emissive bounce, and
  `RGBA16Float` running mean.
- Cycle quickly through primitive ID, normal, step count, and trace state.

## 1:40-2:05 - Off-screen emitter evidence

- Show the off/on raw PNGs or the controlled-delta contact sheet.
- State that the emitter is analytically outside the camera frustum.
- Show the recorded receiver result: threshold `4/255`, observed `100/255`,
  8,653 changed ROI pixels.
- Show the determinism comparison with zero decoded differences.

## 2:05-2:35 - Codex, GPT-5.6, and handoff

- Show `docs/hackathon/build-log.tsv` and the dated commits.
- Explain that Codex used AEP's GPT-5.6-developed math, Rust, WGPU, WGSL, and
  verification skills to scaffold, freeze decisions, implement, test, and
  package the proof.
- End on the public Radia and AEP repository links and identify Developer Tools
  as the submission category.

Record the narration in the owner's own voice or an allowed narrated voiceover.
Do not read an AI-generated Devpost description verbatim; explain the work in
the owner's own words.
