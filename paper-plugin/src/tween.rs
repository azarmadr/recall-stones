use bevy::prelude::*;
pub use bevy_tweening::{lens::*, *};

pub type Lerp<T> = dyn Fn(&mut T, &T, f32)+ Send + Sync + 'static;
pub struct BeTween<T> {
    lerp: Box<Lerp<T>>,
    start: Option<T>,
}
impl<T> BeTween<T> {
    /// Construct a lens from a pair of getter functions
    pub fn  with_lerp<U>(lerp: U) -> Self
        where U: Fn(&mut T, &T, f32)+ Send + Sync + 'static,
    {
        Self {
            lerp: Box::new(lerp),
            start: None,
        }
    }
}
impl<T: Clone> Lens<T> for BeTween<T>
where
    T: Component,
{
    fn lerp(&mut self, target: &mut T, ratio: f32) {
        if self.start.is_none() {
            self.start = Some(target.clone());
        }
        if let Some(start) = &self.start {
            (self.lerp)(target, start, ratio);
        }
    }
}
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
        BeTween::with_lerp(move |c: &mut Visibility, _, r| c.is_visible = show ^ (r<0.5))
    )
}
pub fn shake_seq(duration: std::time::Duration) -> Sequence<Transform> {
    let tween = |x, i| {
        Tween::new(
            EaseFunction::ElasticInOut,
            TweeningType::Once,
            duration * i / 3,
            BeTween::with_lerp(move
                |c: &mut Transform, s: &Transform, r: f32| c.translation = s.translation + x * r,)
        )
    };
    Sequence::new((1..4).rev().map(|i| {
        tween(Vec3::X / 3. * i as f32, i)
            .then(tween(Vec3::X / 3. * -2. * i as f32, i))
            .then(tween(Vec3::X / 3. * i as f32, i))
    }))
}
