---
id: 77c5dd21-4686-44b5-83a2-7f1dc5e0f637
slug: adr:separated-dragon-triad-pbr-camera-and-ambient-fill
title: Separated dragon triad PBR camera and ambient fill
status: superseded
supersedes: [adr:radial-dragon-triad-and-orbital-light-motion]
supersededBy: adr:conservatively-disjoint-dragon-field-bounds
deciders: ["LairdWT"]
proposedAt: 2026-07-19
decidedAt: 2026-07-19
tags: []
---

## Context

The accepted radial-triad scene places each dragon origin only 0.78 meters from
the shared center. The baked dragon field reaches approximately 1.469 meters
toward its local tail side, so neighboring rigid instances overlap and can clip
at some cluster phases. The default camera is level, close to the floor, and
uses a 60-degree vertical field of view. All dragon instances also share one
green Blinn-Phong-like material, while unlit surfaces receive only a 0.006
albedo term. The owner requires a separated triad, a 65-degree quaternion-only
camera looking down at the shared center by about 30 degrees, cyan/magenta/yellow
materials with roughness 0.2/0.65/0.9, one metallic dragon, and visible ambient
or bounce illumination.

The renderer remains right-handed, matrix-free, normalized-dual-quaternion
based, deterministic at an explicit scene time, and bounded by the accepted
deferred RADIA and sampled-UDF decisions. Screen-space RADIA remains the only
geometry-derived indirect-light producer; a separate analytic environment fill
must not be presented as traced global illumination.

## Decision

Supersede `adr:radial-dragon-triad-and-orbital-light-motion` only where this
record changes placement and presentation. Keep its analytic phase, outward
orientation, cluster center, and light motion, but set the dragon-origin radius
to 1.55 meters. This exceeds the baked field's 1.4694215-meter positive local-Z
tail extent by more than 0.08 meters and places adjacent origins
`sqrt(3) * 1.55` meters apart. Fixed-angle captures remain the visual authority
for inter-instance surface clearance because the sampled UDF is not a convex
bound.

Set the default camera vertical field of view to exactly 65 degrees. Place the
camera at `(0, 3.0, 2.3282032)` meters and rotate it actively by -30 degrees
about world +X, so camera-local -Z points at the shared cluster center
`(0, -1, -4.6)`. Construct the pose only as a normalized unit quaternion plus
translation and preserve analytic matrix-free projection.

Give the three instances distinct material identifiers in radial array order.
Use scene-linear RGB base colors obtained from the project parameterization
`S=0.55, V=0.65`: cyan `(0.2925, 0.65, 0.65)`, magenta
`(0.65, 0.2925, 0.65)`, and yellow `(0.65, 0.65, 0.2925)`. Set perceptual
roughness to 0.2, 0.65, and 0.9 respectively. Set only the cyan instance to
metallic 1.0; the other two use metallic 0.0.

Replace the dragon-specific gloss term with a bounded GGX microfacet BRDF using
Schlick Fresnel, Smith-Schlick masking-shadowing, `alpha=roughness^2`, dielectric
`F0=0.04`, and the standard metallic base-color interpolation. Clamp all cosine
terms to their physical domains and guard the half-vector and BRDF denominators
with operation-scale constants. The proof basis is PBRT 4e, Roughness Using
Microfacet Theory:
`https://www.pbr-book.org/4ed/Reflection_Models/Roughness_Using_Microfacet_Theory`.

Add deterministic, unoccluded sky/ground environment fill to the direct pass.
Use fixed nonnegative linear irradiance colors, weight them by the surface
normal's up/down hemispherical orientation, and evaluate diffuse response plus
a roughness- and Fresnel-bounded specular environment term. This is an analytic
baseline approximation, not visibility-aware transport. The existing RADIA
pass continues to provide current-frame, geometry-derived indirect radiance and
ambient accessibility, so `Off`, `Radia`, and `GiOnly` retain distinct meanings.

## Consequences

The three rigid fields no longer intentionally occupy the center region, the
camera shows the whole composition from above, and each dragon exposes visibly
different PBR response under the moving emitters. Metalness has a meaningful
effect rather than being a display-only label. Shadow rays and deterministic
RADIA remain unchanged, while surfaces outside direct emitter visibility are no
longer nearly black.

The material identifier range grows from six to eight values, so every shader
consumer, debug palette, and source-contract test must recognize all dragon
identifiers. The environment fill is deliberately cheap and unoccluded; it can
brighten crevices, so RADIA ambient accessibility remains the separate bounded
geometry signal. Visual captures at multiple fixed cluster phases, both Vulkan
adapters, the matrix ban, and deterministic repeat hashes remain release gates.

## Alternatives

Scaling the dragons down was rejected because non-rigid scale is outside the
project's dual-quaternion contract. Moving only the camera was rejected because
it would hide rather than remove inter-instance overlap. A look-at matrix was
rejected because project code and shaders ban matrices. Keeping one material or
using encoded sRGB constants was rejected because it would not satisfy the
requested per-instance material study in the scene-linear renderer. Full
image-based lighting, irradiance probes, ray-traced bounce, and path tracing
were rejected for this bounded demo change; they add assets, state, or sampling
noise and would blur the existing RADIA evidence boundary.
