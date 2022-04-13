use bevy::prelude::*;
pub use bevy_tweening::{lens::*, *};

pub trait SetColor<T>: Component { fn set_color(&mut self, v: T); }
impl SetColor<Color> for UiColor {
    fn set_color(&mut self, v: Color) {
        self.0 = v
    }
}
/// A lens to manipulate the [`color`] field of a [`T`] asset.
/// [`color`]: https://docs.rs/bevy/0.6.1/bevy/sprite/struct.Sprite.html#structfield.color
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ColorLens {
    /// Start color.
    pub start: Color,
    /// End color.
    pub end: Color,
}
impl<T: SetColor<Color>+Component> Lens<T> for ColorLens {
    fn lerp(&mut self, target: &mut T, ratio: f32) {
        // Note: Add<f32> for Color affects alpha, but not Mul<f32>. So use Vec4 for consistency.
        let start: Vec4 = self.start.into();
        let end: Vec4 = self.end.into();
        let value = start.lerp(end, ratio);
        target.set_color(value.into());
    }
}
/// boolean to decide whether to show the component. true -> shows.
pub struct VisibilityLens(pub bool);
impl Lens<Visibility> for VisibilityLens {
    fn lerp(&mut self, target: &mut Visibility, ratio: f32) {
        target.is_visible = self.0 ^ (ratio < 0.5);
    }
}
pub struct TransformPositionLensByDelta(pub Vec3);
impl Lens<Transform> for TransformPositionLensByDelta {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        target.translation += self.0 * ratio;
    }
}
