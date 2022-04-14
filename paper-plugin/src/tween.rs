use bevy::prelude::*;
pub use bevy_tweening::{lens::*, *};

pub trait SetLens<T>: Component {
    fn set_lens(&mut self, v: T);
}
impl SetLens<Color> for UiColor {
    fn set_lens(&mut self, v: Color) {
        self.0 = v
    }
}
/// A lens to manipulate the [`color`] field of a [`T`] asset.
/// [`color`]: https://docs.rs/bevy/0.6.1/bevy/sprite/struct.Sprite.html#structfield.color
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ColorLens<T: SetLens<Color>> {
    /// Start color.
    pub start: Color,
    /// End color.
    pub end: Color,
    /// phantom
    phantom: std::marker::PhantomData<T>,
}
impl<T: SetLens<Color>> ColorLens<T> {
    pub fn new(start: Color, end:Color) -> Self {
        Self { start, end, phantom: std::marker::PhantomData}
    }
}
impl<T: SetLens<Color> + Component> Lens<T> for ColorLens<T> {
    fn lerp(&mut self, target: &mut T, ratio: f32) {
        // Note: Add<f32> for Color affects alpha, but not Mul<f32>. So use Vec4 for consistency.
        let start: Vec4 = self.start.into();
        let end: Vec4 = self.end.into();
        let value = start.lerp(end, ratio);
        target.set_lens(value.into());
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
