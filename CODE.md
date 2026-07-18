# CODE.md - Radia

Binding for all committed code. Rationale lives in docs/adr/ (see
INDEX.md); do not relitigate here.

## Conventions by reference

Library: agent-enhanced-project
(install: git clone https://github.com/LairdWT/agent-enhanced-project)

- rust @ plugins/rust/CONVENTIONS.md (v0.25.0)
- graphics @ plugins/graphics/CONVENTIONS.md (v0.16.0)
- math @ plugins/math/CONVENTIONS.md (v0.1.0)
- git @ plugins/git/CONVENTIONS.md (v0.10.0)
- ops @ plugins/ops/CONVENTIONS.md (v0.4.0)
- powershell @ plugins/powershell/CONVENTIONS.md (v0.7.0)
- shell @ plugins/shell/CONVENTIONS.md (v0.5.0)

## Project deviations and additions

- (none yet; add one line per deviation with its adr:<slug>)

## Toolchain configs are law

- `rust-toolchain.toml`
- `rustfmt.toml`
- workspace lint policy in `Cargo.toml`
- WGSL validation through the exact WGPU/Naga dependency line

## Verification

- `cargo test --workspace --all-features`
