//! dithered gradient washes as a rune element

use crate::rune_el::SurfaceTexture;
use dither_core::gradient::{GradientSpec, render_gradient};
use rune::El;

pub struct DitherGradient {
    spec: GradientSpec,
    size: (f32, f32),
    texture: SurfaceTexture,
}

impl DitherGradient {
    pub fn new(size: (f32, f32), spec: GradientSpec) -> Self {
        let (cols, rows) = spec.backing_size(size.0 as f64, size.1 as f64);
        let gradient = Self {
            spec,
            size,
            texture: SurfaceTexture::new(),
        };
        gradient
            .texture
            .upload_owned(render_gradient(cols, rows, &gradient.spec));
        gradient
    }

    pub fn spec(&self) -> &GradientSpec {
        &self.spec
    }

    pub fn view(&self) -> El {
        self.texture.el().size(self.size.0, self.size.1)
    }
}
