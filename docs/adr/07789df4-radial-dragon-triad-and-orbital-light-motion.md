---
id: 07789df4-262a-4fcd-b42a-e34e9f65daa5
slug: adr:radial-dragon-triad-and-orbital-light-motion
title: Radial dragon triad and orbital light motion
status: superseded
supersedes: [adr:rigid-dragon-turntable-presentation]
supersededBy: adr:separated-dragon-triad-pbr-camera-and-ambient-fill
deciders: ["LairdWT"]
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context

The interactive renderer currently animates one dragon pose while the green and
blue emitters remain shader literals. The owner requires three copies of the
same attributed dragon, arranged with their tails at a shared center and their
heads pointing outward, plus three colored emitters that orbit at different
speeds and bob vertically with sine waves.

The scene remains right-handed, meter-based, radian-based, active-Hamilton,
normalized-dual-quaternion, matrix-free, deterministic at an explicit time,
and bounded by the existing sampled-UDF and deferred-RADIA contracts.

## Decision

Supersede `adr:rigid-dragon-turntable-presentation` with a radial triad and an
explicit analytic scene clock. Let the shared cluster center be
`c = (0, -1, -4.6)` meters, the dragon-origin radius be `r = 0.78` meters, and
the cluster phase be `phi(t) = pi/2 + 0.12*t` radians. For dragon `i` in
`{0,1,2}`, let `alpha_i = phi(t) + i*2*pi/3`, place its origin at
`c + r*(cos(alpha_i), 0, sin(alpha_i))`, and use world `+Y` yaw
`-alpha_i - pi/2`. The baked dragon's local `-Z` head direction therefore
points radially outward while local `+Z` and the tail side face the shared
center. Rebuild all three normalized `UnitDualQuat` poses analytically from
finite elapsed seconds; do not integrate pose products frame by frame.

Represent each emitter as finite position, positive radius, nonnegative linear
RGB radiance, and nonnegative intensity. For emitter `j`, use
`theta_j(t) = phase_j + omega_j*t`, horizontal position
`(R_j*cos(theta_j), z_center + R_j*sin(theta_j))`, and height
`y_j(t) = base_j + amplitude_j*sin(bob_phase_j + bob_speed_j*t)`.
Freeze these contracts, in red/green/blue order:

- red: `R=3.1`, `omega=0.32`, `phase=pi`, `base=1.30`, `amplitude=0.55`,
  `bob_speed=0.77`, `bob_phase=0`, radius `0.24`, intensity `26`;
- green: `R=3.0`, `omega=-0.23`, `phase=0`, `base=1.65`, `amplitude=0.65`,
  `bob_speed=0.63`, `bob_phase=2*pi/3`, radius `0.25`, intensity `22`;
- blue: `R=2.5`, `omega=0.17`, `phase=pi/2`, `base=2.40`,
  `amplitude=0.75`, `bob_speed=0.49`, `bob_phase=4*pi/3`, radius `0.27`,
  intensity `24`.

Store all three dragon poses and all three light position/radius and
color/intensity records in the frame uniform. WGSL receives quaternion
components explicitly as `xyzw`; CPU semantic storage remains `wxyz`. Carry a
dragon instance index through distance tracing so the G-buffer normal is
computed in the correct instance frame. Keep the shared UDF payload immutable;
there is no scale, deformation, skinning, copied mesh payload, or matrix path.

Windowed rendering evaluates the analytic scene at monotonic elapsed seconds.
Headless capture defaults to `t=0` and accepts an explicit finite scene time;
an optional explicit cluster angle remains available for angle regressions.
Every repeated capture at the same time, angle, mode, camera, and extent must
remain byte-identical.

## Consequences

The composition retains inward tails and outward heads while the whole triad
turns slowly. The three emitters visibly orbit and bob without frame-rate
dependent drift. Each scene-distance query may evaluate up to three rigid
instances of the same field, so GPU timing and both Vulkan adapters remain
release gates. The larger 304-byte uniform must have asserted host/WGSL offsets
and minimum binding size.

The screen-space indirect method remains current-frame, deterministic, and
non-path-traced. Moving emitters and instances invalidate any future temporal
history, but the accepted renderer has no temporal accumulation.

## Alternatives

Three independently spinning dragons were rejected because their tails would
not remain centered. Frame-by-frame quaternion multiplication was rejected
because redraw cadence would affect drift. Shader-clock animation was rejected
because headless evidence and CPU/GPU contracts need one explicit time owner.
Hard-coded green and blue shader emitters were rejected because host evidence
could not describe or freeze the complete moving scene. Scaling the dragons
down was rejected by the rigid-transform contract.
