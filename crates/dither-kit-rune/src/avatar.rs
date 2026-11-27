//! deterministic generative avatars as a rune element

use crate::rune_el::SurfaceTexture;
use dither_core::Surface;
use dither_core::avatar::{AvatarMirror, AvatarModel, PIXELS, avatar_model, paint_avatar};
use dither_core::bayer::clamp01;
use rune::El;

pub struct Avatar {
    name: String,
    hue: Option<f64>,
    mirror: AvatarMirror,
    model: AvatarModel,
    texture: SurfaceTexture,
    size: f32,
    progress: f64,
    started: Option<f64>,
    animate: bool,
    duration: f64,
}

impl Avatar {
    pub fn new(name: impl Into<String>, size: f32) -> Self {
        let name = name.into();
        let mut avatar = Self {
            model: avatar_model(&name, None, AvatarMirror::Auto),
            hue: None,
            mirror: AvatarMirror::Auto,
            texture: SurfaceTexture::new(),
            size,
            progress: 0.0,
            started: None,
            animate: true,
            duration: 0.6,
            name,
        };
        avatar.repaint();
        avatar
    }

    pub fn hue(mut self, hue: f64) -> Self {
        self.hue = Some(hue);
        self.rebuild();
        self
    }

    pub fn mirror(mut self, mirror: AvatarMirror) -> Self {
        self.mirror = mirror;
        self.rebuild();
        self
    }

    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self.progress = if animate { self.progress } else { 1.0 };
        self.repaint();
        self
    }

    pub fn progress(&self) -> f64 {
        self.progress
    }

    pub fn tick(&mut self, clock: f64) -> bool {
        if !self.animate || self.progress >= 1.0 {
            return false;
        }
        let start = *self.started.get_or_insert(clock);
        let progress = clamp01((clock - start) / self.duration.max(0.001));
        if progress != self.progress {
            self.progress = progress;
            self.repaint();
        }
        progress < 1.0
    }

    pub fn finish(&mut self) -> bool {
        if self.progress < 1.0 {
            self.progress = 1.0;
            self.repaint();
            true
        } else {
            false
        }
    }

    pub fn replay(&mut self) {
        if self.animate {
            self.progress = 0.0;
            self.started = None;
            self.repaint();
        }
    }

    fn rebuild(&mut self) {
        self.model = avatar_model(&self.name, self.hue, self.mirror);
        self.repaint();
    }

    fn repaint(&mut self) {
        let mut surface = Surface::new(PIXELS, PIXELS);
        let eased = 1.0 - (1.0 - self.progress).powi(3);
        paint_avatar(&mut surface, &self.model, eased);
        self.texture.upload_owned(surface);
    }

    pub fn view(&self) -> El {
        self.texture
            .el()
            .size(self.size, self.size)
            .radius(self.size * 0.2)
            .role("img")
            .label(format!("{} avatar", self.name))
    }
}
