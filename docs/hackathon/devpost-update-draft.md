# Owner-review Devpost submission draft

This is a local drafting aid for existing project `1345054`, **Agent Enhanced
Projects (AEPs)**. The owner authorized the ordered publication workflow.
Devpost version 8 now contains the public quaternion-deferred triad, current
controlled-delta measurements, dual-adapter Vulkan results, and fresh-clone
judge proof for Radia commit `c063b879418cf85092ace15a091a0d2e016206ee`.
The project page reports `published`. No video was added and no hackathon
submission operation was called.

The Build Week host explicitly asks entrants to write the final project
description in their own voice. The factual version 8 update is live; the owner
must still review it personally and revise it in their own voice before final
submission.

## Version 8 Radia factual source text

The following facts describe the governed Radia publication commit and support
the live version 8 section.

During Build Week I created Radia from a blank repository as a reproducible
downstream test of AEP. Radia is a Rust 1.95/WGPU renderer that uses normalized
dual quaternions for every rigid pose and analytic perspective instead of
transform or projection matrices. Its current demo bakes the 871,306-triangle
Stanford Chinese Dragon into a deterministic 128 cubed unsigned-distance field,
then renders three disjoint instances under three colored orbiting emissive
sources with occlusion shadows and deterministic current-frame screen-space
global illumination. The cyan, magenta, and yellow dragons rotate slowly as
rigid normalized dual-quaternion poses and use distinct roughness and metallic
material settings.

The important result is not just the image. AEP scaffolded the repository,
loaded small math/Rust/WGPU/WGSL skill spokes on demand, froze decisions as
accepted ADRs before dependencies and runtime code, enforced a source-level
matrix ban, and packaged direct GPU evidence. The renderer has no path-traced
bounce, per-frame random sample, temporal history, or learned denoiser. Its
current-frame indirect pass uses 32 spatially phased taps and a fixed
depth/normal-aware resolve. Repeated fixed-state RTX captures have the same PNG
hash. With camera, dragon digest, three lights, direct shadows, trace bounds,
adapter, and gather contract fixed, enabling the sole indirect producer changes
3,846 subject-and-receiver ROI pixels with a peak change of 50/255 against a
required 4/255 threshold. Human-readable 0..1 views now expose albedo, normals,
emissive, linear depth, ambient accessibility, trace steps, and hit state.
Hit-only production depth and a proven conservative UDF domain extension remove
a former grazing-angle black curtain, while the phased resolve removes
coherent AO/RADIA bands. The ignored Vulkan suite
also passes on an AMD Radeon 610M.

This is new submission-period work. AEP itself, SingularityEngine, and the
earlier private RADIA design predate the submission period. Radia copied none
of their code or assets. The public Stanford scan is separately attributed and
licensed for research/free redistribution, not under the project's Apache-2.0
code license. Dated commits separate scaffold, decisions, math, renderer, and
evidence.

## Live required fields

- Category: `Developer Tools`.
- Repository: `https://github.com/LairdWT/Radia` as the new reference project;
  retain `https://github.com/LairdWT/agent-enhanced-project` as the submitted
  developer tool's primary repository.
- Judge instructions: clone Radia, install Rust 1.95 and a Vulkan driver, run
  `cargo test --workspace --all-features`, then
  `cargo run --release -p radia-demo`.
- `/feedback` session ID: `019f75b8-db9b-77b3-87b3-d4870eb66651`.
- Submitter Type: owner must choose.
- Country of Residence: owner must choose; do not infer it.
- Public YouTube video: still required; add only after owner upload.

## Proposed `built_with` additions

Review against the current list before replacing it:

- Codex
- GPT-5.6
- Rust
- WGPU
- WGSL
- Vulkan
- GitHub Actions

## Remaining approval boundary

Do not call the submission operation until the owner has:

1. rewritten and approved the final description in their own voice;
2. selected submitter type and country;
3. supplied the public YouTube URL;
4. reviewed any replacement `links` and `built_with` arrays; and
5. reviewed the live version 8 factual update in their own voice.

Live deadline at the time of drafting: 2026-07-22 00:00 UTC (July 21 at 5:00
PM Pacific Time). The project page reports `published`; the hackathon entry has
no video URL and was not submitted by this work.
