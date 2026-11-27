//! Rune surface adapter: engine buffers become nearest-neighbor `Pixels`.

use dither_core::Surface;
use rune::{El, ImageFilter, Pixels, el};

/// A `Pixels` handle kept across frames. Uploading reuses the handle, so Rune
/// updates the atlas slot instead of allocating a new image.
#[derive(Clone, Debug, Default)]
pub struct SurfaceTexture {
    pixels: Pixels,
}

impl SurfaceTexture {
    pub fn new() -> Self {
        Self {
            pixels: Pixels::new(),
        }
    }

    pub fn pixels(&self) -> &Pixels {
        &self.pixels
    }

    /// Upload straight-alpha sRGB bytes from a painted surface.
    pub fn upload(&self, surface: &Surface) {
        self.pixels
            .set(surface.cols(), surface.rows(), surface.rgba().to_vec());
    }

    /// Upload by taking the buffer, avoiding a copy on one-shot renders.
    pub fn upload_owned(&self, surface: Surface) {
        self.pixels
            .set(surface.cols(), surface.rows(), surface.into_rgba());
    }

    /// The crisp, pixel-snapped element showing this surface.
    pub fn el(&self) -> El {
        el().pixels(&self.pixels).image_filter(ImageFilter::Nearest)
    }
}
