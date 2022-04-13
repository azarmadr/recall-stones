use super::AppState;
use autodefault::autodefault;
use bevy::log;
use bevy::prelude::*;

use paper_plugin::BoardOptions;
pub use paper_plugin::Mode;

pub struct MenuMaterials {
    pub none: UiColor,
    pub root: UiColor,
    pub menu: UiColor,
    pub border: UiColor,
    pub button: UiColor,
    pub hovered: UiColor,
    pub pressed: UiColor,
    pub font: Handle<Font>,
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
            button: Color::rgb(0.15, 0.15, 0.15).into(),
            hovered: Color::rgb(0.25, 0.25, 0.25).into(),
            pressed: Color::rgb(0.35, 0.75, 0.35).into(),
            //font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font: asset_server.load("fonts/pixeled.ttf"),
            button_text: Color::rgb(0.9, 0.9, 0.9).into(),
        }
    }
}

pub fn button_system(
    button_colors: Res<MenuMaterials>,
    mut buttons: Query<(&Interaction, &mut UiColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut color) in buttons.iter_mut() {
        *color = match *interaction {
            Interaction::Clicked => button_colors.pressed.into(),
            Interaction::Hovered => button_colors.hovered.into(),
            Interaction::None => button_colors.button.into(),
        }
    }
}

/// Button action type
//#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub enum ButtonAction {
    //Clear,
    LevelUp,
    LevelDown,
    CoupletUp,
    CoupletDown,
    Apply,
    Save,
    //#[Inspectable]
    Mode(Mode),
    Menu,
}
impl ButtonAction {
    pub fn name(&self) -> String {
        match self {
            ButtonAction::Mode(x) => format!("{:?}", x),
            _ => format!("{:?}", self),
        }
    }
    #[autodefault]
    pub fn create_button(&self, parent: &mut ChildBuilder, materials: &Res<MenuMaterials>) {
        parent
            .spawn_bundle(button(materials))
            .insert(*self)
            .insert(Name::new("Button"))
            .with_children(|p| {
                p.spawn_bundle(button_text(materials, &self.name()));
            });
    }
}

pub fn action_system(
    mut interaction_query: Query<
        (&Interaction, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut board_options: ResMut<BoardOptions>,
    mut state: ResMut<State<AppState>>,
) {
    for (interaction, action) in interaction_query.iter_mut() {
        log::debug!("{:?}", action);
        if *interaction == Interaction::Clicked {
            match action {
                ButtonAction::LevelUp => board_options.level_up(),
                ButtonAction::LevelDown => board_options.level_down(),
                ButtonAction::CoupletUp => {
                    if board_options.couplets < 5 {
                        board_options.couplets += 1;
                    }
                }
                ButtonAction::CoupletDown => {
                    if board_options.couplets > 2 {
                        board_options.couplets -= 1;
                    }
                }
                ButtonAction::Save => match state.current() {
                    AppState::Menu => state.overwrite_pop().unwrap(),
                    _ => (),
                },
                ButtonAction::Apply => match state.current() {
                    AppState::Menu => state.overwrite_replace(AppState::InGame).unwrap(),
                    _ => (),
                },
                ButtonAction::Menu => match state.current() {
                    AppState::InGame => state.overwrite_push(AppState::Menu).unwrap(),
                    _ => {}
                },
                ButtonAction::Mode(x) => board_options.mode = *x,
            }
        }
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
pub fn button_text(materials: &Res<MenuMaterials>, label: &str) -> TextBundle {
    TextBundle {
        style: Style {
            margin: Rect::all(Val::Px(10.0)),
            ..Default::default()
        },
        text: Text::with_section(
            label,
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
