---
id: fae4cf0e-147a-4aec-935f-2dc90a68a04e
slug: adr:stratified-deferred-screen-space-radia-gather
title: Stratified deferred screen-space RADIA gather
status: superseded
supersedes: [adr:deterministic-deferred-screen-space-radia]
supersededBy: adr:spatially-phased-and-edge-resolved-deferred-radia
deciders: []
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context

The deterministic deferred RADIA pass uses sixteen symmetric integer taps on
two screen-space rings. At planar floor and wall boundaries, many taps cross
the same depth discontinuity on the same framebuffer rows. The resulting hard
validity transitions appear as repeated horizontal bands in both the
accessibility view and RADIA composite. An identical `Off` capture preserves
the direct-shadow shapes but does not contain the repeated gather bands.

The owner wants the artifacts removed without returning to path tracing,
stochastic sampling, temporal history, or denoising.

## Decision

Keep the quaternion-first deferred graph and replace only the sparse two-ring
gather with deterministic current-frame screen-space irradiance v2:

1. `gbuffer_geometry` writes four surface targets and reverse-Z depth.
2. `deferred_direct_lighting` writes direct linear HDR radiance.
3. `screen_space_indirect` gathers current direct radiance and G-buffer data at
   thirty-two fixed, antipodal, radially stratified integer offsets into
   indirect linear HDR plus diagnostic accessibility.
4. `deferred_composite` selects direct, direct plus indirect, GI-only, or debug
   output and writes scene radiance.
5. `presentation` tone-maps physical radiance once and passes normalized debug
   values through unchanged.

The sixteen base offsets follow monotonically increasing square-root radial
strata and a golden-angle azimuth; each has an exact antipodal partner. The
offsets are compile-time constants, not per-frame or per-pixel random values.
This distributes vertical crossings across distinct rows while retaining
determinism and directional balance.

Out-of-frame taps are rejected instead of clamped so an edge texel cannot be
counted repeatedly. Depth, trace-state, facing, separation, and finite-domain
guards remain. Accessibility is normalized for thirty-two taps and remains a
human-readable diagnostic in `[0,1]`; it does not modulate production light.

The pass still performs no random sampling, secondary scene trace, recursive
bounce, previous-frame read, temporal accumulation, or denoising. `Radia`
means direct plus this deterministic screen-space approximation; `GiOnly`
isolates it; `Off` is direct only. Primary and direct-shadow visibility remain
bounded distance-field traces and are not path-traced transport bounces.

Evidence records `screen-space-irradiance-v2`, a gather count of 32, exact
fixed-state repeat hashes, AO and RADIA captures at the reported angle, and an
identical-state Off control.

The clean-room boundary remains: Legaia informs only named structural stages.
No private formula, identifier, code, data, path, matrix, or asset is copied.

## Consequences

The gather doubles texture-load work relative to v1 but remains a bounded
single full-resolution pass with no history resources. More radial and vertical
strata remove the coherent two-ring band pattern while preserving stable output
and exact repeatability.

Screen-space irradiance remains view-dependent and cannot gather off-screen or
occluded sources. Some bounded screen-space bias remains possible, so the Off,
AO, RADIA, and full-turn evidence set is required before publication.

## Alternatives

A bilateral denoising pass was rejected because it adds a graph stage, hides
rather than removes the sparse sampling structure, and violates the no-denoiser
boundary. Per-pixel random or blue-noise rotation was rejected because it can
reintroduce visible grain. Temporal accumulation was rejected because it adds
history and motion-reset policy. Direct-only lighting was rejected because it
removes the named RADIA indirect mode rather than correcting it.
