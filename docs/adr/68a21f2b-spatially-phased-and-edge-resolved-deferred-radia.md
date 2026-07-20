---
id: 68a21f2b-e8a1-49b9-82fe-0f34712530b4
slug: adr:spatially-phased-and-edge-resolved-deferred-radia
title: Spatially phased and edge-resolved deferred RADIA
status: accepted
supersedes: [adr:stratified-deferred-screen-space-radia-gather]
supersededBy: null
deciders: []
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context

The thirty-two-tap stratified gather removed the two-ring alignment but failed
the visual gate: fresh fixed-pose AO and RADIA captures still show finer
horizontal bands. All pixels still use the same integer-offset phase, so planar
depth boundaries enter the kernel coherently by framebuffer row. Increasing
the fixed tap count alone does not remove that spatial coherence.

The owner requires a stable deferred result without path tracing, temporal
history, stochastic convergence, or visible grain.

## Decision

Retain the five-stage quaternion-first deferred graph and use deterministic
spatial phasing plus edge-aware reconstruction for indirect data:

1. `gbuffer_geometry` writes four surface targets and reverse-Z depth.
2. `deferred_direct_lighting` writes direct linear HDR radiance.
3. `screen_space_indirect` gathers current direct radiance and G-buffer data at
   thirty-two fixed antipodal radial strata. A fixed 4x4 screen-space phase
   tile rotates the offset set; the tile is compile-time data and does not vary
   by frame, clock, adapter, or run.
4. `deferred_composite` performs a bounded 5x5 edge-aware reconstruction of
   indirect radiance and diagnostic accessibility, then selects direct, direct
   plus indirect, GI-only, or debug output. Reconstruction weights use fixed
   spatial weights plus reverse-Z-relative depth and decoded-normal agreement.
5. `presentation` tone-maps physical radiance once and passes normalized debug
   values through unchanged.

Out-of-frame taps are rejected. The indirect gather retains trace-state,
finite-domain, depth, facing, and separation guards. The reconstruction rejects
out-of-frame and non-surface neighbors and gives zero weight across declared
depth or normal discontinuities. It never filters direct radiance or the other
G-buffer debug modes. Accessibility remains diagnostic `[0,1]` data and does
not modulate production lighting.

The phase tile is deterministic spatial stratification, not a random or
Monte Carlo sequence. The renderer still performs no secondary transport ray,
recursive bounce, previous-frame read, temporal accumulation, or stochastic
sample. The edge-aware reconstruction is a fixed current-frame deferred
resolve, not a temporal or learned denoiser.

Evidence records `screen-space-irradiance-v3`, 32 gather taps, the 4x4 phase
period, the 5x5 resolve width, exact repeat hashes, and identical-angle Off,
AO, and RADIA captures. Publication is gated on the reported bands being absent
at 640x360 and 1280x720 plus the existing full-turn curtain scan.

The clean-room boundary remains: no Legaia code, formula, identifier, data,
path, matrix, or asset is copied.

## Consequences

Row-coherent discontinuities become spatially interleaved before the bounded
edge-aware resolve, removing visible bands without temporal noise. Composite
now reads the normal/material and depth G-buffer attachments and performs up to
25 local indirect/normal/depth loads per active indirect pixel. Pass count and
transient count do not change.

Screen-space irradiance remains view-dependent and incomplete for off-screen or
occluded sources. The fixed phase period can create a bounded 4x4 signature if
the resolve is changed or disabled, so shader-contract tests and raw visual
evidence remain required.

## Alternatives

More unphased taps were rejected by the failed v2 capture. Per-frame noise and
temporal averaging were rejected because they reintroduce grain, history, and
motion-reset policy. A standalone denoising pass was rejected because the
existing composite can perform the bounded resolve without another transient
or pass. Direct-only lighting was rejected because it removes RADIA indirect
light instead of correcting it.
