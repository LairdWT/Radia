---
id: a376627c-eebf-471f-9427-f4ffcda2bfa6
slug: adr:inspectable-deferred-buffer-and-telemetry-contract
title: Inspectable deferred buffer and telemetry contract
status: accepted
supersedes: [adr:inspectable-deferred-buffer-contract]
supersededBy: null
deciders: ["LairdWT"]
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context

The owner captured a thin black plane beside the rotating dragon. The bounded
primary tracer reports hit, miss, and indeterminate states, but the G-buffer
currently writes production depth for indeterminate results. Deferred shading
renders that non-surface dark, producing a view-dependent tangent curtain.
The owner also requires human-readable views of deferred data, including depth
and ambient occlusion, with bounded display values.

The first inspectable-buffer decision required miss and indeterminate trace
telemetry to survive while depth remained at the reverse-Z clear value. That is
not implementable in the existing single pass: the accepted `Greater` depth
comparison rejects a fragment whose depth equals the zero clear value, so its
color attachments cannot retain trace telemetry.

## Decision

Supersede `adr:inspectable-deferred-buffer-contract`. Make the deferred frame
inspectable through explicit display modes while preserving the five-pass graph,
four-color-target G-buffer, and greater reverse-Z comparison.

In physical and surface-buffer modes, only `TRACE_HIT` writes nonzero reverse-Z
depth or surface attributes. Miss and indeterminate rays write the clear depth
and therefore cannot become geometry, occlude, shade, or feed screen-space
gathers.

In the explicit HitState and StepCount modes only, the geometry shader writes a
depth-one diagnostic sentinel for non-hit rays so `gbuffer_trace` survives the
greater comparison. Deferred shading consumes trace state or normalized step
count before any surface reconstruction. The sentinel represents telemetry,
not scene depth, and never appears in Radia, Off, GiOnly, depth, AO, albedo,
normal, emissive, SDF-distance, material-ID, or triangle modes.

Add display modes for albedo, bounded emissive preview, linear depth, and
ambient occlusion. Existing Off and GiOnly remain direct-radiance and
indirect-radiance views. Debug mappings are:

- albedo: stored linear RGB, already in `[0,1]`;
- normal: world normal remapped from `[-1,1]` to `[0,1]`;
- emissive: component-wise `c / (1 + c)` for nonnegative linear HDR input;
- linear depth: camera distance divided by the maximum trace distance, clamped
  to `[0,1]`, with miss and indeterminate equal to `1`;
- ambient occlusion: deterministic screen-space accessibility in `[0,1]`,
  where `0` is fully occluded and `1` is unoccluded.

Store ambient accessibility in the alpha channel of the existing
`indirect_radiance` transient. Compute it from the same sixteen current-frame
integer texel offsets using finite depth, range, and facing guards. It is a
diagnostic screen-space approximation and does not modulate production direct
or indirect radiance in this change.

Presentation tone-maps only physical radiance modes. It copies normalized debug
values without tone mapping so displayed black and white remain the declared 0
and 1 endpoints. Space cycles every mode; CLI capture accepts matching lowercase
names and an optional finite dragon angle for exact reproduction.

## Consequences

Step-capped tangent rays cannot contaminate production depth. HitState and
StepCount still expose their full-screen classifications using an intentional,
mode-local sentinel. Users can inspect surface, trace, depth, radiance, and
screen-space accessibility without external GPU capture tools.

Ambient occlusion remains view-dependent and incomplete at screen edges or
behind visible surfaces. It is not a signed-distance visibility solve and is not
applied to final lighting. Bounded emissive is a preview mapping, not raw
radiometric readback. Raw attachment capture remains numeric authority.

## Alternatives

Changing depth comparison to `Always` was rejected because the accepted
reverse-Z contract requires greater semantics. Adding a trace-only pass was
rejected because it would expand the fixed graph and retrace the scene. Adding a
fifth G-buffer target was rejected because the attachment limit is four. Losing
indeterminate telemetry was rejected because the explicit hit-state mode is the
declared diagnostic for bounded traces. Tone-mapping every debug value was
rejected because it maps a declared value of `1` to `0.5`.
