# Build Radia as AEP's quaternion-first reference project

## Context

User decisions (2026-07-18): create public Apache-2.0 `LairdWT/Radia`; Rust
1.95; WGPU 26.0.1 with Vulkan only; WGSL; rigid transforms represented only
by normalized dual quaternions; matrix-free v1; analytic SDF one-bounce RADIA
MVP; existing AEP Build Week submission receives this new evidence.

## Evidence

- `Cargo.toml:1` defines a three-crate Rust 2024 workspace.
- `AGENTS.md:1` makes core-router-first skill loading binding.
- `CODE.md:1` pins Rust, graphics, math, Git, ops, PowerShell, and shell.
- `docs/aep-evaluation.md:1` records scaffold routing gaps and local fixes.

## Steps

1. Refresh AEP runtime and prove all installed bundle versions match the
   canonical clean checkout. Gate: passed before project edits.
2. Scaffold, verify, commit, create the public repo, and push the bootstrap.
3. Accept ADRs for math conventions, projection, dependencies, MVP scope,
   lineage, and visual evidence before dependency or runtime code lands.
4. Implement and property-test `radia-math`.
5. Implement Vulkan-only WGPU lifecycle, matrix-free analytic projection,
   deterministic headless capture, and windowed display.
6. Implement analytic SDF geometry, bounded sphere tracing, debug modes,
   deterministic Hammersley one-bounce indirect light, and temporal mean.
7. Prove controlled off-screen-emitter delta and deterministic capture; write
   judge-facing docs and Build Week evidence.

## Files to touch

- `crates/radia-math/` (language-neutral semantic math adapter and tests).
- `crates/radia-render/` (WGPU host, WGSL, capture, SDF, RADIA).
- `crates/radia-demo/` (CLI, headless mode, window lifecycle).
- `docs/adr/` (accepted decisions only through AEP ADR commands).
- `docs/hackathon/` and `README.md` (repro and submission evidence).
- `.github/workflows/` and `scripts/` (pinned gates and matrix ban).

## Verification

1. `cargo fmt --all -- --check` -> exit 0 and no diff.
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings` ->
   exit 0.
3. `cargo test --workspace --all-features` -> exit 0.
4. Matrix-ban script -> no project `Mat*` or WGSL `matNxM` types.
5. AEP ADR, pins, doctor, and projection checks -> exit 0.
6. Two fixed-state headless captures -> identical hashes.
7. Controlled RADIA capture -> receiver ROI changes by at least 4/255 with
   direct/G-buffer inputs unchanged.

## Non-goals

Mesh SDFs, imported assets, editor, ECS, physics, skinning, direct Vulkan,
non-Vulkan backends, and production multi-bounce GI.

## Risks

- WGPU API drift -> compile against exactly 26.0.1 and its matching Naga.
- GPU variance -> keep semantic CPU tests distinct from adapter pixel proof.
- Deadline pressure -> retain synthetic analytic MVP; defer production RADIA.

## Open questions

None.

