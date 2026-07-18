use crate::{ErrorScale, MathError, UnitDualQuat, Vec2, Vec3, Vec4};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClipPoint {
    pub value: Vec4,
}

impl ClipPoint {
    #[must_use]
    pub fn ndc(self) -> Vec3 {
        Vec3::new(
            self.value.x / self.value.w,
            self.value.y / self.value.w,
            self.value.z / self.value.w,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenPoint {
    pub position: Vec2,
    pub depth: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

/// Infinite-far perspective with WGPU zero-to-one reverse-Z depth.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReverseZPerspective {
    vertical_fov: f32,
    aspect: f32,
    near: f32,
    tangent_half_fov: f32,
}

impl ReverseZPerspective {
    /// Validates and builds an analytic projection.
    ///
    /// # Errors
    ///
    /// Returns an error for non-finite or out-of-domain parameters.
    pub fn new(vertical_fov: f32, aspect: f32, near: f32) -> Result<Self, MathError> {
        if !vertical_fov.is_finite() || vertical_fov <= 0.0 || vertical_fov >= std::f32::consts::PI
        {
            return Err(MathError::InvalidProjection("vertical field of view"));
        }
        if !aspect.is_finite() || aspect <= 0.0 {
            return Err(MathError::InvalidProjection("aspect"));
        }
        if !near.is_finite() || near <= 0.0 {
            return Err(MathError::InvalidProjection("near"));
        }
        let tangent_half_fov = (vertical_fov * 0.5).tan();
        if !tangent_half_fov.is_finite() || tangent_half_fov <= 0.0 {
            return Err(MathError::InvalidProjection("field of view tangent"));
        }
        Ok(Self {
            vertical_fov,
            aspect,
            near,
            tangent_half_fov,
        })
    }

    #[must_use]
    pub fn vertical_fov(self) -> f32 {
        self.vertical_fov
    }

    #[must_use]
    pub fn aspect(self) -> f32 {
        self.aspect
    }

    #[must_use]
    pub fn near(self) -> f32 {
        self.near
    }

    /// Projects a camera-space point without a projection matrix.
    ///
    /// # Errors
    ///
    /// Returns an error for non-finite input or points on/behind the camera.
    pub fn project_camera(self, camera_point: Vec3) -> Result<ClipPoint, MathError> {
        if !camera_point.is_finite() {
            return Err(MathError::NonFinite("camera point"));
        }
        if camera_point.z >= 0.0 {
            return Err(MathError::OutsideProjectionDomain);
        }
        let clip = Vec4::new(
            camera_point.x / (self.tangent_half_fov * self.aspect),
            camera_point.y / self.tangent_half_fov,
            self.near,
            -camera_point.z,
        );
        if !clip.is_finite() {
            return Err(MathError::NonFinite("clip point"));
        }
        Ok(ClipPoint { value: clip })
    }

    /// Projects a world point into top-left framebuffer coordinates.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid extent, pose, or projection domain.
    pub fn project_world(
        self,
        camera_to_world: UnitDualQuat,
        world_point: Vec3,
        width: u32,
        height: u32,
    ) -> Result<ScreenPoint, MathError> {
        let (width_f32, height_f32) = extent_as_f32(width, height)?;
        let camera_point = camera_to_world.inverse().transform_point(world_point);
        let ndc = self.project_camera(camera_point)?.ndc();
        Ok(ScreenPoint {
            position: Vec2::new(
                (ndc.x + 1.0) * 0.5 * width_f32,
                (1.0 - ndc.y) * 0.5 * height_f32,
            ),
            depth: ndc.z,
        })
    }

    /// Builds a world-space ray through a framebuffer pixel center.
    ///
    /// # Errors
    ///
    /// Returns an error for out-of-range pixels or invalid normalization.
    pub fn screen_ray_for_pixel(
        self,
        camera_to_world: UnitDualQuat,
        pixel_x: u32,
        pixel_y: u32,
        width: u32,
        height: u32,
        error_scale: ErrorScale,
    ) -> Result<Ray, MathError> {
        let (width_f32, height_f32) = extent_as_f32(width, height)?;
        if pixel_x >= width || pixel_y >= height {
            return Err(MathError::PixelOutsideExtent);
        }
        let pixel_column = f32::from(
            u16::try_from(pixel_x)
                .map_err(|_| MathError::InvalidProjection("pixel exceeds exact adapter range"))?,
        );
        let pixel_row = f32::from(
            u16::try_from(pixel_y)
                .map_err(|_| MathError::InvalidProjection("pixel exceeds exact adapter range"))?,
        );
        let pixel_center = Vec2::new(pixel_column + 0.5, pixel_row + 0.5);
        let ndc_x = pixel_center.x.mul_add(2.0 / width_f32, -1.0);
        let ndc_y = 1.0 - pixel_center.y * (2.0 / height_f32);
        let camera_direction = Vec3::new(
            ndc_x * self.tangent_half_fov * self.aspect,
            ndc_y * self.tangent_half_fov,
            -1.0,
        )
        .try_normalize(error_scale)?;
        let world_direction = camera_to_world
            .transform_direction(camera_direction)
            .try_normalize(error_scale)?;
        Ok(Ray {
            origin: camera_to_world.translation(),
            direction: world_direction,
        })
    }
}

fn extent_as_f32(width: u32, height: u32) -> Result<(f32, f32), MathError> {
    if width == 0 || height == 0 {
        return Err(MathError::InvalidProjection("framebuffer extent"));
    }
    let exact_width = u16::try_from(width).map_err(|_| {
        MathError::InvalidProjection("framebuffer width exceeds exact adapter range")
    })?;
    let exact_height = u16::try_from(height).map_err(|_| {
        MathError::InvalidProjection("framebuffer height exceeds exact adapter range")
    })?;
    Ok((f32::from(exact_width), f32::from(exact_height)))
}
