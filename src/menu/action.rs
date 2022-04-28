use super::AppState;
use autodefault::autodefault;
use bevy::ecs::system::Resource;
use bevy::prelude::*;
use enum_dispatch::enum_dispatch;
use paper_plugin::BoardOptions;
pub use paper_plugin::MatchRules;
use {super::materials::MenuMaterials, std::cmp::*, std::sync::Arc};

pub type Act<T> = dyn Fn(&mut T) + Send + Sync + 'static;
#[derive(Component)]
pub struct Action<T> {
    name: String,
    action: Box<Act<T>>,
}
impl<T: Resource> Action<T> {
    pub fn new<U>(name: String, action: U) -> Self
    where
        U: Fn(&mut T) + Send + Sync + 'static,
    {
        Self {
            name,
            action: Box::new(action),
        }
    }
}
#[derive(Component)]
pub struct LabelText<T>(Box<Label<T>>);
impl<T: Resource> LabelText<T> {
    pub fn new<U>(action: U) -> Self
    where
        U: Fn(&T) -> String + Send + Sync + 'static,
    {
        Self(Box::new(action))
    }
}
pub type Volume<T> = dyn Fn(&mut T, bool) + Send + Sync + 'static;
pub type Label<T> = dyn Fn(&T) -> String + Send + Sync + 'static;
pub struct Vol<T> {
    label: Box<Label<T>>,
    action: Box<Volume<T>>,
}
impl<T: Resource> Vol<T> {
    pub fn new<U, V>(label: V, action: U) -> Self
    where
        U: Fn(&mut T, bool) + Send + Sync + 'static,
        V: Fn(&T) -> String + Send + Sync + 'static,
    {
        Self {
            label: Box::new(label),
            action: Box::new(action),
        }
    }
}
#[enum_dispatch]
pub trait SpawnButtonWithAction {
    fn spawn_button(self, parent: &mut ChildBuilder, materials: &Res<MenuMaterials>);
}
//impl SpawnButtonWithAction
impl<T: Resource> SpawnButtonWithAction for Vol<T> {
    fn spawn_button(self, parent: &mut ChildBuilder, materials: &Res<MenuMaterials>) {
        let f = Arc::new(self);
        let m = f.clone();
        let t = f.clone();
        parent.spawn_bundle(materials.menu_td()).with_children(|p| {
            Action::new("+".to_string(), move |o: &mut T| (f.action)(o, true))
                .spawn_button(p, materials);
            p.spawn_bundle(materials.button_text("".to_string()))
                .insert(LabelText::new(move |o: &T| (t.label)(o)));
            Action::new("-".to_string(), move |o: &mut T| (m.action)(o, false))
                .spawn_button(p, materials);
        }).insert(Name::new("Volume Buttons"));
    }
}
impl<T: Resource> SpawnButtonWithAction for Action<T> {
    fn spawn_button(self, parent: &mut ChildBuilder, materials: &Res<MenuMaterials>) {
        let name = &self.name.clone();
        parent
            .spawn_bundle(materials.button_border())
            .insert(Name::new("Action"))
            .with_children(|p| {
                p.spawn_bundle(materials.button())
                    .insert(self)
                    .insert(Name::new(format!("Button({:?})", name)))
                    .with_children(|p| {
                        p.spawn_bundle(materials.button_text(name));
                    });
            });
    }
}
impl SpawnButtonWithAction for String {
    fn spawn_button(self, parent: &mut ChildBuilder, materials: &Res<MenuMaterials>) {
        parent.spawn_bundle(materials.button_text(self));
    }
}

#[enum_dispatch(SpawnButtonWithAction)]
pub enum ResourceMap {
    State(Action<State<AppState>>),
    Opts(Action<BoardOptions>),
    Vol(Vol<BoardOptions>),
    Text(String),
}
impl From<&str> for ResourceMap {
    fn from(text: &str) -> ResourceMap {
        ResourceMap::Text(text.to_string())
    }
}

/// Button action type
//#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub enum ButtonAction {
    Level,
    Mode(MatchRules),
    Human,
    Bot,
    Apply,
    Save,
    Menu,
}
pub use ButtonAction::*;
impl ButtonAction {
    pub fn name(&self) -> String {
        match self {
            Mode(x) => format!("{:?}", x),
            _ => format!("{:?}", self),
        }
    }
    #[autodefault]
    pub fn into(self) -> ResourceMap {
        let set = |i, x: u8, lb, ub| {
            if i == true {
                min(x + 1, ub)
            } else {
                max(lb, x.saturating_sub(1))
            }
        };
        match self {
            Apply => ResourceMap::State(Action::new(self.name(), |state: &mut State<AppState>| {
                if *state.current() == AppState::Menu {
                    state.overwrite_replace(AppState::InGame).unwrap();
                }
            })),
            Save => ResourceMap::State(Action::new(self.name(), |state: &mut State<AppState>| {
                if !state.inactives().is_empty() && *state.current() == AppState::Menu {
                    state.overwrite_pop().unwrap();
                }
            })),
            Menu => ResourceMap::State(Action::new(self.name(), |state: &mut State<AppState>| {
                if *state.current() == AppState::InGame {
                    state.overwrite_push(AppState::Menu).unwrap();
                }
            })),
            Mode(x) => {
                ResourceMap::Opts(Action::new(self.name(), move |opts: &mut BoardOptions| {
                    opts.mode.rule = x
                }))
            }
            Level => ResourceMap::Vol(Vol::new(
                |o: &BoardOptions| format!("Level: {}", o.level),
                move |opts: &mut BoardOptions, x| opts.level = set(x, opts.level, 0, 5),
            )),
            Human => ResourceMap::Vol(Vol::new(
                |o: &BoardOptions| format!("Human: {}", o.players.0),
                move |opts: &mut BoardOptions, x| opts.players.0 = set(x, opts.players.0, 1, 2),
            )),
            Bot => ResourceMap::Vol(Vol::new(
                |o: &BoardOptions| format!("Bot: {}", o.players.1),
                move |opts: &mut BoardOptions, x| opts.players.1 = set(x, opts.players.1, 0, 1),
            )),
        }
    }
}
pub fn asset_button_server<T: Resource>(
    button_colors: Res<MenuMaterials>,
    mut asset: ResMut<T>,
    mut interaction_query: Query<(&Interaction, &Action<T>, &mut UiColor), Changed<Interaction>>,
    mut labels: Query<(&LabelText<T>, &mut Text)>,
) {
    for (interaction, action, mut color) in interaction_query.iter_mut() {
        if *interaction == Interaction::Clicked {
            (action.action)(asset.as_mut());
        }
        *color = match *interaction {
            Interaction::Clicked => button_colors.pressed.into(),
            Interaction::Hovered => button_colors.hovered.into(),
            Interaction::None => button_colors.button.into(),
        }
    }
    for (label, mut text) in labels.iter_mut() {
        text.sections[0].value = (label.0)(&asset);
    }
}
