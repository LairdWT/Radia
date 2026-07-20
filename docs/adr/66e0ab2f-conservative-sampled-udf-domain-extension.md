---
id: 66e0ab2f-1ccb-4863-8e46-28adbe77b657
slug: adr:conservative-sampled-udf-domain-extension
title: Conservative sampled UDF domain extension
status: accepted
supersedes: []
supersededBy: null
deciders: []
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context

At dragon angles near 170 and 190 degrees, production, Off, depth, primitive,
SDF, step-count, and hit-state captures expose a thin vertical curtain. The
hit-state view classifies the curtain as `TRACE_INDETERMINATE`, not geometry.

The sampled dragon UDF returns `max(outside_distance, 2 * error)` for every
point outside its storage bounds without sampling the clamped boundary point.
After the runtime error subtraction, rays advance by roughly one field-error
unit while crossing the rotated volume. The 2.94 meter-long volume can then
exhaust the 128-step bound. The field has at least 0.06944 meters of boundary
clearance and a declared 0.01527 meter conservative error, so discarding the
boundary samples is the defect.

## Decision

Extend the sampled unsigned-distance field outside its axis-aligned local
domain with a conservative lower bound. For local point `p`, clamped boundary
point `q`, outside distance `b = length(p - q)`, sampled boundary distance `s`,
and declared sample error `e`, use:

`safe(p) = max(max(b, s - b) - e, 0)`

The mesh lies inside the sampled domain, so its exact distance from `p` is at
least `b`. Exact distance is 1-Lipschitz, so it is also at least
`distance(q, mesh) - b`; with sample error, `s - e - b` is a lower bound.
Taking the maximum of these two lower bounds remains conservative. At the
domain boundary `b = 0`, the expression retains the boundary field value
instead of collapsing to an error-sized shell.

Continue trilinear sampling at `q`, retain the existing error subtraction,
rigid inverse dual-quaternion placement, trace guard, and bounded hit/miss/
indeterminate states. Do not classify the storage AABB as geometry and do not
increase the trace step limit to mask the defect.

Add source-contract coverage for the extension formula and asset coverage that
all six volume faces retain clearance greater than the declared error. Capture
Radia, Off, HitState, PrimitiveId, SDF distance, step count, and depth at 170
degrees, then scan the full rigid turntable.

## Consequences

Rays cross empty parts of the rotated volume using a proven lower bound rather
than an error-sized constant step. The false indeterminate curtain disappears
without changing the dragon mesh, field payload, trace limit, camera, or rigid
pose semantics.

The extension is conservative, not exact. Close to a domain corner it can take
smaller steps than the true distance, but it cannot intentionally step farther
than the proven bounds permit.

## Alternatives

Raising the 128-step limit was rejected because it hides the invalid extension
and increases all trace costs. Treating indeterminate pixels as the wall was
rejected because it invents geometry and breaks trace telemetry. Expanding or
rebaking the volume was rejected because the existing field already has valid
boundary clearance; the runtime discarded it. Using `s + b` was rejected
because it is an upper bound through `q`, not a safe sphere-tracing step.
