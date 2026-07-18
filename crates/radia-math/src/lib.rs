//! Matrix-free game math for Radia.
//!
//! Conventions are fixed by `adr:coordinate-and-dual-quaternion-semantics`
//! and `adr:matrix-free-reverse-z-projection`.

mod error;
mod projection;
mod rotation;
mod vector;

pub use error::{ErrorScale, MathError};
pub use projection::{ClipPoint, Ray, ReverseZPerspective, ScreenPoint};
pub use rotation::{UnitDualQuat, UnitQuat};
pub use vector::{Vec2, Vec3, Vec4};
