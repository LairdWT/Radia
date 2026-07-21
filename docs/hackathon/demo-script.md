# OpenAI Build Week YouTube demo flow

Target: 2 minutes 30 seconds to 2 minutes 40 seconds. Hard limit: 2 minutes
59 seconds. Review and approve the full draft before recording. The detailed
checklist below remains a technical accuracy reference.

The video must visibly show the working project and audibly say both `Codex`
and `GPT-5.6`.

## Full read-aloud draft

This is the exact owner-approved synthetic-speech pack assembled for OBS. It is
split at the automated cues. The pack starts with the development problem,
explains AEP's mechanism, and ends with developer and project outcomes. The
draft is 332 words: 124.5 words per minute across the 160-second presentation,
with deliberate visual breathing room between sections.

### 0:00-0:33 - Problem and thesis

"It is well known that Coding agents work quickly. However, they are prone to
drift and repeating work. When user decisions live only in chat, those
decisions disappear, standards drift, and users are forced to remind agents of
lost decisions and context. Agent Enhanced Projects, or A E P, fixes that, by
providing an extensible framework of decision schemas, plugins, skills, C L I
tools, and hooks that can grow to fit the project needs."

### 0:33-0:49 - Governed setup

"A E P makes a repository executable. Instructions, decisions, specialist
skills, handoffs, and verification gates are all enforced and maintained by
Mechanical code that does not rely on agent dilligence and user vigilence."

### 0:49-1:04 - Bounded context and technical contracts

"Skills are context efficient and smart loaded on demand. In the example
project the lightweight skill routers load only the Rust, graphics,
mathematics, and operations guidance each task needs, when they are needed."

### 1:04-1:19 - Visible result

"Those contracts and initial user design and directives were solidified as the
railway that built Radia's quaternion-first, matrix-free deferred renderer with
realtime global illumination in just a single session."

### 1:19-1:40 - RADIA pipeline

"In Radia, deterministic deferred lighting, real time shadows, indirect light,
and human readable debug buffers make results inspectable and reviewable. These
features were built using the A E P skills, plugin, and ADRs developed as the
GPT 5.6 Sol agents worked on the project."

### 1:40-2:00 - Investigation and evidence

"When artifacts appeared, A E P required reproduction, measured fixes, and
repeatable G P U evidence. The testing framework enforced by the A E P
governance and hooks created human reviewable deliverables presented in session
with the working GPT 5.6 agent."

### 2:00-2:23 - Developer experience

"Codex with GPT-5.6 also used A E P to build this isolated voice project:
recording consent, governing dependencies, testing conditioning, and generating
this narration you're listening to right now. All using the A E P framework to
create the text to speech project and produce the results you've seen and heard
today."

### 2:23-2:32 - Close

"A E P is the Developer Tools submission, and this voice combined with the
Radia rendering workflow are proof of its effectiveness."

## 0:00-0:15 - Thesis

Show: Radia running in its normal `Radia` view.

Cover:

- [ ] Radia is the Build Week reference project for Agent Enhanced Projects,
  or AEP.
- [ ] AEP is the actual Developer Tools submission; Radia is the public working
  proof that exercises it.
- [ ] One-sentence value: AEP turns agent guidance, decisions, and verification
  into repository-owned, executable governance.

Suggested shape: "This is Radia, a quaternion-first Rust renderer I built from
a blank repository to test AEP on a real graphics project."

## 0:15-0:35 - How AEP built the project

Show: `AGENTS.md`, `CODE.md`, the ADR index, or the build log.

Cover:

- [ ] Radia began as a blank repository during the submission period.
- [ ] AEP scaffolded the workspace, verification commands, CI, convention pins,
  and architecture-decision lifecycle.
- [ ] Only small `*-core` routers load at startup; math, Rust, WGPU, and WGSL
  topic spokes load on demand for the current unit.
- [ ] Decisions were accepted before implementation, then enforced by tests and
  source gates.

Do not imply AEP itself was created during Build Week. AEP, SingularityEngine,
and the earlier private RADIA design predate the submission; this public Radia
repository is new work.

## 0:35-0:55 - Quaternion-first, matrix-free proof

Show: quaternion code or the matrix-ban result, then return to the renderer.

Cover:

- [ ] Every rigid camera and model pose uses a normalized dual quaternion.
- [ ] CPU quaternion semantics are `wxyz`; the GPU adapter packs explicit
  `xyzw` values.
- [ ] Perspective and reverse-Z projection are analytic. No view, model,
  projection, or inverse-view matrix is constructed or uploaded.
- [ ] The project matrix-ban gate reports zero matrix types in the owned Rust
  render/math sources and WGSL shaders.

Avoid saying vectors were removed. Vectors remain the correct representation
for positions, directions, normals, and colors.

## 0:55-1:25 - Three-dragon lighting scene

Show: the normal `Radia` view long enough to see rotation and all light paths.

Cover:

- [ ] The public Stanford Chinese Dragon has 871,306 triangles and is baked
  once into a deterministic `128^3` unsigned-distance field.
- [ ] Call it a UDF, not an exact signed-distance field: the source scan is
  holed, so Radia makes no reliable inside/outside claim for the dragon.
- [ ] Three rigid instances face outward around one center. Their field bounds
  have a tested conservative clearance, so they do not clip.
