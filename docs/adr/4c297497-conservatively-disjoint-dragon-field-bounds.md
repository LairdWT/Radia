---
id: 4c297497-3bc9-4c65-9553-11f575e48e2b
slug: adr:conservatively-disjoint-dragon-field-bounds
title: Conservatively disjoint dragon field bounds
status: accepted
supersedes: [adr:separated-dragon-triad-pbr-camera-and-ambient-fill]
supersededBy: null
deciders: ["LairdWT"]
proposedAt: 2026-07-19
decidedAt: 2026-07-19
tags: []
---

## Context

`adr:separated-dragon-triad-pbr-camera-and-ambient-fill` selected a 1.55-meter
origin radius from the dragon's local tail extent. That keeps each tail tip
away from the exact cluster center, but it does not prove that adjacent rotated
sampled-field bounds are disjoint. The embedded field has symmetric horizontal
local bounds `x = +/-0.6954665` and `z = +/-1.469422` meters. With three frames
rotated by 120 degrees, lateral extent contributes to the neighbor-facing
projection, so a stronger separating-axis contract is required to guarantee
that no two rigid UDF instances clip.

## Decision

Supersede the 1.55-meter radius with a 2.0-meter dragon-origin radius. Keep the
same cluster center, 120-degree phase separation, outward local `-Z` direction,
inward local `+Z` direction, and all other camera, PBR, lighting, and motion
choices from the superseded record.

Use the outward axis of either member of an adjacent pair as a conservative
separating axis. At radius `r`, adjacent centers project `1.5*r` meters apart
on that axis. The first oriented field bound projects 1.469422 meters. The
second projects
`1.469422*abs(cos(120 degrees)) + 0.6954665*abs(sin(120 degrees))`,
approximately 1.337004 meters. At `r=2.0`, center separation is 3.0 meters and
the summed conservative radii are approximately 2.806426 meters, leaving more
than 0.193 meters of projected clearance. Because every sampled dragon surface
is contained by its rigidly transformed field bounds, the separating axis
proves all three pairwise bounds are disjoint at every shared cluster phase.

Add a CPU test that derives this inequality from embedded asset metadata and
the 120-degree layout. Retain multi-angle visual capture to verify framing and
presentation, but do not use an image as the non-clipping proof.

## Consequences

All three sampled fields are conservatively disjoint rather than merely having
tail-tip clearance at the center. The tails still point toward the same middle,
but their nearest possible field support remains separated. The composition is
wider; the accepted 65-degree camera must therefore remain a capture and live
demo gate. Future asset-bound changes fail the derived clearance test rather
than silently reintroducing clipping.

## Alternatives

Keeping 1.55 meters was rejected because center clearance alone ignores lateral
extent after a 120-degree rotation. A radius of approximately 1.871 meters is
the mathematical threshold, but it was rejected because it leaves no useful
margin for stored-bound rounding. Per-angle collision tests on sampled surface
points were rejected as the primary contract because finite sampling cannot
prove absence of intersection. Scaling the dragons was rejected because scale
is outside the rigid dual-quaternion representation.
