---
id: be81c17f-89d4-4cab-b103-74c8723572e5
slug: adr:source-lineage-and-clean-room-boundary
title: Source lineage and clean-room boundary
status: accepted
supersedes: []
supersededBy: null
deciders: ["LairdWT"]
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context

The project is informed by the public SingularityEngine and a private Legaia
RADIA work-in-progress. The new public repository needs clear provenance and
must not disclose private code, content, paths, or architecture by accidental
transliteration.

## Decision

SingularityEngine commit
`15beff98380e34bcdd0b0b4aa392546441e3b472` is an Apache-2.0 behavioral
reference for basic engine bring-up only. Radia may compare visible lifecycle
goals but does not translate its C++ classes, matrix/Euler code, shaders, or
file layout.

Legaia RADIA is private design evidence only. No Legaia code, assets, game
data, identifiers, local paths, or implementation text may enter this repo.
Public Radia uses original Rust/WGSL code, a synthetic scene, and public
primary math/API sources.

The README and Build Week log distinguish prior AEP work from new Radia work
using dated commits and Codex evidence.

## Consequences

Review includes a private-name/path scan and dependency/license inventory.
Conceptual similarity is documented at the feature level; source lineage is
not implied. Public artifacts remain reproducible without private roots.

## Alternatives

Porting SingularityEngine was rejected because its architecture and matrix
basis conflict with Radia's contract. Copying Legaia's current renderer was
rejected because it is private, active, and built for a different project.
Publishing no lineage note was rejected because the hackathon evaluates only
new work and public reviewers need a clear boundary.
