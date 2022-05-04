mod action;
mod materials;
pub use {action::*, materials::MenuMaterials};
use {
    bevy::{
        //ecs::system::Resource,
        prelude::*},
    duplicate::duplicate_item,
    std::cmp::{max, min},
};
pub trait MenuItem {
    type Attributes: Default + Clone + Component;

    fn ui(
        parent: &mut ChildBuilder,
        materials: &Res<MenuMaterials>,
        opts: Self::Attributes,
    ) -> Entity;

    #[allow(unused_variables)]
    fn system(&mut self, entity: Entity, world: &mut World);
}
#[derive(Debug, Clone, Component)]
pub struct Volume<T> {
    pub min: Option<T>,
    pub max: Option<T>,
    pub speed: f32,
    pub vertical: bool,
    pub prefix: String,
    pub suffix: String,
}
impl<T> Default for Volume<T> {
    fn default() -> Self {
        Volume {
            min: None,
            max: None,
            speed: 0.0,
            vertical: true,
            prefix: "".to_string(),
            suffix: "".to_string(),
        }
    }
}
#[duplicate_item(t; [u8];)]
impl MenuItem for t {
    type Attributes = Volume<t>;
    fn ui(p: &mut ChildBuilder, m: &Res<MenuMaterials>, opts: Volume<t>) -> Entity {
        p.spawn_bundle(if opts.vertical {
            m.menu_td()
        } else {
            m.menu_lr()
        })
        .with_children(|p| {
            p.spawn_bundle(m.button_border()).with_children(|p| {
                p.spawn_bundle(m.button()).with_children(|p| {
                    p.spawn_bundle(m.button_text("+"));
                });
            });
            p.spawn_bundle(m.button_text(""));
            p.spawn_bundle(m.button_border()).with_children(|p| {
                p.spawn_bundle(m.button()).with_children(|p| {
                    p.spawn_bundle(m.button_text("-"));
                });
            });
        })
        .insert(opts)
        .id()
    }
    fn system(&mut self, e: Entity, world: &mut World) {
        let o = world.entity(e).get::<Volume<t>>().unwrap().clone();
        let mut ch = world.query::<&mut Children>();
        let mut chv: Vec<Entity> = ch.get(world, e).unwrap()[..].to_vec();
        chv[0] = ch.get(world, chv[0]).unwrap()[0];
        chv[2] = ch.get(world, chv[2]).unwrap()[0];
        if world.entity(chv[0]).get::<Interaction>().unwrap() == &Interaction::Clicked {
            *self = min(o.max.unwrap_or(t::MAX), *self + o.speed as t);
        } else if world.entity(chv[0]).get::<Interaction>().unwrap() == &Interaction::Clicked {
            *self = max(o.min.unwrap_or(t::MIN), *self - o.speed as t);
        }
    }
}


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum MenuState {
    InGame,
    Menu,
    Disabled,
}
use MenuState::*;
pub struct MenuPlugin<T: MenuItem>(std::marker::PhantomData<T>);
impl<T:MenuItem> MenuPlugin<T> {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(SystemSet::on_enter(InGame).with_system(setup_ui))
            .add_system_set(SystemSet::on_resume(InGame).with_system(setup_ui))
            .add_system_set(SystemSet::on_exit(InGame).with_system(despawn::<UI>))
            .add_system_set(
                SystemSet::on_enter(Menu)
                    .with_system(setup_menu)
                    .with_system(despawn::<UI>),
            )
            .add_system_set(SystemSet::on_exit(Menu).with_system(despawn::<MenuUI>));
    }
}
pub struct UI;
pub struct MenuUI;
fn setup_ui(){}
fn setup_menu(){}
fn despawn<T>(){}
