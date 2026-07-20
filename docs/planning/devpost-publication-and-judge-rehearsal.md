# Devpost publication and judge rehearsal

## Context

The repository now contains the final three-dragon quaternion-deferred demo,
but the complete feature and evidence set is still unpublished. The README
defines the judge gate sequence at `README.md:303`, the current controlled-delta
measurements are recorded at
`docs/evidence/separated-pbr-triad-rtx4070/visual-evidence.tsv:31`, and the
remaining owner-only Devpost fields are listed at
`docs/hackathon/devpost-update-draft.md:67`.

## Objective

Publish the governed Radia reference project, prove a fresh public clone, and
prepare the existing AEP Devpost entry for owner review and final submission.

## Evidence contract

Current fixed-state proof is anchored at
`docs/evidence/separated-pbr-triad-rtx4070/visual-evidence.tsv:28`.

1. The public repository contains no private local paths, credentials, or
   untracked runtime dependencies.
2. Formatting, Clippy, workspace tests, the matrix ban, ADR checks, AEP pin
   checks, and AEP doctor all exit successfully before publication.
3. The ignored Vulkan capture suite succeeds on the available NVIDIA and AMD
   adapters, or the unavailable adapter is recorded explicitly.
4. A fresh clone from the public URL passes the documented judge commands.
5. The Devpost description uses current measured evidence: a 50/255 peak delta,
   3,846 changed subject-and-receiver ROI pixels, and exact repeat hashes.
6. Final submission remains blocked until the owner supplies the required
   submitter type, country, and public under-three-minute YouTube URL and reviews
   the final description in their own voice.

## Ordered steps

1. Sanitize the clean-room boundary and correct stale evidence prose.
2. Run all CPU, governance, source-boundary, and Vulkan gates.
3. Review the complete diff, stage only Radia deliverables, and commit.
4. Push `main`, verify the remote commit, then rehearse from a fresh clone.
5. Prepare the exact Devpost field update for owner approval.
6. Apply the reviewed update and submit only after every required owner field
   and the public video are present.

## Verification

- `cargo fmt --all -- --check` exits 0.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` exits
  0.
- `cargo test --workspace --all-features` reports all non-ignored tests passing.
- `scripts/check-matrix-ban.ps1` reports zero findings.
- `agent-code-skills adr check`, `pins --check`, and `doctor` report zero
  findings or drift.
- The ignored Vulkan capture test reports identical fixed-state PNGs on each
  selected adapter.
- The same CPU gates pass in a fresh clone of the public repository.

## Non-goals

- No private source, asset, path, identifier, or implementation disclosure.
- No dependency change or accepted-ADR mutation.
- No fabricated owner identity field, video URL, visual verdict, or gate result.
