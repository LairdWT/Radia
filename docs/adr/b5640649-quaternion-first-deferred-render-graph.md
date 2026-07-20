---
id: b5640649-4c9e-4ad7-8ef5-c830c270b17a
slug: adr:quaternion-first-deferred-render-graph
title: Quaternion-first deferred render graph
status: superseded
supersedes: []
supersededBy: adr:deterministic-deferred-screen-space-radia
deciders: ["LairdWT"]
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context

Radia currently performs primary distance-field tracing, direct and indirect
lighting, temporal accumulation, and history output in one full-screen fragment
pass. That proves the math and transport contracts but does not exercise a
production-shaped deferred boundary. The owner's private Legaia renderer proves
a useful structural pattern: named transients, fixed pass order, graph-owned
resizing, a four-color-target G-buffer under the adapter limit, a full-screen
lighting resolve, then temporal and presentation passes. Legaia uses different
scene data and matrix adapters, so it is design evidence only.

## Decision

Replace Radia's monolithic accumulation path with a fresh fixed deferred graph
whose registration rejects any pass reading a named transient before an earlier
pass writes it. The graph owns all extent-dependent textures and recreates them
together on resize.

The pass order is:

1. `gbuffer_geometry` writes `gbuffer_albedo` as `Rgba8Unorm`,
   `gbuffer_normal_material` as `Rgba16Float`, `gbuffer_emissive` as
   `Rgba16Float`, `gbuffer_trace` as `Rgba16Float`, and reverse-Z depth as
   `Depth32Float`.
2. `deferred_lighting` reads those five transients plus the immutable dragon
   field and frame uniform, then writes one linear-light HDR sample to
   `scene_radiance` as `Rgba16Float`.
3. `temporal_accumulation` reads `scene_radiance` and the previous
   `Rgba16Float` history target, then writes the next history target using the
   running Monte Carlo mean.
4. `presentation` reads the current history, tone-maps once, and writes the
   declared surface or capture format.

The geometry pass traces the existing mesh-derived dragon, receiver planes, and
three emissive spheres. A hit writes world-space normal, material identity,
emissive radiance, trace/debug values, and finite reverse-Z depth in `(0,1]`.
A miss leaves the zero depth clear and cleared G-buffer values. The lighting
pass reconstructs the camera-space ray from framebuffer pixel center, field of
view, aspect, and the frozen top-left origin rule. For depth `z_r` and normalized
camera ray direction `d`, camera-space distance is
`(-near / z_r) / d.z`; the camera dual quaternion rotates the ray and supplies
translation. No view, inverse-view, projection, or inverse-projection matrix
type or value is constructed or uploaded.

Direct lighting continues to visibility-test all three finite emitters against
the shared distance field. `Off` writes deterministic direct radiance;
`Radia` adds exactly one cosine-weighted secondary sample; `GiOnly` isolates
that sample. Debug modes consume G-buffer values. Triangle mode remains an
analytic matrix-free baseline emitted by the deferred stage. Camera, scene,
mode, light, and extent changes reset temporal history.

Use Legaia only for the structural facts recorded above. Do not copy its source,
identifiers, shader formulas, game data, assets, material model, matrices, or
feature-specific pass implementations. Velocity, TAA, SSAO, DFAO, SSGI, bloom,
transparent geometry, and full Legaia graph parity remain deferred.

## Consequences

Geometry and lighting become independently inspectable and reusable, while the
existing dual-quaternion and analytic projection contracts remain authoritative.
The graph adds five full-resolution textures beyond history and requires format,
binding, load/store, depth, resize, and read-before-write tests on both target
Vulkan adapters. Distance-field geometry is still traced once for primary
visibility and again only for shadows and the optional RADIA bounce.

The four G-buffer color targets consume the known simultaneous attachment
limit without exceeding it. World position is not stored; analytic depth
reconstruction saves one attachment and prevents a hidden matrix path. No
velocity target means history still resets on camera or scene changes rather
than reprojecting across them.

## Alternatives

Keeping the monolithic pass was rejected because it does not test a deferred
geometry/lighting contract. Copying the private Legaia graph was rejected by
the clean-room boundary and because its mesh, matrix, shadow-map, material, and
temporal needs differ. Storing world position was rejected because analytic
reverse-Z reconstruction is exact under Radia's camera contract and preserves
the four-target budget. Adding velocity/TAA and the full Legaia post stack now
was rejected because the requested reference slice is the quaternion-first
deferred foundation, not feature parity.
