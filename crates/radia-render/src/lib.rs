//! Vulkan-backed, matrix-free rendering for Radia.

/// Confirms the renderer crate and math crate are linked.
#[must_use]
pub const fn crate_ready() -> bool {
    radia_math::crate_ready()
}
