# CODE.md - Radia

Binding for all committed code. Rationale lives in docs/adr/ (see
INDEX.md); do not relitigate here.

## Conventions by reference

Library: agent-enhanced-project
(install: git clone https://github.com/LairdWT/agent-enhanced-project)

- rust @ plugins/rust/CONVENTIONS.md (v0.26.0)
- graphics @ plugins/graphics/CONVENTIONS.md (v0.17.0)
- math @ plugins/math/CONVENTIONS.md (v0.1.0)
- git @ plugins/git/CONVENTIONS.md (v0.11.0)
- ops @ plugins/ops/CONVENTIONS.md (v0.5.0)
- powershell @ plugins/powershell/CONVENTIONS.md (v0.8.0)
- shell @ plugins/shell/CONVENTIONS.md (v0.6.0)

## Project deviations and additions

- Only `*-core` routers load by default; spokes load only when routed by the
  smallest active unit.
- Matrix types and non-rigid runtime transforms are forbidden in v1
  (`adr:coordinate-and-dual-quaternion-semantics`).
- Projection is analytic, infinite-far, WGPU zero-to-one reverse-Z
  (`adr:matrix-free-reverse-z-projection`).
- Third-party additions are limited to the exact accepted WGPU, Winit, and PNG
  set (`adr:wgpu-vulkan-dependency-set`).

## Toolchain configs are law

- `rust-toolchain.toml`
- `rustfmt.toml`
- workspace lint policy in `Cargo.toml`
- WGSL validation through the exact WGPU/Naga dependency line

## Verification

- `cargo test --workspace --all-features`
