use super::AppState;
use autodefault::autodefault;
use bevy::ecs::system::Resource;
use bevy::prelude::*;
use paper_plugin::BoardOptions;
pub use paper_plugin::Mode;
use enum_dispatch::enum_dispatch;

pub type Act<T> = dyn Fn(&mut T) + Send + Sync + 'static;
#[derive(Component)]
pub struct ButtonAct<T> {
    name: String,
    action: Box<Act<T>>,
}
impl<T: Resource> ButtonAct<T> {
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
#[enum_dispatch]
pub trait SpawnButtonWithAction {
    fn spawn_button(self, parent: &mut ChildBuilder, materials: &Res<MenuMaterials>);
}
//impl SpawnButtonWithAction
impl<T: Resource> SpawnButtonWithAction for ButtonAct<T> {
    fn spawn_button(self, parent: &mut ChildBuilder, materials: &Res<MenuMaterials>) {
        let name = &self.name.clone();
        parent
            .spawn_bundle(button_border(materials))
            .with_children(|p| {
                p.spawn_bundle(button(materials))
                    .insert(self)
                    .insert(Name::new(format!("Button({:?})", name)))
                    .with_children(|p| {
                        p.spawn_bundle(button_text(materials, name));
                    });
            });
    }
}
#[enum_dispatch(SpawnButtonWithAction)]
pub enum ResourceMap{
    State(ButtonAct<State<AppState>>),
    Opts(ButtonAct<BoardOptions>),
    //PhantomData(std::marker::PhantomData<T>)
}
pub fn asset_button_server<T: Resource>(
    button_colors: Res<MenuMaterials>,
    mut asset: ResMut<T>,
    mut interaction_query: Query<(&Interaction, &ButtonAct<T>, &mut UiColor), Changed<Interaction>>,
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
}

pub struct MenuMaterials {
    pub none: UiColor,
    pub root: UiColor,
    pub menu: UiColor,
    pub border: UiColor,
    pub button: UiColor,
    pub hovered: UiColor,
    pub pressed: UiColor,
    pub font: Handle<Font>,
    pub button_border: UiColor,
    pub button_text: UiColor,
}
impl FromWorld for MenuMaterials {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        MenuMaterials {
            none: Color::NONE.into(),
            root: Color::rgba(0., 0., 0., 0.27).into(),
            menu: Color::rgb(0.15, 0.15, 0.15).into(),
            border: Color::rgb(0.65, 0.65, 0.65).into(),
            button_border: Color::rgb(0.81, 0.65, 0.65).into(),
            button: Color::rgb(0.15, 0.15, 0.15).into(),
            hovered: Color::rgb(0.25, 0.25, 0.25).into(),
            pressed: Color::rgb(0.35, 0.75, 0.35).into(),
            //font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font: asset_server.load("fonts/pixeled.ttf"),
            button_text: Color::rgb(0.9, 0.9, 0.9).into(),
        }
    }
}
/// Button action type
//#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub enum ButtonAction {
    LevelUp,
    LevelDown,
    Apply,
    Save,
    //#[Inspectable]
    Mode(Mode),
    Menu,
}
pub use ButtonAction::*;
impl ButtonAction {
    pub fn name(&self) -> String {
        match self {
            ButtonAction::Mode(x) => format!("{:?}", x),
            _ => format!("{:?}", self),
        }
    }
    #[autodefault]
    pub fn spawn_button(self, parent: &mut ChildBuilder, materials: &Res<MenuMaterials>) {
        match self {
            Apply => ResourceMap::State(ButtonAct::new(self.name(), |state: &mut State<AppState>| {
                if *state.current() == AppState::Menu {
                    state.overwrite_replace(AppState::InGame).unwrap();
                }
            })),
            Save => ResourceMap::State(ButtonAct::new(self.name(), |state: &mut State<AppState>| {
                if !state.inactives().is_empty() && *state.current() == AppState::Menu {
                    state.overwrite_pop().unwrap();
                }
            })),
            Menu => ResourceMap::State(ButtonAct::new(self.name(), |state: &mut State<AppState>| {
                if *state.current() == AppState::InGame {
                    state.overwrite_push(AppState::Menu).unwrap();
                }
            })),
            LevelUp => ResourceMap::Opts(ButtonAct::new(self.name(), |opts: &mut BoardOptions| opts.level_up())),
            LevelDown => ResourceMap::Opts(ButtonAct::new(self.name(), |opts: &mut BoardOptions| opts.level_down())),
            Mode(x) => ResourceMap::Opts(ButtonAct::new(self.name(), move |opts: &mut BoardOptions| opts.mode = x)),
        }.spawn_button(parent, materials);
    }
}
pub fn root(materials: &Res<MenuMaterials>) -> NodeBundle {
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            flex_direction: FlexDirection::ColumnReverse,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        color: materials.root.clone(),
        ..Default::default()
    }
}
pub fn button_border(materials: &Res<MenuMaterials>) -> NodeBundle {
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Px(50.)),
            border: Rect::all(Val::Px(3.0)),
            ..Default::default()
        },
        color: materials.button_border.clone(),
        ..Default::default()
    }
}
pub fn border(materials: &Res<MenuMaterials>) -> NodeBundle {
    NodeBundle {
        style: Style {
            size: Size::new(Val::Px(400.0), Val::Auto),
            border: Rect::all(Val::Px(8.0)),
            ..Default::default()
        },
        color: materials.border.clone(),
        ..Default::default()
    }
}
pub fn button(materials: &Res<MenuMaterials>) -> ButtonBundle {
    ButtonBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            justify_content: JustifyContent::Center,
            //size: Size::new(Val::Px(350.0), Val::Px(65.0)),
            //margin: Rect::all(Val::Auto),
            //size: Size::new(Val::Percent(95.), Val::Auto),
            //margin: Rect::all(Val::Px(10.)),
            align_items: AlignItems::Center,
            ..Default::default()
        },
        color: materials.button,
        ..Default::default()
    }
}
pub fn menu_background(
    materials: &Res<MenuMaterials>,
    flex_direction: FlexDirection,
) -> NodeBundle {
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: Rect::all(Val::Px(5.0)),
            flex_direction,
            ..Default::default()
        },
        color: materials.menu.clone(),
        ..Default::default()
    }
}
pub fn menu_lr(materials: &Res<MenuMaterials>) -> NodeBundle {
    menu_background(materials, FlexDirection::RowReverse)
}
pub fn menu_td(materials: &Res<MenuMaterials>) -> NodeBundle {
    menu_background(materials, FlexDirection::ColumnReverse)
}
pub fn button_text<S: Into<String>>(materials: &Res<MenuMaterials>, label: S) -> TextBundle {
    TextBundle {
        style: Style {
            margin: Rect::all(Val::Px(10.0)),
            ..Default::default()
        },
        text: Text::with_section(
            label.into(),
            TextStyle {
                font: materials.font.clone(),
                font_size: 30.0,
                color: materials.button_text.0,
            },
            Default::default(),
        ),
        ..Default::default()
    }
}
