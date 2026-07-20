---
id: 76bc3596-c1ca-420c-8f03-b134321f0866
slug: adr:rigid-dragon-turntable-presentation
title: Rigid dragon turntable presentation
status: superseded
supersedes: []
supersededBy: adr:radial-dragon-triad-and-orbital-light-motion
deciders: ["LairdWT"]
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context

The dragon pose is currently hard-coded in WGSL, so the window cannot present a
slow rotation through the project's normalized dual-quaternion transform path.
The owner requires the dragon to rotate slowly while the camera, three lights,
and scene remain fixed.

## Decision

Make the dragon world pose an explicit `UnitDualQuat` render setting and GPU
uniform packed as two `xyzw` quaternions. Preserve the right-handed, active
rotation convention. In the interactive demo, rotate about world `+Y` at a
constant `0.12` radians per second:

`theta(t) = pi/2 + 0.12 * t`, reduced modulo `2*pi` before axis-angle
construction.

Use a monotonic `Instant` epoch and construct a fresh normalized `UnitQuat` and
`UnitDualQuat` from analytic elapsed time each redraw. Keep translation fixed at
`(0, -1, -4.6)` meters. Do not integrate products frame by frame. Headless
capture uses the fixed initial angle `pi/2`, so evidence is reproducible.

This is rigid turntable presentation only. Do not deform, morph, skin, break,
or otherwise alter the dragon geometry. Retain Stanford and McGuire attribution
and the noncommercial asset boundary.

## Consequences

Rotation speed is independent of frame rate and quaternion drift cannot
accumulate across frames. The shader uses the same pose for distance sampling
and normal rotation, and the matrix ban remains enforceable. Continuous object
motion invalidates any future temporal feature unless velocity/reprojection is
added, but the deterministic current-frame GI decision has no history.

## Alternatives

Incremental quaternion integration was rejected because elapsed error and
normalization drift would depend on redraw cadence. Rotating the camera was
rejected because the owner requested the dragon to rotate while scene framing
stays fixed. Vertex deformation was rejected because it is unrelated to rigid
presentation and inappropriate for this cultural reference asset.
