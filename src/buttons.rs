use crate::AppState;
use autodefault::autodefault;
use bevy::log;
use bevy::prelude::*;
use paper_plugin::BoardOptions;

/// Button action type
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component)]
pub enum ButtonAction {
    //Clear,
    LevelUp,
    LevelDown,
    CoupletUp,
    CoupletDown,
    Generate,
    Mode,
    Menu,
}

impl ButtonAction {
    fn name(&self) -> String {
        /*
        match self {
            Self::LevelUp=>"LevelUp".to_string(),
            Self::LevelDown=>"LevelDown".to_string(),
            Self::CoupletUp=>"CoupletUp".to_string(),
            Self::CoupletDown=>"CoupletDown".to_string(),
            Self::Generate=>"Generate".to_string(),
            Self::Mode=>"Mode".to_string(),
        }
        */
        format!("{:?}", self)
    }
}

#[derive(Default, Debug)]
pub struct ButtonColors {
    //none: Color,
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
    //font: Handle<Font>,
}

pub struct ButtonMaterials {
    none: Color,
    normal: Color,
    hovered: Color,
    pressed: Color,
    font: Handle<Font>,
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();

        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        ButtonMaterials {
            none: Color::NONE,
            normal: Color::rgb(0.15, 0.15, 0.15),
            hovered: Color::rgb(0.25, 0.25, 0.25),
            pressed: Color::rgb(0.35, 0.75, 0.35),
            //font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font: asset_server.load("fonts/pixeled.ttf"),
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn button_system(
    button_colors: Res<ButtonMaterials>,
    mut interaction_query: Query<
        (&Interaction, &ButtonAction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut board_options: ResMut<BoardOptions>,
    mut state: ResMut<State<AppState>>,
) {
    for (interaction, action, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = button_colors.pressed.into();
                log::debug!("{:?}", action);
                match action {
                    /*
                    ButtonAction::Clear => {
                    match state.current() {
                    AppState::InGame => state.set(AppState::Splash).unwrap(),
                    AppState::Menu => state.replace(AppState::Splash).unwrap(),
                    _ => (),
                    }
                    }
                    */
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
                    ButtonAction::Generate => match state.current() {
                        AppState::Menu => state.overwrite_pop().unwrap(),
                        _ => (),
                    },
                    ButtonAction::Menu => match state.current() {
                        AppState::InGame => state.overwrite_push(AppState::Menu).unwrap(),
                        _ => {}
                    },
                    _ => (),
                }
            }
            Interaction::Hovered => {
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                *color = button_colors.normal.into();
            }
        }
    }
}

#[derive(Component)]
struct MenuUI;
#[autodefault]
fn setup_menu(mut commands: Commands, button_materials: Res<ButtonMaterials>) {
    // Make list of buttons
    let buttons: Vec<ButtonAction> = vec![
        ButtonAction::LevelUp,
        ButtonAction::LevelDown,
        ButtonAction::CoupletUp,
        ButtonAction::CoupletDown,
        ButtonAction::Generate,
        ButtonAction::Mode,
    ];
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::FlexStart,
            },
            color: button_materials.none.into(),
            //color: Color::BLACK.into(),
        })
        .insert(MenuUI)
        .insert(Name::new("MenuUI"))
        .with_children(|parent| {
            for button in buttons {
                parent
                    .spawn_bundle(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(350.0), Val::Px(65.0)),
                            margin: Rect::all(Val::Auto),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                        },
                        color: button_materials.normal.into(),
                    })
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle {
                            text: Text::with_section(
                                button.name(),
                                TextStyle {
                                    font: button_materials.font.clone(),
                                    font_size: 20.0,
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                                Default::default(),
                            ),
                        });
                    })
                    .insert(button);
            }
        });
}

fn despawn_menu(mut commands: Commands, query: Query<(Entity, &MenuUI)>) {
    for (entity, _) in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[autodefault]
fn setup_ui(mut commands: Commands, button_materials: Res<ButtonMaterials>) {
    let font = &button_materials.font;
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Px(150.)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Absolute,
                position: Rect { top: Val::Px(0.) },
            },
            color: Color::BLACK.into(),
        })
    .insert(Name::new("Instructions"))
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![
                        write_strings("Instructions:",27.,Color::WHITE,font.clone()),
                        write_strings("1. Match a card with its couplets",23.,Color::WHITE,font.clone()),
                        write_strings("if couplet is two, then two cards have same value which can be matched",17.,Color::WHITE,font.clone()),
                    ],
                    alignment: TextAlignment {
                        vertical: VerticalAlign::Center,
                        horizontal: HorizontalAlign::Left,
                    },
                },
            });
        });
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Px(50.)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
            },
            color: Color::WHITE.into(),
        })
        .insert(Name::new("UI"))
        .with_children(|parent| {
            setup_single_menu(
                parent,
                &ButtonAction::Menu.name(),
                button_materials.normal.into(),
                font.clone(),
                ButtonAction::Menu,
            );
        });
}

#[autodefault]
fn setup_single_menu(
    parent: &mut ChildBuilder,
    text: &str,
    color: UiColor,
    font: Handle<Font>,
    action: ButtonAction,
) {
    parent
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Percent(95.), Val::Auto),
                margin: Rect::all(Val::Px(10.)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            },
            color,
        })
        .insert(action)
        .insert(Name::new(text.to_string()))
        .with_children(|builder| {
            builder.spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: text.to_string(),
                        style: TextStyle {
                            font,
                            font_size: 27.,
                            color: Color::WHITE,
                        },
                    }],
                    alignment: TextAlignment {
                        vertical: VerticalAlign::Center,
                        horizontal: HorizontalAlign::Center,
                    },
                },
            });
        });
}
fn write_strings(text: &str, font_size: f32, color: Color, font: Handle<Font>) -> TextSection {
    TextSection {
        value: (text.to_owned() + "\n").to_string(),
        style: TextStyle {
            font,
            font_size,
            color,
        },
    }
}

use bevy::ecs::schedule::StateData;
pub struct MenuPlugin<T>(pub T);
impl<T: StateData> Plugin for MenuPlugin<T> {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonMaterials>()
            .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup_ui))
            .add_system_set(SystemSet::on_enter(self.0.clone()).with_system(setup_menu))
            //.add_system_set(SystemSet::on_update(self.0.clone()) .with_system(button_system))
            .add_system(button_system)
            .add_system_set(SystemSet::on_exit(self.0.clone()).with_system(despawn_menu));
    }
}
