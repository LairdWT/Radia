---
id: e47edcb6-7e3e-4b58-82c7-6c13cef82d6c
slug: adr:language-set-and-toolchain
title: Language set and toolchain
status: accepted
supersedes: []
supersededBy: null
deciders: ["project owner"]
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context
The project needs a declared language set so standards, rules, and tooling bind to something stable.

## Decision
Languages: rust, graphics, math, git, ops, powershell, shell. Conventions bind by reference through CODE.md pins to the agent-enhanced-project bundles.

## Consequences
Adding a language is a superseding decision plus a CODE.md pin, not an ad hoc drift.

## Alternatives
- Unpinned per-project standards documents: proven to bloat and drift. Rejected.
