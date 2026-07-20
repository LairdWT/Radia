---
id: 82e613c0-8773-43fd-b16c-906f99382713
slug: adr:inspectable-deferred-buffer-contract
title: Inspectable deferred buffer contract
status: superseded
supersedes: []
supersededBy: adr:inspectable-deferred-buffer-and-telemetry-contract
deciders: ["LairdWT"]
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context

The owner captured a thin black plane beside the rotating dragon. The bounded
primary tracer reports hit, miss, and indeterminate states, but the G-buffer
currently writes depth for both hit and indeterminate results. The deferred
pass then renders an indeterminate result as dark background at that nonzero
depth, producing a view-dependent tangent curtain. The owner also requires
human-readable views of the deferred buffers, including depth and ambient
occlusion, with bounded display values.

## Decision

Make the deferred frame inspectable through explicit display modes while
preserving the existing five-pass graph and four-color-target G-buffer.

Only `TRACE_HIT` may write reverse-Z depth or surface attributes. Miss and
indeterminate results retain trace state and step telemetry in
`gbuffer_trace`, but leave depth at the reverse-Z clear value. Hit-state and
step-count modes read trace telemetry before rejecting clear depth, so an
indeterminate ray remains diagnosable without becoming scene geometry.

Add display modes for albedo, bounded emissive preview, linear depth, and
ambient occlusion. Existing Off and GiOnly modes remain the direct-radiance and
indirect-radiance views. Debug mappings are:

- albedo: stored linear RGB, already in `[0,1]`;
- normal: world normal remapped from `[-1,1]` to `[0,1]`;
- emissive: component-wise `c / (1 + c)` for nonnegative linear HDR input;
- linear depth: camera distance divided by the declared maximum trace distance,
  clamped to `[0,1]`, with miss and indeterminate equal to `1`;
- ambient occlusion: deterministic screen-space accessibility in `[0,1]`,
  where `0` is fully occluded and `1` is unoccluded.

Store ambient accessibility in the alpha channel of the existing
`indirect_radiance` transient. Compute it from the same sixteen current-frame
integer texel offsets using finite depth, range, and facing guards. It is a
diagnostic screen-space approximation and does not modulate production direct
or indirect radiance in this change.

Presentation tone-maps only physical radiance modes. It copies normalized
debug values without tone mapping so displayed black and white remain the
declared 0 and 1 endpoints. Space continues to cycle every mode; the CLI gains
matching lowercase mode names, and the window title identifies the active
view.

## Consequences

Step-capped tangent rays can no longer occlude, shade, or feed screen-space
gathers as false geometry. The explicit hit-state view still exposes them.
Users can inspect surface, trace, depth, radiance, and AO-like accessibility
without GPU capture tools or hidden transfer functions.

The ambient-occlusion view is screen-space, view-dependent, and incomplete at
screen edges or behind visible surfaces. It is not a signed-distance ambient
visibility solve and is not applied to final lighting. Bounded emissive is a
preview mapping, not raw radiometric readback. Raw attachment evidence remains
the authority for numeric inspection.

## Alternatives

Writing depth for indeterminate rays and changing only their color was rejected
because false geometry would continue to contaminate depth consumers and
screen-space gathers. Adding a fifth G-buffer attachment was rejected because
the declared simultaneous color-target limit is four. Adding a separate AO
texture and pass was rejected because the existing indirect target has an
unused alpha channel and the requested diagnostic does not yet affect final
lighting. Tone-mapping every debug view was rejected because it would map a
declared value of `1` to `0.5` and make the display contract false.
