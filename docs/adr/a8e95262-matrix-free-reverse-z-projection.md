---
id: a8e95262-6645-48f8-b454-57a614b29ff6
slug: adr:matrix-free-reverse-z-projection
title: Matrix-free reverse-Z projection
status: accepted
supersedes: []
supersededBy: null
deciders: ["LairdWT"]
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context

WGPU clip depth is zero to one, while camera space is right-handed with
forward along negative Z. Radia needs project, unproject, and screen rays
without constructing projection or inverse-view matrices.

Normative API basis: WebGPU and WGSL specifications,
https://gpuweb.github.io/gpuweb/ and https://gpuweb.github.io/gpuweb/wgsl/.

## Decision

`ReverseZPerspective { vertical_fov, aspect, near }` validates finite positive
aspect and near values and a vertical field of view strictly between zero and
pi. It maps the near plane to depth one and infinite distance toward zero.

For camera-space point `(x,y,z)` with `z < 0`, compute
`clip_x = x / (tan(fov_y/2) * aspect)`,
`clip_y = y / tan(fov_y/2)`, `clip_z = near`, and `clip_w = -z`.
NDC is `clip.xyz / clip_w`. Projection refuses points on or behind the camera
and non-finite results.

Framebuffer origin is top-left and pixel centers are `integer + 0.5`. A screen
ray maps the pixel center to NDC X in `[-1,1]`, flips framebuffer Y into NDC Y,
builds camera direction `(ndc_x*tan*aspect, ndc_y*tan, -1)`, normalizes it, then
rotates origin and direction by the camera rigid pose.

No projection, view, inverse-view, or inverse-view-projection matrix type or
value is permitted in v1 project code or WGSL.

## Consequences

CPU and WGSL use the same formulas and boundary tests. Reverse-Z depth compare
uses greater semantics and clears depth to zero where a depth attachment is
used. Infinite-far projection cannot represent a finite far clipping plane;
ray and sphere-trace bounds own visibility termination.

## Alternatives

Conventional finite perspective matrices and inverse-matrix unprojection were
rejected by the matrix-free contract. Forward-Z was rejected because the MVP
freezes reverse-Z. A left-handed positive-Z camera was rejected because it
would conflict with the chosen world/view basis.
