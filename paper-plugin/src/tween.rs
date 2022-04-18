use bevy::prelude::*;
pub use bevy_tweening::{lens::*, *};

pub struct BeTween<T, U, Get, GetMut> {
    get: Get,
    get_mut: GetMut,
    start: Option<U>,
    delta: U,
    _phantom: std::marker::PhantomData<T>,
}
impl<T, U, Get, GetMut> BeTween<T, U, Get, GetMut>
where
    U: Copy,
    Get: Fn(&T) -> &U,
    GetMut: Fn(&mut T) -> &mut U,
{
    /// Construct a lens from a pair of getter functions
    pub fn delta(get: Get, get_mut: GetMut, delta: U) -> Self
    {
        Self {
            get,
            get_mut,
            delta,
            start: None,
            _phantom: std::marker::PhantomData,
        }
    }
    pub fn from_start(get: Get, get_mut: GetMut, start: U, delta: U) -> Self
    {
        Self {
            get,
            get_mut,
            delta,
            start: Some(start),
            _phantom: std::marker::PhantomData,
        }
    }
    fn get_start(&mut self, target: &T) {
        if self.start.is_none() {
            self.start = Some(*(self.get)(target));
        }
    }
}
impl<T, U, Get, GetMut> Lens<T> for BeTween<T, U, Get, GetMut>
where
    T: Component,
    U: Copy,
    Get: Fn(&T) -> &U,
    GetMut: Fn(&mut T) -> &mut U,
    U: std::ops::Mul<f32, Output = U> + std::ops::Add<Output = U>,
{
    fn lerp(&mut self, target: &mut T, ratio: f32) {
        self.get_start(target);
        if let Some(start) = &self.start {
            let curr = (self.get_mut)(target);
            *curr = *start + self.delta * ratio;
        }
    }
}
pub trait SetLens<T>: Component {
    fn set_lens(&mut self, v: T);
    fn get_lens(&mut self) -> T;
}
impl SetLens<Color> for UiColor {
    fn set_lens(&mut self, v: Color) {
        self.0 = v
    }
    fn get_lens(&mut self) -> Color {
        self.0
    }
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ColorLens {
    /// Start color.
    pub start: Color,
    /// End color.
    pub end: Color,
}
impl ColorLens {
    pub fn new(start: Color, end: Color) -> Self {
        Self { start, end }
    }
}
impl<T: SetLens<Color> + Component> Lens<T> for ColorLens {
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
/* Phew nice learning
pub struct ByDelta<T>{
    pub delta: T,
    start: T,
    end: T,
}
impl<T: Default> ByDelta<T>{
    pub fn new(delta: T) -> Self {
        ByDelta::<T>{
            delta,
            start: Default::default(),
            end: Default::default(),
        }
    }
}
impl<T: SetLens<W>, W: Lerp<Scalar =f32>+Copy+ PartialEq + std::ops::Add<Output = W>> Lens<T> for ByDelta<W> {
    fn lerp(&mut self, target: &mut T, ratio: f32) {
        if self.start == self.end {
            self.start = target.get_lens();
            self.end = self.start + self.delta;
        }
        target.set_lens(self.start.lerp(&self.end, &ratio));
    }
}
pub struct Trnslate(pub Vec3);
impl Lerp for Trnslate{
    type Scalar = f32;
    fn lerp(&self, other: &self, scalar: &Self::Scalar) {
        self.0.lerp(other.0, scalar);
    }
}
impl SetLens<Vec3> for Transform {
    fn set_lens(&mut self, v:Vec3) { self.translation = v }
    fn get_lens(&mut self) ->Vec3{ self.translation }
}
*/
pub fn rot_seq(duration: std::time::Duration) -> Sequence<Transform> {
    let start = 0.;
    let end = std::f32::consts::PI / 2.;
    let tween = |start, end| {
        Tween::new(
            EaseFunction::QuadraticIn,
            TweeningType::Once,
            duration,
            TransformRotateYLens { start, end },
        )
    };
    tween(start, end).then(tween(end, start))
}
pub fn vis_seq(duration: std::time::Duration,show: bool) -> Tween<Visibility> {
    Tween::new(
        EaseFunction::QuadraticIn,
        TweeningType::Once,
        2 * duration,
        VisibilityLens(show),
    )
}
pub fn shake_seq(duration: std::time::Duration) -> Sequence<Transform> {
    let tween = |x, i| {
        Tween::new(
            EaseFunction::ElasticInOut,
            TweeningType::Once,
            duration * i / 3,
            BeTween::delta(
                |x: &Transform| &x.translation,
                |x: &mut Transform| &mut x.translation,
                 x),
        )
    };
    Sequence::new((1..4).rev().map(|i| {
        tween(Vec3::X / 3. * i as f32, i)
            .then(tween(Vec3::X / 3. * -2. * i as f32, i))
            .then(tween(Vec3::X / 3. * i as f32, i))
    }))
}
