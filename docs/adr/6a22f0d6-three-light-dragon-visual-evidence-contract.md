---
id: 6a22f0d6-2667-47f8-a55c-1e4b143d616b
slug: adr:three-light-dragon-visual-evidence-contract
title: Three-light dragon visual evidence contract
status: superseded
supersedes: [adr:deterministic-visual-evidence-contract]
supersededBy: adr:deterministic-deferred-screen-space-radia
deciders: ["LairdWT"]
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context

The original visual-evidence contract proved one off-screen analytic emitter in
a synthetic courtyard. The mesh-derived dragon scene now has three visible
colored emitters, direct occlusion shadows, and a distinct RADIA indirect path.
The old single-emitter off-screen predicate no longer describes the shipped
demo.

## Decision

Replace the off-screen-emitter comparison with a fixed-state three-light
comparison. Capture Off and Radia with identical camera, dragon field digest,
three light definitions, direct-light equations, trace bounds, dimensions, and
adapter identity. Only the indirect mode and temporal sample sequence differ.

Require the Radia capture to change a declared subject-and-receiver ROI by at
least 4/255 while the direct shadows remain present in both images. Repeat the
same fixed-state Radia capture and require byte-identical decoded samples and
PNG hashes on one declared adapter. Record the dragon artifact SHA-256, all
three light colors and positions, Hammersley sequence version, sample count,
GPU adapter, driver, commands, output hashes, and comparison statistics.

Visual inspection remains an owner judgment. Mechanical gates prove
provenance, repeatability, controlled state, and numeric delta; they do not
self-certify artistic quality.

## Consequences

Evidence now matches the public demo and can distinguish direct-only Off from
direct-plus-indirect Radia without claiming that one off-screen source caused
the full delta. Existing analytic MVP evidence remains historical and is not
rewritten. New manifests use the mesh-derived dragon contract and must not be
compared as replacements for the old baseline.

## Alternatives

Keeping the old off-screen predicate was rejected because two additional
visible emitters can also contribute indirect radiance. Treating one plausible
image as proof was rejected because it does not establish deterministic state
or isolate the indirect mode. Requiring identical PNGs across different GPU
vendors was rejected because floating-point and driver differences are outside
the fixed-adapter determinism scope.
