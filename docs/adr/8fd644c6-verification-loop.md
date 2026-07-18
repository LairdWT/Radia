---
id: 8fd644c6-9a02-4787-87a3-a8f43625676f
slug: adr:verification-loop
title: Verification loop
status: accepted
supersedes: []
supersededBy: null
deciders: ["project owner"]
proposedAt: 2026-07-18
decidedAt: 2026-07-18
tags: []
---

## Context
Without a check an agent can run, 'looks done' is the only stop signal.

## Decision
The verification command is `cargo test --workspace --all-features`. A change is not done until it passes; unattended runs set it as a /goal condition.

## Consequences
Every task ends with evidence, not assertion.

## Alternatives
- Manual review as the only gate: does not scale to unattended agent runs. Rejected.
