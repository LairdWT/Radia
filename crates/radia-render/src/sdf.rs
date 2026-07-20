use radia_math::{ErrorScale, UnitDualQuat, UnitQuat, Vec3};

use crate::RenderError;

const BOX_HALF_EXTENT: Vec3 = Vec3::new(0.8, 0.8, 0.8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SdfMaterial {
    Floor = 0,
    Box = 1,
    Sphere = 2,
    Emitter = 3,
    Wall = 4,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdfSample {
    pub distance: f32,
    pub material: SdfMaterial,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceTermination {
    Miss,
    Hit,
    Indeterminate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CpuTraceResult {
    pub position: Vec3,
    pub traveled: f32,
    pub material: SdfMaterial,
    pub steps: u32,
    pub termination: TraceTermination,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CpuTraceConfig {
    pub maximum_distance: f32,
    pub hit_guard: f32,
    pub maximum_steps: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CpuSdfScene {
    box_pose: UnitDualQuat,
    sphere_pose: UnitDualQuat,
    emitter_pose: UnitDualQuat,
    emitter_radius: f32,
}

impl CpuSdfScene {
    /// Builds the synthetic courtyard with the same meter-space placements as WGSL.
    ///
    /// # Errors
    ///
    /// Returns an error when the emitter contract is non-finite or non-positive.
    pub fn courtyard(emitter_position: Vec3, emitter_radius: f32) -> Result<Self, RenderError> {
        if !emitter_position.is_finite() || !emitter_radius.is_finite() || emitter_radius <= 0.0 {
            return Err(RenderError::InvalidConfig(
                "CPU SDF emitter must have a finite position and positive finite radius".to_owned(),
            ));
        }
        let error_scale = ErrorScale::new(100.0, 64)?;
        let pose = |translation| {
            UnitDualQuat::from_rotation_translation(UnitQuat::identity(), translation, error_scale)
                .map_err(RenderError::from)
        };
        Ok(Self {
            box_pose: pose(Vec3::new(-1.2, -0.2, -3.6))?,
            sphere_pose: pose(Vec3::new(1.25, 0.0, -4.6))?,
            emitter_pose: pose(emitter_position)?,
            emitter_radius,
        })
    }

    #[must_use]
    pub fn distance(self, world_point: Vec3) -> SdfSample {
        let mut closest = SdfSample {
            distance: world_point.y + 1.0,
            material: SdfMaterial::Floor,
        };
        closest = closer(
            closest,
            SdfSample {
                distance: world_point.z + 8.0,
                material: SdfMaterial::Wall,
            },
        );
        closest = closer(
            closest,
            SdfSample {
                distance: box_distance(
                    self.box_pose.inverse().transform_point(world_point),
                    BOX_HALF_EXTENT,
                ),
                material: SdfMaterial::Box,
            },
        );
        closest = closer(
            closest,
            SdfSample {
                distance: sphere_distance(
                    self.sphere_pose.inverse().transform_point(world_point),
                    1.0,
                ),
                material: SdfMaterial::Sphere,
            },
        );
        closer(
            closest,
            SdfSample {
                distance: sphere_distance(
                    self.emitter_pose.inverse().transform_point(world_point),
                    self.emitter_radius,
                ),
                material: SdfMaterial::Emitter,
            },
        )
    }

    /// Returns the analytic outward normal for a declared winning primitive.
    ///
    /// # Errors
    ///
    /// Returns an error when asked for a normal at a degenerate primitive point.
    pub fn normal(self, material: SdfMaterial, world_point: Vec3) -> Result<Vec3, RenderError> {
        let error_scale = ErrorScale::new(100.0, 32)?;
        let normal = match material {
            SdfMaterial::Floor => Vec3::Y,
            SdfMaterial::Wall => Vec3::Z,
            SdfMaterial::Sphere => self
                .sphere_pose
                .rotation()
                .rotate_vector(self.sphere_pose.inverse().transform_point(world_point)),
            SdfMaterial::Emitter => self
                .emitter_pose
                .rotation()
                .rotate_vector(self.emitter_pose.inverse().transform_point(world_point)),
            SdfMaterial::Box => {
                let local = self.box_pose.inverse().transform_point(world_point);
                let boundary = Vec3::new(
                    local.x.abs() - BOX_HALF_EXTENT.x,
                    local.y.abs() - BOX_HALF_EXTENT.y,
                    local.z.abs() - BOX_HALF_EXTENT.z,
                );
                let local_normal = if boundary.x >= boundary.y && boundary.x >= boundary.z {
                    Vec3::new(local.x.signum(), 0.0, 0.0)
                } else if boundary.y >= boundary.z {
                    Vec3::new(0.0, local.y.signum(), 0.0)
                } else {
                    Vec3::new(0.0, 0.0, local.z.signum())
                };
                self.box_pose.transform_direction(local_normal)
            }
        };
        normal.try_normalize(error_scale).map_err(RenderError::from)
    }

    /// Traces with the same bounded termination policy as the shader.
    ///
    /// # Errors
    ///
    /// Returns an error for non-finite rays, non-unit directions, or invalid bounds.
    pub fn trace(
        self,
        origin: Vec3,
        direction: Vec3,
        config: CpuTraceConfig,
    ) -> Result<CpuTraceResult, RenderError> {
        validate_trace(origin, direction, config)?;
        let mut traveled = 0.0;
        let mut material = SdfMaterial::Floor;
        for step in 0..config.maximum_steps {
            let position = origin + direction * traveled;
            let sample = self.distance(position);
            material = sample.material;
            let guard = config.hit_guard * traveled.max(1.0);
            if sample.distance.abs() <= guard {
                return Ok(CpuTraceResult {
                    position,
                    traveled,
                    material,
                    steps: step + 1,
                    termination: TraceTermination::Hit,
                });
            }
            traveled += sample.distance.max(guard * 0.5);
            if traveled > config.maximum_distance {
                return Ok(CpuTraceResult {
                    position,
                    traveled,
                    material,
                    steps: step + 1,
                    termination: TraceTermination::Miss,
                });
            }
        }
        Ok(CpuTraceResult {
            position: origin + direction * traveled,
            traveled,
            material,
            steps: config.maximum_steps,
            termination: TraceTermination::Indeterminate,
        })
    }
}

fn sphere_distance(local_point: Vec3, radius: f32) -> f32 {
    local_point.length() - radius
}

fn box_distance(local_point: Vec3, half_extent: Vec3) -> f32 {
    let offset = Vec3::new(
        local_point.x.abs() - half_extent.x,
        local_point.y.abs() - half_extent.y,
        local_point.z.abs() - half_extent.z,
    );
    let outside = Vec3::new(offset.x.max(0.0), offset.y.max(0.0), offset.z.max(0.0));
    outside.length() + offset.x.max(offset.y.max(offset.z)).min(0.0)
}

fn closer(left: SdfSample, right: SdfSample) -> SdfSample {
    if right.distance < left.distance {
        right
    } else {
        left
    }
}

fn validate_trace(
    origin: Vec3,
    direction: Vec3,
    config: CpuTraceConfig,
) -> Result<(), RenderError> {
    if !origin.is_finite() || !direction.is_finite() {
        return Err(RenderError::InvalidConfig(
            "CPU SDF ray must be finite".to_owned(),
        ));
    }
    let error_scale = ErrorScale::new(1.0, 32)?;
    let length_error = (direction.length_squared() - 1.0).abs();
    if length_error > error_scale.linear_guard() {
        return Err(RenderError::InvalidConfig(
            "CPU SDF ray direction must be normalized".to_owned(),
        ));
    }
    if !config.maximum_distance.is_finite()
        || config.maximum_distance <= 0.0
        || !config.hit_guard.is_finite()
        || config.hit_guard <= 0.0
        || config.maximum_steps == 0
        || config.maximum_steps > 128
    {
        return Err(RenderError::InvalidConfig(
            "CPU SDF trace bounds must be positive, finite, and at most 128 steps".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CpuSdfScene, CpuTraceConfig, SdfMaterial, TraceTermination};
    use crate::default_render_settings;
    use crate::renderer::RadiaMode;
    use radia_math::{ErrorScale, Vec3};

    fn scene() -> CpuSdfScene {
        let settings = default_render_settings(RadiaMode::Off).expect("frozen settings are valid");
        let primary_light = settings.lights[0];
        CpuSdfScene::courtyard(primary_light.position, primary_light.radius)
            .expect("frozen scene is valid")
    }

    #[test]
    fn sign_and_rigid_placement_match_the_shader_contract() {
        let scene = scene();
        let center = Vec3::new(1.25, 0.0, -4.6);
        let sample = scene.distance(center);
        assert_eq!(sample.material, SdfMaterial::Sphere);
        let guard = ErrorScale::new(100.0, 64)
            .expect("scale is valid")
            .linear_guard();
        assert!((sample.distance + 1.0).abs() <= guard);

        let surface = Vec3::new(2.25, 0.0, -4.6);
        let surface_sample = scene.distance(surface);
        assert!(surface_sample.distance.abs() <= guard);
        assert_eq!(
            scene
                .normal(SdfMaterial::Sphere, surface)
                .expect("surface normal is valid"),
            Vec3::X
        );
    }

    #[test]
    fn bounded_trace_distinguishes_hit_miss_and_indeterminate() {
        let scene = scene();
        let standard = CpuTraceConfig {
            maximum_distance: 40.0,
            hit_guard: 0.000_5,
            maximum_steps: 128,
        };
        let hit = scene
            .trace(Vec3::new(0.0, 0.6, 4.5), -Vec3::Z, standard)
            .expect("trace is valid");
        assert_eq!(hit.termination, TraceTermination::Hit);

        let miss = scene
            .trace(Vec3::new(0.0, 0.6, 4.5), Vec3::Y, standard)
            .expect("trace is valid");
        assert_eq!(miss.termination, TraceTermination::Miss);

        let indeterminate = scene
            .trace(
                Vec3::new(0.0, 0.6, 4.5),
                -Vec3::Z,
                CpuTraceConfig {
                    maximum_steps: 1,
                    ..standard
                },
            )
            .expect("trace is valid");
        assert_eq!(indeterminate.termination, TraceTermination::Indeterminate);
    }

    #[test]
    fn source_contract_keeps_cpu_and_wgsl_limits_aligned() {
        let shader = include_str!("shaders/scene.wgsl");
        assert!(shader.contains("world_point.y + 1.0"));
        assert!(shader.contains("world_point.z + 8.0"));
        assert!(shader.contains("@group(0) @binding(1) var<storage, read> dragon_field"));
        assert!(shader.contains("dragon_index < 3u"));
        assert!(shader.contains("dragon_safe_distance(world_point, dragon_index)"));
        assert!(shader.contains("dragon_material(dragon_index),\n                dragon_index"));
        assert!(shader.contains("light_index < 3u"));
        assert!(shader.contains("step < 128u"));
        assert!(shader.contains("abs(sample.distance) <= hit_guard"));
        assert!(shader.contains("max(outside_distance, sampled_distance - outside_distance)"));
        assert!(!shader.contains("max(outside_distance, 2.0 * frame.dragon_minimum_error.w)"));
    }
}
