// avatar — same name, same glyph, every time
use dither_kit_rune::avatar::Avatar;

pub fn avatar(name: &str) -> Avatar {
    Avatar::new(name, 92.0)
}

// view: avatar.view()
// hue:  Avatar::new("ada", 92.0).hue(210.0)
// fold: .mirror(AvatarMirror::Horizontal)