- [ ] Dragon base colors are cyan, magenta, and yellow. Roughness values are
  `0.2`, `0.65`, and `0.9`; only cyan is metallic `1.0`.
- [ ] The camera uses a 65-degree vertical FOV and looks down toward the center
  at about 30 degrees.
- [ ] Three colored emissive lights orbit at independent speeds and move up and
  down with independent sine waves.
- [ ] Direct GGX lighting, distance-field visibility, colored shadows, and a
  bounded analytic sky/ground fill make the material response readable.

## 1:25-1:55 - What RADIA does

Show: press `Space` through `Off`, `GiOnly`, albedo, normal, emissive, depth,
and ambient-occlusion/accessibility views.

Cover:

- [ ] This version is deterministic deferred rendering, not path tracing.
- [ ] Pass order: G-buffer -> direct lighting -> current-frame indirect light ->
  edge-aware composite -> presentation.
- [ ] RADIA gathers geometry-derived screen-space indirect radiance with 32
  spatially phased taps and a fixed 5x5 depth/normal-aware resolve.
- [ ] There is no per-frame random sample, temporal accumulation, convergence
  grain, learned denoiser, or second indirect producer.
- [ ] Debug views are human-readable `0..1` inspections. Linear depth is
  normalized over the trace interval; accessibility is black when occluded and
  white when open.

Do not call the indirect result unbiased, physically complete, or multi-bounce.
It is a bounded, current-frame, screen-space one-bounce approximation.

## 1:55-2:15 - Defect fixes and proof

Show: hit state, step count, then the committed evidence manifest or Off/Radia
comparison.

Cover:

- [ ] Production depth is written only for hits. The sampled UDF extends beyond
  its volume using a proven conservative lower bound.
- [ ] Those changes removed the former grazing-angle black curtain; spatial
  phasing plus edge-aware resolve removed the coherent banding.
- [ ] Fixed-state Radia captures repeat byte-for-byte on the same adapter.
- [ ] Controlled comparison fixes camera, dragon digest and poses, all three
  lights, shadows, trace bounds, adapter, and gather contract. Only indirect
  mode changes.
- [ ] Required threshold: `4/255`. Observed peak: `50/255`. Changed
  subject-and-receiver ROI pixels: `3,846`.
- [ ] All 36 CPU tests pass; ignored Vulkan capture tests pass on both the RTX
  4070 Laptop GPU and AMD Radeon 610M.

## 2:15-2:40 - Codex, GPT-5.6, voice workflow, and close

Show: build log, dated commits, README reproduction commands, then finish on
the running scene or public repo links.

Cover:

- [ ] Explicitly say `Codex` and `GPT-5.6`.
- [ ] Codex with GPT-5.6 used AEP's routed math, Rust, WGPU, WGSL, planning, and
  verification skills to scaffold, research, freeze decisions, implement,
  diagnose screenshot/video defects, and package evidence.
- [ ] AEP also scaffolded and governed the isolated LocalTTS project used for
  this narration: owner consent, dependency decisions, reference hashing,
  bounded conditioning, tests, watermarking, and provenance all passed through
  explicit gates.
- [ ] Explain why this matters: the same governed workflow survived context
  changes and produced a fresh-clone result that judges can reproduce.
- [ ] Name the submission category: Developer Tools.
- [ ] End with both public repositories:
  `github.com/LairdWT/agent-enhanced-project` and
  `github.com/LairdWT/Radia`.

Suggested close: "AEP is the tool; Radia is the proof. Both repositories are
public, and the Radia README reproduces the tests, evidence capture, and demo."

## Renderer control order

For the automated take, use `scripts/record-devpost-demo.ps1`; its versioned
timeline selects modes explicitly and keeps this cue sheet synchronized with
the visuals. The keyboard sequence below remains the manual recovery path.

Start in `Radia`. Each `Space` press advances:

1. `Off` - direct lighting and ambient fill, no RADIA indirect term.
2. `GiOnly` - RADIA indirect contribution alone.
3. `Albedo`.
4. `Normal`.
5. `Emissive`.
6. `LinearDepth`.
7. `AmbientOcclusion` - ambient accessibility display.
8. `SdfDistance`.
9. `PrimitiveId`.
10. `StepCount`.
11. `HitState` - blue miss, green hit, magenta indeterminate.
12. `Triangle` - matrix-free baseline.
13. `Radia`.

Use `R` to restart the dragon and light animation before recording. Avoid
camera movement unless it helps composition. Leave at least five uninterrupted
seconds on the final `Radia` view.

## Final upload checklist

- [ ] Duration below 3:00.
- [ ] Public or unlisted YouTube URL that judges can open without signing in.
- [ ] Working project visibly demonstrated.
- [ ] Spoken `Codex` and `GPT-5.6` coverage.
- [ ] AEP identified as submission; Radia identified as reference project.
- [ ] Prior work and new Build Week work distinguished.
- [ ] No private Legaia paths, source, assets, identifiers, or implementation
  details shown.
- [ ] No claim of path tracing, exact dragon SDF, unbiased GI, or production
  multi-bounce.
- [ ] Public repo links visible or spoken.
- [ ] Audio intelligible; no desktop notifications or private windows visible.
