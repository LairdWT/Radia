---
id: fb0acee4-7e8b-4f3b-829e-df509f7350b8
slug: adr:coordinate-and-dual-quaternion-semantics
title: Coordinate and dual quaternion semantics
status: accepted
supersedes: []
supersededBy: null
deciders: ["LairdWT"]
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context

Radia must keep camera, object, CPU, GPU, and SDF transforms consistent
without matrix types. Ambiguous quaternion component order, multiplication
order, or coordinate conventions would make mathematically valid code disagree
across those boundaries.

Primary proof basis: Kavan et al., "Skinning with Dual Quaternions",
https://users.cs.utah.edu/~ladislav/kavan07skinning/kavan07skinning.pdf.

## Decision

Use finite `f32`, meters, radians, a right-handed basis, `+X` right, `+Y` up,
and camera forward `-Z`. Rotations are active Hamilton rotations. CPU semantic
quaternions use `wxyz`; GPU adapters pack `xyzw` explicitly.

`UnitQuat` has private raw components and only finite normalized constructors.
It rotates a pure-vector quaternion as `q * p * conjugate(q)`.

`UnitDualQuat` stores `real + epsilon * dual`, with
`dual = 0.5 * translation_quat * real`. Composition
`(ar*br, ar*bd + ad*br)` applies the right operand first. For unit dual
quaternions, conjugating both quaternion parts is the rigid inverse.
Translation is the vector part of `2 * dual * conjugate(real)`. Points receive
rotation plus translation; directions and normals receive rotation only.

Construction and renormalization enforce `length(real)=1` and
`dot(real,dual)=0`. Renormalization first divides both parts by the real-part
length, then subtracts the real-part projection from the dual part. A dual
quaternion and its all-component negation denote the same rigid pose.

Scale, shear, non-finite values, and degenerate normalization are rejected.
Asset scale is baked into vertices.

## Consequences

Every transform is eight `f32` values with explicit host/shader layout. Tests
must cover composition order, inverse, antipodality, Study condition, length
preservation, packing, and invalid input. Blending needs antipodal alignment
and is deferred.

## Alternatives

Matrices were rejected because v1 explicitly tests matrix-free rendering.
Quaternion plus translation was rejected because one normalized rigid pose is
the public transform basis. Euler angles remain input adapters only. Dual
quaternion scale extensions and ScLERP are deferred.
