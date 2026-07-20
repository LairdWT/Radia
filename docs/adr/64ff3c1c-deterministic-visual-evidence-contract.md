---
id: 64ff3c1c-c174-4bb4-89f5-c84835f6b255
slug: adr:deterministic-visual-evidence-contract
title: Deterministic visual evidence contract
status: superseded
supersedes: []
supersededBy: adr:three-light-dragon-visual-evidence-contract
deciders: ["LairdWT"]
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context

Rendered pixels are the MVP claim, but a plausible screenshot does not prove
which adapter, settings, code, inputs, or indirect-light path produced it.
Determinism and a controlled delta are needed for review and Build Week.

## Decision

Headless capture records UTC, Git commit, command, Rust/WGPU versions, Vulkan
adapter identity, dimensions, mode, camera/emitter state, sample count, seed
sequence version, output SHA-256, and declared receiver ROI in a manifest.

Two fixed-state captures must decode identically and have identical hashes.
The controlled comparison renders Off and Radia with the emitter outside the
camera's direct view. Receiver albedo, direct-light inputs, depth, and emissive
inputs remain identical. The receiver ROI must change by at least 4/255 in one
decoded display channel. Any validation error, indeterminate trace overflow,
or mismatched control input fails the claim.

Raw PNGs and the manifest remain in a reviewed evidence directory. The user
owns final visual acceptance, video recording, Devpost wording, and submission.

## Consequences

Build and unit tests do not self-certify visual output. Capture and comparison
are repeatable commands with machine-readable failure. Adapter-specific proof
is labeled and does not become a universal cross-GPU equivalence claim.

## Alternatives

Manual screenshots were rejected because provenance and exact comparison are
missing. A perceptual-only threshold was rejected because the synthetic fixed
scene supports decoded pixel comparison. Automatic Devpost submission was
rejected because external publication remains owner-controlled.
