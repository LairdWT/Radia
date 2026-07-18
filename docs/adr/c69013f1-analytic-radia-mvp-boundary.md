---
id: c69013f1-d557-4949-b84b-dee10214f404
slug: adr:analytic-radia-mvp-boundary
title: Analytic RADIA MVP boundary
status: accepted
supersedes: []
supersededBy: null
deciders: ["LairdWT"]
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context

Radia is a new AEP reference project and a bounded extension to the existing
Build Week AEP submission. It needs an end-to-end global-illumination claim
that can be proved on synthetic content without private assets or production
engine infrastructure.

## Decision

The MVP is a matrix-free WGPU fullscreen renderer over a synthetic courtyard:
analytic exact sphere, box, and plane SDFs; rigid dual-quaternion placement;
CSG union; bounded sphere tracing; analytic normals; primitive and material
IDs; an emissive orb; and a receiver floor/wall.

One indirect diffuse bounce uses a versioned deterministic Hammersley sequence
with cosine-weighted hemisphere samples. The emitter contributes through
visibility, not a proxy point light. A running Monte Carlo mean accumulates in
ping-pong RGBA16Float history and resets on camera, scene, emitter, or mode
change.

Required views are Off, Radia, GiOnly, SDF distance, primitive ID, normal,
step count, and hit/miss/indeterminate. Lighting stays linear until one tone
map and one sRGB encode. A deterministic headless path captures PNG output.

## Consequences

One renderer and one indirect producer own the claim. CPU reference tests and
GPU shader behavior share conventions, thresholds, trace limits, and terminal
states. The scene has no imported assets or hidden direct-light substitute.

## Alternatives

Mesh SDF bake, clipmaps, Surface Cache, radiance probes, multi-bounce, editor,
ECS, physics, skinning, and production content were deferred. A triangle/cube
baseline is a bring-up step, not the final MVP proof. Raster-only lighting was
rejected because it cannot prove the RADIA path.
