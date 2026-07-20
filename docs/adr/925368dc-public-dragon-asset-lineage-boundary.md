---
id: 925368dc-eb83-45fa-ae5b-2ceec18b3036
slug: adr:public-dragon-asset-lineage-boundary
title: Public dragon asset lineage boundary
status: accepted
supersedes: [adr:source-lineage-and-clean-room-boundary]
supersededBy: null
deciders: ["LairdWT"]
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context

The original clean-room boundary required a fully synthetic scene so public
Radia could not accidentally disclose private Legaia work. The owner has now
requested the public Stanford Chinese Dragon scan. The source is independent
of both prior engine references, but its Stanford Scan terms differ from the
project's Apache-2.0 code license.

## Decision

Preserve the original clean-room boundary: SingularityEngine commit
`15beff98380e34bcdd0b0b4aa392546441e3b472` remains an Apache-2.0 behavioral
bring-up reference only, and no Legaia code, assets, game data, names, paths, or
implementation text may enter the public repository.

Expand the public-content boundary only for the Chinese Dragon OBJ distributed
by Morgan McGuire's Computer Graphics Archive and derived from the Stanford 3D
Scanning Repository. Pin the downloaded archive SHA-256, retain Stanford and
McGuire attribution, and store only the deterministic derived distance volume
plus generation metadata. The engine and tools remain Apache-2.0. The dragon
artifact is separately governed by the Stanford terms: attributed research use
and free redistribution are allowed; commercial use requires permission.

All renderer and baker code remains original Rust and WGSL informed by public
primary math and API sources. The README and Build Week log continue to
distinguish prior AEP work from new Radia work.

## Consequences

The requested public reference asset may be reproduced without private roots,
while code and asset license scopes stay explicit. Release review must scan for
private identifiers and ensure the artifact attribution remains present. A
commercial fork must replace the dragon or obtain Stanford permission.

## Alternatives

Keeping the synthetic-only restriction was rejected because it blocks the
owner's requested scene. Treating the scan as Apache-2.0 or CC0 was rejected
because the primary source does not grant those terms. Copying any Legaia asset
or renderer implementation remains rejected because it violates the public
clean-room boundary.
