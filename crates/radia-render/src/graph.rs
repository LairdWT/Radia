use crate::RenderError;

pub(crate) const GBUFFER_ALBEDO_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub(crate) const GBUFFER_NORMAL_MATERIAL_FORMAT: wgpu::TextureFormat =
    wgpu::TextureFormat::Rgba16Float;
pub(crate) const GBUFFER_EMISSIVE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
pub(crate) const GBUFFER_TRACE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
pub(crate) const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub(crate) const SCENE_RADIANCE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
pub(crate) const INDIRECT_RADIANCE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
pub(crate) const DEPTH_CLEAR: f32 = 0.0;
pub(crate) const DEPTH_COMPARE: wgpu::CompareFunction = wgpu::CompareFunction::Greater;

const COLOR_TARGET_LIMIT: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Transient {
    GBufferAlbedo,
    GBufferNormalMaterial,
    GBufferEmissive,
    GBufferTrace,
    Depth,
    DirectRadiance,
    IndirectRadiance,
    SceneRadiance,
    Presentation,
}

impl Transient {
    const fn name(self) -> &'static str {
        match self {
            Self::GBufferAlbedo => "gbuffer_albedo",
            Self::GBufferNormalMaterial => "gbuffer_normal_material",
            Self::GBufferEmissive => "gbuffer_emissive",
            Self::GBufferTrace => "gbuffer_trace",
            Self::Depth => "depth",
            Self::DirectRadiance => "direct_radiance",
            Self::IndirectRadiance => "indirect_radiance",
            Self::SceneRadiance => "scene_radiance",
            Self::Presentation => "presentation",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct PassContract {
    name: &'static str,
    reads: &'static [Transient],
    writes: &'static [Transient],
}

const GBUFFER_WRITES: &[Transient] = &[
    Transient::GBufferAlbedo,
    Transient::GBufferNormalMaterial,
    Transient::GBufferEmissive,
    Transient::GBufferTrace,
    Transient::Depth,
];
const DIRECT_READS: &[Transient] = GBUFFER_WRITES;
const DIRECT_WRITES: &[Transient] = &[Transient::DirectRadiance];
const INDIRECT_READS: &[Transient] = &[
    Transient::GBufferAlbedo,
    Transient::GBufferNormalMaterial,
    Transient::GBufferTrace,
    Transient::Depth,
    Transient::DirectRadiance,
];
const INDIRECT_WRITES: &[Transient] = &[Transient::IndirectRadiance];
const COMPOSITE_READS: &[Transient] = &[
    Transient::DirectRadiance,
    Transient::IndirectRadiance,
    Transient::GBufferNormalMaterial,
    Transient::Depth,
];
const COMPOSITE_WRITES: &[Transient] = &[Transient::SceneRadiance];
const PRESENT_READS: &[Transient] = &[Transient::SceneRadiance];
const PRESENT_WRITES: &[Transient] = &[Transient::Presentation];

const PASS_CONTRACTS: &[PassContract] = &[
    PassContract {
        name: "gbuffer_geometry",
        reads: &[],
        writes: GBUFFER_WRITES,
    },
    PassContract {
        name: "deferred_direct_lighting",
        reads: DIRECT_READS,
        writes: DIRECT_WRITES,
    },
    PassContract {
        name: "screen_space_indirect",
        reads: INDIRECT_READS,
        writes: INDIRECT_WRITES,
    },
    PassContract {
        name: "deferred_composite",
        reads: COMPOSITE_READS,
        writes: COMPOSITE_WRITES,
    },
    PassContract {
        name: "presentation",
        reads: PRESENT_READS,
        writes: PRESENT_WRITES,
    },
];

pub(crate) fn validate_pass_contracts() -> Result<(), RenderError> {
    validate_contracts(PASS_CONTRACTS)
}

fn validate_contracts(contracts: &[PassContract]) -> Result<(), RenderError> {
    let mut written = Vec::new();
    for pass in contracts {
        for transient in pass.reads {
            if !written.contains(transient) {
                return Err(RenderError::InvalidConfig(format!(
                    "deferred pass '{}' reads '{}' before it is written",
                    pass.name,
                    transient.name()
                )));
            }
        }
        for transient in pass.writes {
            if !written.contains(transient) {
                written.push(*transient);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn pass_names() -> impl Iterator<Item = &'static str> {
    PASS_CONTRACTS.iter().map(|pass| pass.name)
}

#[derive(Debug)]
pub(crate) struct FrameTargets {
    pub(crate) gbuffer_albedo: wgpu::TextureView,
    pub(crate) gbuffer_normal_material: wgpu::TextureView,
    pub(crate) gbuffer_emissive: wgpu::TextureView,
    pub(crate) gbuffer_trace: wgpu::TextureView,
    pub(crate) depth: wgpu::TextureView,
    pub(crate) direct_radiance: wgpu::TextureView,
    pub(crate) indirect_radiance: wgpu::TextureView,
    pub(crate) scene_radiance: wgpu::TextureView,
}

impl FrameTargets {
    pub(crate) fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        debug_assert_eq!(GBUFFER_WRITES.len() - 1, COLOR_TARGET_LIMIT);
        Self {
            gbuffer_albedo: create_view(
                device,
                width,
                height,
                GBUFFER_ALBEDO_FORMAT,
                color_usage(),
                "radia-gbuffer-albedo",
            ),
            gbuffer_normal_material: create_view(
                device,
                width,
                height,
                GBUFFER_NORMAL_MATERIAL_FORMAT,
                color_usage(),
                "radia-gbuffer-normal-material",
            ),
            gbuffer_emissive: create_view(
                device,
                width,
                height,
                GBUFFER_EMISSIVE_FORMAT,
                color_usage(),
                "radia-gbuffer-emissive",
            ),
            gbuffer_trace: create_view(
                device,
                width,
                height,
                GBUFFER_TRACE_FORMAT,
                color_usage(),
                "radia-gbuffer-trace",
            ),
            depth: create_view(
                device,
                width,
                height,
                DEPTH_FORMAT,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                "radia-depth",
            ),
            direct_radiance: create_view(
                device,
                width,
                height,
                SCENE_RADIANCE_FORMAT,
                color_usage(),
                "radia-direct-radiance",
            ),
            indirect_radiance: create_view(
                device,
                width,
                height,
                INDIRECT_RADIANCE_FORMAT,
                color_usage(),
                "radia-indirect-radiance",
            ),
            scene_radiance: create_view(
                device,
                width,
                height,
                SCENE_RADIANCE_FORMAT,
                color_usage(),
                "radia-scene-radiance",
            ),
        }
    }
}

const fn color_usage() -> wgpu::TextureUsages {
    wgpu::TextureUsages::RENDER_ATTACHMENT.union(wgpu::TextureUsages::TEXTURE_BINDING)
}

fn create_view(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
    label: &'static str,
) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor {
            label: Some(label),
            ..Default::default()
        })
}

#[cfg(test)]
mod tests {
    use super::{
        COLOR_TARGET_LIMIT, GBUFFER_WRITES, PASS_CONTRACTS, PassContract, Transient, pass_names,
        validate_contracts,
    };

    #[test]
    fn fixed_graph_has_expected_order_and_fits_attachment_limit() {
        let names = pass_names().collect::<Vec<_>>();
        assert_eq!(
            names,
            [
                "gbuffer_geometry",
                "deferred_direct_lighting",
                "screen_space_indirect",
                "deferred_composite",
                "presentation"
            ]
        );
        assert_eq!(GBUFFER_WRITES.len() - 1, COLOR_TARGET_LIMIT);
        super::validate_pass_contracts().expect("fixed graph is valid");
    }

    #[test]
    fn read_before_write_is_rejected_with_named_pass_and_transient() {
        let invalid = [PassContract {
            name: "deferred_direct_lighting",
            reads: &[Transient::Depth],
            writes: &[Transient::SceneRadiance],
        }];
        let error = validate_contracts(&invalid).expect_err("read-before-write must fail");
        let message = error.to_string();
        assert!(message.contains("deferred_direct_lighting"));
        assert!(message.contains("depth"));
        assert_eq!(PASS_CONTRACTS.len(), 5);
    }
}
