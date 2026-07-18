use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::{ErrorScale, MathError};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
    pub const X: Self = Self::new(1.0, 0.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0, 0.0);
    pub const Z: Self = Self::new(0.0, 0.0, 1.0);

    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    #[must_use]
    pub fn dot(self, right: Self) -> f32 {
        self.x
            .mul_add(right.x, self.y.mul_add(right.y, self.z * right.z))
    }

    #[must_use]
    pub fn cross(self, right: Self) -> Self {
        Self::new(
            self.y.mul_add(right.z, -self.z * right.y),
            self.z.mul_add(right.x, -self.x * right.z),
            self.x.mul_add(right.y, -self.y * right.x),
        )
    }

    #[must_use]
    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }

    #[must_use]
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Normalizes a vector after a scale-derived degeneracy check.
    ///
    /// # Errors
    ///
    /// Returns an error for non-finite input or insufficient length.
    pub fn try_normalize(self, error_scale: ErrorScale) -> Result<Self, MathError> {
        if !self.is_finite() {
            return Err(MathError::NonFinite("vector"));
        }
        let length_squared = self.length_squared();
        if !length_squared.is_finite() {
            return Err(MathError::NonFinite("vector length"));
        }
        if length_squared <= error_scale.squared_length_guard() {
            return Err(MathError::Degenerate("vector"));
        }
        Ok(self / length_squared.sqrt())
    }

    #[must_use]
    pub fn to_array(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, right: Self) -> Self::Output {
        Self::new(self.x + right.x, self.y + right.y, self.z + right.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, right: Self) -> Self::Output {
        Self::new(self.x - right.x, self.y - right.y, self.z - right.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self::Output {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, scalar: f32) -> Self::Output {
        Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite() && self.w.is_finite()
    }

    #[must_use]
    pub fn to_array(self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
    }
}
