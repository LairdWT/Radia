# Owner-review Devpost update draft

This is a local drafting aid for existing project `1345054`, **Agent Enhanced
Projects (AEPs)**. The Devpost project was read on 2026-07-18 and remained in
`submission_draft`; no remote fields were changed.

The Build Week host explicitly asks entrants to write the final project
description in their own voice. Treat the text below as factual source material,
rewrite it personally, review every claim, and only then approve an update.

## Proposed new-work addendum

During Build Week I created Radia from a blank repository as a reproducible
downstream test of AEP. Radia is a Rust 1.95/WGPU renderer that uses normalized
dual quaternions for every rigid pose and analytic perspective instead of
transform or projection matrices. It progresses from a rotating triangle to an
analytic signed-distance courtyard and a deterministic one-bounce emissive
global-illumination mode.

The important result is not just the image. AEP scaffolded the repository,
loaded small math/Rust/WGPU/WGSL skill spokes on demand, froze six decisions as
accepted ADRs before dependencies and runtime code, enforced a source-level
matrix ban, and packaged direct GPU evidence. Two 1024-sample RTX captures are
byte-identical. With the emitter analytically off-screen, enabling the sole
indirect producer changes 8,653 receiver-ROI pixels with a peak change of
100/255 against a required 4/255 threshold. The ignored GPU suite also passes
on an AMD Radeon 610M.

This is new submission-period work. AEP itself, SingularityEngine, and the
earlier private RADIA design predate the submission period. Radia copied none
of their code or assets; its dated commits separate scaffold, decisions, math,
renderer, and evidence.

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

## Approval boundary

Do not call the Devpost update or submit operations until the owner has:

1. rewritten and approved the description in their own voice;
2. selected submitter type and country;
3. supplied the public YouTube URL;
4. reviewed any replacement `links` and `built_with` arrays; and
5. explicitly authorized the remote update and, separately, final submission.

Live deadline at the time of drafting: 2026-07-22 00:00 UTC (July 21 at 5:00
PM Pacific Time). The project state was `submission_draft` and had no video URL.
