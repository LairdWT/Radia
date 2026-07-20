---
id: eacacbd1-0d7e-4323-adaf-2a2282b1c37d
slug: adr:deterministic-deferred-screen-space-radia
title: Deterministic deferred screen-space RADIA
status: superseded
supersedes: [adr:quaternion-first-deferred-render-graph, adr:mesh-derived-jade-dragon-lighting-scene, adr:three-light-dragon-visual-evidence-contract]
supersededBy: adr:stratified-deferred-screen-space-radia-gather
deciders: ["LairdWT"]
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context

The current RADIA mode launches one cosine-weighted secondary scene ray per
pixel and frame, then averages those Monte Carlo samples in history. The owner
rejects the resulting path-tracing grain and artifacts and wants Radia to match
the structural separation of the private Legaia deferred renderer without
copying its code.

## Decision

Replace stochastic secondary transport and temporal history with deterministic
current-frame deferred screen-space irradiance. The fixed graph becomes:

1. `gbuffer_geometry` writes four surface targets and reverse-Z depth.
2. `deferred_direct_lighting` writes direct linear HDR radiance.
3. `screen_space_indirect` gathers the current direct-radiance and G-buffer
   surfaces at sixteen fixed symmetric offsets into indirect linear HDR.
4. `deferred_composite` selects direct, direct plus indirect, GI-only, or debug
   output and writes scene radiance.
5. `presentation` tone-maps scene radiance once.

The indirect pass uses current-frame integer texel loads only. It rejects
missing depth, non-finite or degenerate separation, back-facing receiver/source
pairs, and out-of-bounds taps. It performs no random sampling, secondary scene
trace, recursive bounce, previous-frame read, temporal accumulation, or
denoising. `Radia` means direct plus this deterministic screen-space
approximation; `GiOnly` isolates it; `Off` is direct only.

Primary camera visibility remains bounded distance-field ray marching because
the scene is represented by a mesh-derived unsigned-distance volume. Direct
shadow visibility to each finite light also remains bounded. Neither operation
is a path-traced light-transport bounce. Indeterminate primary traces are shown
only in the explicit hit-state debug mode; production modes render them as
background rather than magenta.

Current evidence renders one fixed frame per capture, repeats the RADIA capture
for exact determinism, and compares Off with Radia under identical state.
Manifests record `screen-space-irradiance-v1` and tap count instead of a Monte
Carlo sequence or sample count.

Use Legaia only for the public structural idea of named G-buffer, SSGI,
lighting, and presentation boundaries. Do not copy private source, formulas,
identifiers, matrices, data, or assets.

## Consequences

The demo becomes immediately stable with no convergence grain, ghost history,
or path-traced secondary ray. Moving rigid objects can be evaluated from the
current frame without reprojection. The graph adds direct and indirect
full-resolution `Rgba16Float` transients but deletes two history targets.

Screen-space irradiance is view-dependent and cannot gather off-screen or
occluded source surfaces. It is an intentional real-time approximation, not a
reference path-integrated result. Fixed symmetric taps can show bounded
screen-space bias, so depth, facing, and distance guards plus direct visual
evidence remain required.

## Alternatives

Keeping the Monte Carlo bounce and adding a denoiser was rejected because the
owner explicitly rejected path tracing. Copying Legaia SSGI was rejected by the
clean-room boundary. A radiance-probe volume was deferred because it needs a
new update/cache contract beyond this correction. Direct-only lighting was
rejected because it would remove Radia's visible indirect mode rather than
replace its transport method.
