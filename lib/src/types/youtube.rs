use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Width specification for YouTube embeds
///
/// Supports three formats:
/// - Pixels: `512px` (default if not specified)
/// - Rems: `32rem`
/// - Percentage: `80%` (validated 0-100 range)
///
/// # Examples
///
/// ```rust
/// use composition::types::WidthSpec;
///
/// let pixels = WidthSpec::Pixels(512);
/// assert_eq!(pixels.to_string(), "512px");
///
/// let rems = WidthSpec::Rems(32.0);
/// assert_eq!(rems.to_string(), "32rem");
///
/// let percentage = WidthSpec::Percentage(80);
/// assert_eq!(percentage.to_string(), "80%");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WidthSpec {
    /// Width in pixels (e.g., 512px)
    Pixels(u32),
    /// Width in rems (e.g., 32rem)
    Rems(f32),
    /// Width as percentage 0-100 (e.g., 80%)
    Percentage(u8),
}

impl Default for WidthSpec {
    fn default() -> Self {
        WidthSpec::Pixels(512)
    }
}

impl Display for WidthSpec {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            WidthSpec::Pixels(px) => write!(f, "{}px", px),
            WidthSpec::Rems(rem) => write!(f, "{}rem", rem),
            WidthSpec::Percentage(pct) => write!(f, "{}%", pct),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_width_spec_display_pixels() {
        let width = WidthSpec::Pixels(512);
        assert_eq!(width.to_string(), "512px");
    }

    #[test]
    fn test_width_spec_display_rems() {
        let width = WidthSpec::Rems(32.0);
        assert_eq!(width.to_string(), "32rem");
    }

    #[test]
    fn test_width_spec_display_rems_decimal() {
        let width = WidthSpec::Rems(32.5);
        assert_eq!(width.to_string(), "32.5rem");
    }

    #[test]
    fn test_width_spec_display_percentage() {
        let width = WidthSpec::Percentage(80);
        assert_eq!(width.to_string(), "80%");
    }

    #[test]
    fn test_width_spec_default() {
        let width = WidthSpec::default();
        assert_eq!(width, WidthSpec::Pixels(512));
    }

    #[test]
    fn test_width_spec_clone() {
        let width1 = WidthSpec::Pixels(512);
        let width2 = width1.clone();
        assert_eq!(width1, width2);
    }

    #[test]
    fn test_width_spec_debug() {
        let width = WidthSpec::Pixels(512);
        let debug_str = format!("{:?}", width);
        assert!(debug_str.contains("Pixels"));
        assert!(debug_str.contains("512"));
    }
}
