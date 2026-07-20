use crate::RenderError;

const ASSET_BYTES: &[u8] = include_bytes!("../../../assets/stanford-dragon/dragon-128.rduf");
pub(crate) const DRAGON_FIELD_SHA256: &str =
    "9a8babdacdab6dbc3b8789b5008bbbaee4c58c7ffea42183ada83397d5cb3862";
const MAGIC: &[u8; 8] = b"RADIAUDF";
const VERSION: u32 = 1;
const HEADER_BYTES: usize = 52;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DragonFieldMetadata {
    pub resolution: [u32; 3],
    pub minimum: [f32; 3],
    pub maximum: [f32; 3],
    pub conservative_error: f32,
}

#[derive(Debug)]
pub(crate) struct DragonFieldAsset {
    pub metadata: DragonFieldMetadata,
    pub payload: Vec<u8>,
}

impl DragonFieldAsset {
    pub fn embedded() -> Result<Self, RenderError> {
        Self::parse(ASSET_BYTES)
    }

    fn parse(bytes: &[u8]) -> Result<Self, RenderError> {
        if bytes.len() < HEADER_BYTES || bytes.get(0..8) != Some(MAGIC) {
            return Err(asset_error("missing RADIAUDF header"));
        }
        let version = read_u32(bytes, 8)?;
        if version != VERSION {
            return Err(asset_error(format!("unsupported UDF version {version}")));
        }
        let resolution = [
            read_u32(bytes, 12)?,
            read_u32(bytes, 16)?,
            read_u32(bytes, 20)?,
        ];
        if resolution.iter().any(|value| !(32..=256).contains(value)) {
            return Err(asset_error("UDF resolution is outside 32..=256"));
        }
        if resolution[0] != resolution[1] || resolution[1] != resolution[2] {
            return Err(asset_error("UDF volume must be cubic"));
        }
        let minimum = [
            read_f32(bytes, 24)?,
            read_f32(bytes, 28)?,
            read_f32(bytes, 32)?,
        ];
        let maximum = [
            read_f32(bytes, 36)?,
            read_f32(bytes, 40)?,
            read_f32(bytes, 44)?,
        ];
        let conservative_error = read_f32(bytes, 48)?;
        if minimum
            .iter()
            .chain(maximum.iter())
            .any(|value| !value.is_finite())
            || !conservative_error.is_finite()
            || conservative_error <= 0.0
            || (0..3).any(|axis| minimum[axis] >= maximum[axis])
        {
            return Err(asset_error("UDF bounds or error contract is invalid"));
        }

        let sample_count = resolution.iter().try_fold(1_usize, |product, value| {
            product.checked_mul(*value as usize)
        });
        let payload_bytes = sample_count
            .and_then(|count| count.checked_mul(4))
            .ok_or_else(|| asset_error("UDF payload size overflow"))?;
        if bytes.len() != HEADER_BYTES + payload_bytes {
            return Err(asset_error(format!(
                "UDF payload has {} bytes, expected {payload_bytes}",
                bytes.len().saturating_sub(HEADER_BYTES)
            )));
        }

        let mut payload = Vec::with_capacity(payload_bytes);
        for chunk in bytes[HEADER_BYTES..].chunks_exact(4) {
            let value = f32::from_le_bytes(
                chunk
                    .try_into()
                    .map_err(|_| asset_error("UDF sample is not four bytes"))?,
            );
            if !value.is_finite() || value < 0.0 {
                return Err(asset_error("UDF sample is negative or non-finite"));
            }
            payload.extend_from_slice(&value.to_ne_bytes());
        }
        Ok(Self {
            metadata: DragonFieldMetadata {
                resolution,
                minimum,
                maximum,
                conservative_error,
            },
            payload,
        })
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, RenderError> {
    let field = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| asset_error("UDF header is truncated"))?;
    Ok(u32::from_le_bytes(field.try_into().map_err(|_| {
        asset_error("UDF u32 field is not four bytes")
    })?))
}

fn read_f32(bytes: &[u8], offset: usize) -> Result<f32, RenderError> {
    let field = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| asset_error("UDF header is truncated"))?;
    Ok(f32::from_le_bytes(field.try_into().map_err(|_| {
        asset_error("UDF f32 field is not four bytes")
    })?))
}

fn asset_error(message: impl Into<String>) -> RenderError {
    RenderError::InvalidConfig(format!("embedded dragon asset: {}", message.into()))
}

#[cfg(test)]
mod tests {
    use super::{ASSET_BYTES, DRAGON_FIELD_SHA256, DragonFieldAsset, HEADER_BYTES};
    use crate::sha256::digest_hex;

    #[test]
    fn embedded_volume_has_declared_layout_and_finite_samples() {
        let asset = DragonFieldAsset::embedded().expect("embedded field is valid");
        assert_eq!(asset.metadata.resolution, [128; 3]);
        assert_eq!(asset.payload.len(), 128 * 128 * 128 * 4);
        assert!(asset.metadata.conservative_error > 0.0);
        assert!(asset.metadata.minimum[1] < 0.0);
        assert!(asset.metadata.maximum[1] > 2.0);
        let digest = digest_hex(ASSET_BYTES);
        assert_eq!(digest.as_bytes(), DRAGON_FIELD_SHA256.as_bytes());
    }

    #[test]
    fn embedded_volume_faces_keep_clearance_beyond_declared_error() {
        let asset = DragonFieldAsset::embedded().expect("embedded field is valid");
        let resolution = asset.metadata.resolution[0] as usize;
        let sample = |x: usize, y: usize, z: usize| {
            let index = x + resolution * (y + resolution * z);
            let offset = index * 4;
            f32::from_ne_bytes(
                asset.payload[offset..offset + 4]
                    .try_into()
                    .expect("sample is four bytes"),
            )
        };
        let mut face_minimums = [f32::INFINITY; 6];
        for z in 0..resolution {
            for y in 0..resolution {
                face_minimums[0] = face_minimums[0].min(sample(0, y, z));
                face_minimums[1] = face_minimums[1].min(sample(resolution - 1, y, z));
            }
        }
        for z in 0..resolution {
            for x in 0..resolution {
                face_minimums[2] = face_minimums[2].min(sample(x, 0, z));
                face_minimums[3] = face_minimums[3].min(sample(x, resolution - 1, z));
            }
        }
        for y in 0..resolution {
            for x in 0..resolution {
                face_minimums[4] = face_minimums[4].min(sample(x, y, 0));
                face_minimums[5] = face_minimums[5].min(sample(x, y, resolution - 1));
            }
        }
        assert!(
            face_minimums
                .iter()
                .all(|distance| *distance > asset.metadata.conservative_error),
            "sampled volume boundary must not touch the dragon: {face_minimums:?}"
        );
    }

    #[test]
    fn parser_rejects_truncation_and_invalid_magic() {
        assert!(DragonFieldAsset::parse(&ASSET_BYTES[..HEADER_BYTES]).is_err());
        let mut invalid = ASSET_BYTES.to_vec();
        invalid[0] = b'X';
        assert!(DragonFieldAsset::parse(&invalid).is_err());
    }
}
