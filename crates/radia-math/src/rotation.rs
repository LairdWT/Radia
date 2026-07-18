use crate::{ErrorScale, MathError, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Quat {
    w: f32,
    x: f32,
    y: f32,
    z: f32,
}

impl Quat {
    const IDENTITY: Self = Self::new(1.0, 0.0, 0.0, 0.0);

    const fn new(w: f32, x: f32, y: f32, z: f32) -> Self {
        Self { w, x, y, z }
    }

    fn from_vector(vector: Vec3) -> Self {
        Self::new(0.0, vector.x, vector.y, vector.z)
    }

    fn is_finite(self) -> bool {
        self.w.is_finite() && self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    fn dot(self, right: Self) -> f32 {
        self.w.mul_add(
            right.w,
            self.x
                .mul_add(right.x, self.y.mul_add(right.y, self.z * right.z)),
        )
    }

    fn hamilton(self, right: Self) -> Self {
        Self::new(
            self.w.mul_add(
                right.w,
                -self.x * right.x - self.y * right.y - self.z * right.z,
            ),
            self.w.mul_add(
                right.x,
                self.x.mul_add(right.w, self.y * right.z - self.z * right.y),
            ),
            self.w.mul_add(
                right.y,
                self.y.mul_add(right.w, self.z * right.x - self.x * right.z),
            ),
            self.w.mul_add(
                right.z,
                self.z.mul_add(right.w, self.x * right.y - self.y * right.x),
            ),
        )
    }

    fn conjugate(self) -> Self {
        Self::new(self.w, -self.x, -self.y, -self.z)
    }

    fn scale(self, scalar: f32) -> Self {
        Self::new(
            self.w * scalar,
            self.x * scalar,
            self.y * scalar,
            self.z * scalar,
        )
    }

    fn add(self, right: Self) -> Self {
        Self::new(
            self.w + right.w,
            self.x + right.x,
            self.y + right.y,
            self.z + right.z,
        )
    }

    fn sub(self, right: Self) -> Self {
        Self::new(
            self.w - right.w,
            self.x - right.x,
            self.y - right.y,
            self.z - right.z,
        )
    }

    fn vector(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    fn xyzw(self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
    }

    fn wxyz(self) -> [f32; 4] {
        [self.w, self.x, self.y, self.z]
    }
}

/// Finite normalized Hamilton quaternion with semantic `wxyz` components.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnitQuat {
    raw: Quat,
}

impl UnitQuat {
    #[must_use]
    pub const fn identity() -> Self {
        Self {
            raw: Quat::IDENTITY,
        }
    }

    /// Normalizes finite semantic `wxyz` components.
    ///
    /// # Errors
    ///
    /// Returns an error for non-finite or scale-degenerate input.
    pub fn try_from_wxyz(
        w: f32,
        x: f32,
        y: f32,
        z: f32,
        error_scale: ErrorScale,
    ) -> Result<Self, MathError> {
        Self::try_from_raw(Quat::new(w, x, y, z), error_scale)
    }

    /// Builds an active rotation from a normalized axis adapter and radians.
    ///
    /// # Errors
    ///
    /// Returns an error for non-finite angle or invalid axis.
    pub fn try_from_axis_angle(
        axis: Vec3,
        radians: f32,
        error_scale: ErrorScale,
    ) -> Result<Self, MathError> {
        if !radians.is_finite() {
            return Err(MathError::NonFinite("angle"));
        }
        let unit_axis = axis.try_normalize(error_scale)?;
        let half_angle = radians * 0.5;
        let vector_scale = half_angle.sin();
        Self::try_from_raw(
            Quat::new(
                half_angle.cos(),
                unit_axis.x * vector_scale,
                unit_axis.y * vector_scale,
                unit_axis.z * vector_scale,
            ),
            error_scale,
        )
    }

    fn try_from_raw(raw: Quat, error_scale: ErrorScale) -> Result<Self, MathError> {
        if !raw.is_finite() {
            return Err(MathError::NonFinite("quaternion"));
        }
        let length_squared = raw.dot(raw);
        if !length_squared.is_finite() {
            return Err(MathError::NonFinite("quaternion length"));
        }
        if length_squared <= error_scale.squared_length_guard() {
            return Err(MathError::Degenerate("quaternion"));
        }
        Ok(Self {
            raw: raw.scale(length_squared.sqrt().recip()),
        })
    }

    #[must_use]
    pub fn conjugate(self) -> Self {
        Self {
            raw: self.raw.conjugate(),
        }
    }

    /// Composes rotations so `left.compose(right)` applies `right` first.
    ///
    /// # Errors
    ///
    /// Returns an error when accumulated floating-point state is invalid.
    pub fn compose(self, right: Self, error_scale: ErrorScale) -> Result<Self, MathError> {
        Self::try_from_raw(self.raw.hamilton(right.raw), error_scale)
    }

    #[must_use]
    pub fn rotate_vector(self, vector: Vec3) -> Vec3 {
        self.raw
            .hamilton(Quat::from_vector(vector))
            .hamilton(self.raw.conjugate())
            .vector()
    }

    #[must_use]
    pub fn to_wxyz(self) -> [f32; 4] {
        self.raw.wxyz()
    }

    #[must_use]
    pub fn to_gpu_xyzw(self) -> [f32; 4] {
        self.raw.xyzw()
    }

    #[must_use]
    pub fn orientation_dot(self, right: Self) -> f32 {
        self.raw.dot(right.raw)
    }
}

/// Normalized rigid dual quaternion. Scale and shear are not representable.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnitDualQuat {
    real: Quat,
    dual: Quat,
}

impl UnitDualQuat {
    #[must_use]
    pub const fn identity() -> Self {
        Self {
            real: Quat::IDENTITY,
            dual: Quat::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Creates a rigid pose from active rotation and world translation.
    ///
    /// # Errors
    ///
    /// Returns an error for non-finite translation or failed normalization.
    pub fn from_rotation_translation(
        rotation: UnitQuat,
        translation: Vec3,
        error_scale: ErrorScale,
    ) -> Result<Self, MathError> {
        if !translation.is_finite() {
            return Err(MathError::NonFinite("translation"));
        }
        let dual = Quat::from_vector(translation)
            .hamilton(rotation.raw)
            .scale(0.5);
        Self::try_from_parts(rotation.raw, dual, error_scale)
    }

    fn try_from_parts(real: Quat, dual: Quat, error_scale: ErrorScale) -> Result<Self, MathError> {
        if !real.is_finite() || !dual.is_finite() {
            return Err(MathError::NonFinite("dual quaternion"));
        }
        let real_length_squared = real.dot(real);
        if !real_length_squared.is_finite() {
            return Err(MathError::NonFinite("dual quaternion length"));
        }
        if real_length_squared <= error_scale.squared_length_guard() {
            return Err(MathError::Degenerate("dual quaternion real part"));
        }
        let inverse_length = real_length_squared.sqrt().recip();
        let unit_real = real.scale(inverse_length);
        let scaled_dual = dual.scale(inverse_length);
        let unit_dual = scaled_dual.sub(unit_real.scale(unit_real.dot(scaled_dual)));
        Ok(Self {
            real: unit_real,
            dual: unit_dual,
        })
    }

    /// Composes poses so `left.compose(right)` applies `right` first.
    ///
    /// # Errors
    ///
    /// Returns an error when accumulated floating-point state is invalid.
    pub fn compose(self, right: Self, error_scale: ErrorScale) -> Result<Self, MathError> {
        let real = self.real.hamilton(right.real);
        let dual = self
            .real
            .hamilton(right.dual)
            .add(self.dual.hamilton(right.real));
        Self::try_from_parts(real, dual, error_scale)
    }

    #[must_use]
    pub fn inverse(self) -> Self {
        Self {
            real: self.real.conjugate(),
            dual: self.dual.conjugate(),
        }
    }

    #[must_use]
    pub fn rotation(self) -> UnitQuat {
        UnitQuat { raw: self.real }
    }

    #[must_use]
    pub fn translation(self) -> Vec3 {
        self.dual
            .hamilton(self.real.conjugate())
            .scale(2.0)
            .vector()
    }

    #[must_use]
    pub fn transform_point(self, point: Vec3) -> Vec3 {
        self.rotation().rotate_vector(point) + self.translation()
    }

    #[must_use]
    pub fn transform_direction(self, direction: Vec3) -> Vec3 {
        self.rotation().rotate_vector(direction)
    }

    #[must_use]
    pub fn study_dot(self) -> f32 {
        self.real.dot(self.dual)
    }

    /// Packs two explicit `xyzw` quaternions for WGSL.
    #[must_use]
    pub fn to_gpu_xyzw(self) -> [f32; 8] {
        let real = self.real.xyzw();
        let dual = self.dual.xyzw();
        [
            real[0], real[1], real[2], real[3], dual[0], dual[1], dual[2], dual[3],
        ]
    }

    #[must_use]
    pub fn pose_dot(self, right: Self) -> f32 {
        self.real.dot(right.real) + self.dual.dot(right.dual)
    }
}
