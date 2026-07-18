use std::error::Error;
use std::fmt;

/// Scale and operation-count inputs used to derive floating-point guards.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ErrorScale {
    characteristic_length: f32,
    operation_count: u16,
}

impl ErrorScale {
    /// Builds a finite, positive error scale.
    ///
    /// # Errors
    ///
    /// Returns [`MathError::InvalidScale`] when length is non-finite or not
    /// positive, or when the operation count is zero.
    pub fn new(characteristic_length: f32, operation_count: u16) -> Result<Self, MathError> {
        if !characteristic_length.is_finite() || characteristic_length <= 0.0 {
            return Err(MathError::InvalidScale);
        }
        if operation_count == 0 {
            return Err(MathError::InvalidOperationCount);
        }
        Ok(Self {
            characteristic_length,
            operation_count,
        })
    }

    /// Conservative linear roundoff guard for this operation graph.
    #[must_use]
    pub fn linear_guard(self) -> f32 {
        self.characteristic_length * f32::EPSILON * f32::from(self.operation_count)
    }

    /// Squared guard suitable for a squared-length comparison.
    #[must_use]
    pub fn squared_length_guard(self) -> f32 {
        let linear_guard = self.linear_guard();
        linear_guard * linear_guard
    }
}

/// Recoverable failures at Radia math boundaries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MathError {
    NonFinite(&'static str),
    Degenerate(&'static str),
    InvalidScale,
    InvalidOperationCount,
    InvalidProjection(&'static str),
    OutsideProjectionDomain,
    PixelOutsideExtent,
}

impl fmt::Display for MathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite(value) => write!(formatter, "{value} must be finite"),
            Self::Degenerate(value) => write!(formatter, "{value} is degenerate at this scale"),
            Self::InvalidScale => write!(
                formatter,
                "characteristic length must be finite and positive"
            ),
            Self::InvalidOperationCount => write!(formatter, "operation count must be positive"),
            Self::InvalidProjection(value) => {
                write!(formatter, "invalid perspective parameter: {value}")
            }
            Self::OutsideProjectionDomain => write!(formatter, "point is on or behind the camera"),
            Self::PixelOutsideExtent => {
                write!(formatter, "pixel lies outside the framebuffer extent")
            }
        }
    }
}

impl Error for MathError {}
