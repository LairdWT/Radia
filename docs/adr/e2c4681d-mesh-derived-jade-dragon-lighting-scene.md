---
id: e2c4681d-f4bf-4aac-9fd3-23118b9ace8d
slug: adr:mesh-derived-jade-dragon-lighting-scene
title: Mesh-derived jade dragon lighting scene
status: superseded
supersedes: [adr:analytic-radia-mvp-boundary]
supersededBy: adr:deterministic-deferred-screen-space-radia
deciders: ["LairdWT"]
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context

The analytic courtyard proved the matrix-free RADIA path, but the public demo
now needs a recognizable production-shaped subject, three colored light
sources, and visible occlusion shadows. The requested subject is the Chinese
Dragon from the Stanford 3D Scanning Repository. Stanford permits attributed
research use and free redistribution, but the scan is not Apache-2.0 and may
not be used commercially without permission. The reconstructed scan also has
documented holes, so it cannot support an unqualified exact signed-distance
claim.

## Decision

Supersede the analytic-only scene boundary with a mesh-derived jade-dragon
scene. Use Morgan McGuire's OBJ conversion of the Stanford Chinese Dragon as
the reproducible source and verify its archive digest before baking. Commit a
deterministic 128 cubed single-channel f32 unsigned-distance volume derived
from the mesh, its generation metadata, attribution, and the Stanford Scan use
terms. Do not commit the source archive. The Apache-2.0 license continues to
cover project code only; the derived dragon artifact is separately governed by
the Stanford terms and is not approved for commercial use.

The runtime samples the mesh-derived unsigned-distance volume from a read-only
WGSL storage buffer and traces it conservatively as a surface field. It never
claims an exact signed field or interior CSG semantics. The dragon pose is a
normalized dual quaternion; asset scale and orientation are baked. No matrix
type or WGSL matrix is introduced.

Light the jade subject with three finite emissive spheres using red, green,
and blue radiance. Direct diffuse and bounded specular terms are evaluated in
linear light. Each light is visibility-tested with a bounded shadow trace
against the same scene field. RADIA remains the sole indirect producer: a
versioned deterministic cosine-weighted Hammersley sample traces scene
visibility and accumulates one emissive bounce into RGBA16Float history.

## Consequences

The demo gains a recognizable mesh-derived dragon, three colored area lights,
hard-to-soft finite-source shadows, and colored one-bounce indirect light while
preserving the quaternion-first, matrix-free renderer. A checked-in volume
increases repository size and must remain within default WGPU storage-buffer
limits. Runtime validation must prove host/WGSL binding parity and both Vulkan
adapters. Visual evidence must distinguish direct shadowing from RADIA's
indirect contribution.

Because the mesh is represented by a sampled unsigned field, thin features
below the voxel resolution can disappear and interior classification is not
available. README and debug views must label that limitation. Redistribution
must retain Stanford and McGuire attribution, and commercial users must replace
the asset or obtain Stanford permission.

## Alternatives

Keeping only analytic primitives was rejected because it does not satisfy the
requested dragon demo. Runtime brute-force triangle tracing was rejected as
incompatible with the source mesh size and the bounded demo schedule. Adding a
mesh-import or geometry-processing dependency was rejected because the current
dependency ADR admits only WGPU, Winit, and PNG. Raster shadow maps were
rejected for this iteration because three point-light cube maps would add a
second geometry and projection path; the shared distance field gives one
visibility contract for camera, direct shadows, and RADIA.
