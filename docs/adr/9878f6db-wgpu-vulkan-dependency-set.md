---
id: 9878f6db-9097-41c5-837b-d2a99959851b
slug: adr:wgpu-vulkan-dependency-set
title: WGPU Vulkan dependency set
status: accepted
supersedes: []
supersededBy: null
deciders: ["LairdWT"]
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context

The MVP needs a safe Rust GPU API, cross-platform window/event integration,
and deterministic PNG capture before the Build Week deadline. Implementing
Vulkan bindings, platform windows, or DEFLATE/PNG correctly in-project would
expand risk and scope.

Exact dependency metadata and licenses were reviewed for WGPU 26.0.1, Winit
0.30.13, and PNG 0.18.1. WGPU and Winit are dual MIT/Apache-2.0; PNG is
MIT/Apache-2.0. Lockfile and Cargo metadata remain the machine proof.

## Decision

Approve exactly:

- `wgpu = 26.0.1`, default features disabled, with only `std`, `wgsl`, and
  `vulkan` enabled.
- `winit = 0.30.13` for window lifecycle and raw window handles.
- `png = 0.18.1` for lossless RGBA capture.

WGPU instance creation requests only the Vulkan backend. No direct `ash`,
`pollster`, math, byte-cast, image, logging, or random dependency is approved.
Small semantic vectors, a minimal standard-library future executor, explicit
byte packing, and deterministic sampling are project code.

CI pins the AEP source commit used by this scaffold rather than consuming a
moving branch. Cargo.lock is committed.

## Consequences

WGPU/Naga API and validation claims must use the locked version. Native runs
require a Vulkan 1.3-capable loader and adapter. Linux window features may add
platform transitive crates. Dependency additions or feature expansion require
a new accepted build-vs-buy ADR.

## Alternatives

Direct `ash` was rejected because unsafe Vulkan lifecycle code cannot earn its
verification cost before the deadline. SDL and GLFW were rejected because
Winit already supplies the required handles and event loop. A custom PNG
encoder was rejected because PNG filtering, chunk CRC, and compression are not
project value. `pollster`, `bytemuck`, `glam`, and `rand` were rejected because
their narrow required behavior is small and testable in the standard library.
