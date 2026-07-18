---
id: 574d036a-361a-43ad-b178-f71c9abe58ca
slug: adr:project-layout-and-strictness
title: Project layout and strictness
status: accepted
supersedes: []
supersededBy: null
deciders: ["project owner"]
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context
The AEP layer (CLAUDE.md, CODE.md, docs/adr/, .claude/) needs a declared enforcement posture.

## Decision
Strictness is gated: hooks and checks block. ADRs are uuid-named (never sequential). Agent transients live under Temp/ only.

## Consequences
The posture can be raised by a superseding ADR when concurrency or audit pressure demands it (conventions -> gates -> locks -> services).

## Alternatives
- Day-one service-mediated governance: top-rung machinery without the pressure that justifies it. Rejected.
