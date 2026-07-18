# AEP evaluation notes

## 2026-07-18 scaffold

Command: `agent-code-skills scaffold` with the pinned Rust, graphics, math,
Git, ops, PowerShell, and shell bundles.

Findings:

- Graphics path rules omitted `**/*.wgsl`; Radia adds
  `.claude/rules/wgsl.md`.
- Math was pinned in `CODE.md` but had no path-scoped rule; Radia adds
  `.claude/rules/math.md` for its math and SDF ownership paths.
- `.claude/aep-skills.json` remains generated output. Project-local rules
  preserve the core-router-first contract without rewriting generator data.

These are consumer findings only. AEP generator changes are outside Radia's
write scope.

