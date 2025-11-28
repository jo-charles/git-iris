//! Gradient color interpolation.

use super::color::ThemeColor;

/// A color gradient defined by a series of color stops.
#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    /// The colors that make up this gradient.
    stops: Vec<ThemeColor>,
}

impl Gradient {
    /// Create a new gradient from a list of color stops.
    ///
    /// # Panics
    /// Panics if `stops` is empty.
    #[must_use]
    pub fn new(stops: Vec<ThemeColor>) -> Self {
        assert!(!stops.is_empty(), "gradient must have at least one color");
        Self { stops }
    }

    /// Get the interpolated color at position `t` (0.0 to 1.0).
    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::as_conversions
    )]
    pub fn at(&self, t: f32) -> ThemeColor {
        let t = t.clamp(0.0, 1.0);

        if self.stops.len() == 1 {
            return self.stops[0];
        }

        let scaled = t * (self.stops.len() - 1) as f32;
        let idx = scaled.floor() as usize;
        let local_t = scaled - scaled.floor();

        if idx >= self.stops.len() - 1 {
            self.stops[self.stops.len() - 1]
        } else {
            self.stops[idx].lerp(&self.stops[idx + 1], local_t)
        }
    }

    /// Get the number of color stops in this gradient.
    #[must_use]
    pub fn len(&self) -> usize {
        self.stops.len()
    }

    /// Check if the gradient has only one color.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stops.is_empty()
    }

    /// Generate a smooth gradient with `n` evenly spaced colors.
    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        clippy::as_conversions
    )]
    pub fn generate(&self, n: usize) -> Vec<ThemeColor> {
        if n == 0 {
            return vec![];
        }
        if n == 1 {
            return vec![self.stops[0]];
        }

        (0..n)
            .map(|i| {
                let t = i as f32 / (n - 1) as f32;
                self.at(t)
            })
            .collect()
    }
}

impl Default for Gradient {
    fn default() -> Self {
        Self {
            stops: vec![ThemeColor::FALLBACK],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_endpoints() {
        let gradient = Gradient::new(vec![
            ThemeColor::new(0, 0, 0),
            ThemeColor::new(255, 255, 255),
        ]);

        assert_eq!(gradient.at(0.0), ThemeColor::new(0, 0, 0));
        assert_eq!(gradient.at(1.0), ThemeColor::new(255, 255, 255));
    }

    #[test]
    fn test_gradient_midpoint() {
        let gradient = Gradient::new(vec![
            ThemeColor::new(0, 0, 0),
            ThemeColor::new(255, 255, 255),
        ]);

        let mid = gradient.at(0.5);
        assert_eq!(mid, ThemeColor::new(127, 127, 127));
    }

    #[test]
    fn test_gradient_multi_stop() {
        let gradient = Gradient::new(vec![
            ThemeColor::new(255, 0, 0),   // red
            ThemeColor::new(0, 255, 0),   // green
            ThemeColor::new(0, 0, 255),   // blue
        ]);

        assert_eq!(gradient.at(0.0), ThemeColor::new(255, 0, 0));
        assert_eq!(gradient.at(0.5), ThemeColor::new(0, 255, 0));
        assert_eq!(gradient.at(1.0), ThemeColor::new(0, 0, 255));
    }

    #[test]
    fn test_generate() {
        let gradient = Gradient::new(vec![
            ThemeColor::new(0, 0, 0),
            ThemeColor::new(255, 255, 255),
        ]);

        let colors = gradient.generate(3);
        assert_eq!(colors.len(), 3);
        assert_eq!(colors[0], ThemeColor::new(0, 0, 0));
        assert_eq!(colors[1], ThemeColor::new(127, 127, 127));
        assert_eq!(colors[2], ThemeColor::new(255, 255, 255));
    }
}
