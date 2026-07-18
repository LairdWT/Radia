# Radia agent bootstrap

- `CODE.md` is binding. Before changes read it, relevant accepted `docs/adr/` decisions, and matching path rules.
- Languages: rust, graphics, math, git, ops, powershell, shell. Load only the matching `*-core` router, then only spokes it selects.
- Verify before claiming success: `cargo test --workspace --all-features`.
- Lint/format: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Noisy commands: `agent-code-skills logstrip run -- <cmd>`.
- Never edit accepted ADR history; supersede it. No third-party dependency without owner approval and a build-vs-buy ADR.
- Use explicit boundary types, logged negative guards, small capability interfaces, composition, and one state owner.
- Nontrivial work follows `planning:plan-authoring`; keep transients under `Temp/`.
- No decisions in source comments, secret leakage, hook bypasses, or unsupported success claims.
