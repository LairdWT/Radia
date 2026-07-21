# Radia judge README update

## Context

Radia is the public proof repo for the submitted AEP Developer Tools project.
Its README must let judges review the result quickly while keeping the full
Vulkan reproduction path accurate.

## Evidence

- The public demo is `https://youtu.be/A5QJKrxsUS4` (`README.md:28`).
- Submission `1096888` is recorded as Submitted
  (`docs/hackathon/devpost-update-draft.md:153`).
- Committed GPU manifests provide a no-build proof path
  (`docs/evidence/separated-pbr-triad-rtx4070/controlled-delta-manifest.json:1`).
- Full execution requires Rust 1.95 plus Vulkan on Windows or Linux
  (`README.md:102`).

## Goal

Make the public Radia README a self-contained judging path for the AEP Build
Week submission without changing renderer behavior or overstating what can be
run without compiling the Rust project.

## Changes

1. Add a judge guide with the submitted Devpost page, public video, AEP repo,
   feedback task, and direct links to the relevant README sections.
2. Separate the no-build evidence review from the full Vulkan build/run path.
3. State supported platforms, bundled sample-data requirements, asset license,
   key ADR decisions, and the distinct roles of AEP, Codex, and GPT-5.6.
4. Update the hackathon handoff from a pre-submission checklist to the recorded
   submission state while preserving the owner-controlled automation boundary.

## Verification

- `git diff --check`
- README local-link and ASCII scan
- `cargo test --workspace --all-features`

Owner authorized committing and pushing this unit with the remaining reviewed
Radia Build Week WIP on 2026-07-21.

## Non-goals

- No renderer behavior, dependency, asset, submission, or remote AEP change.
