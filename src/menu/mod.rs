use super::AppState;
use autodefault::autodefault;
use bevy::prelude::*;
use paper_plugin::MatchRules::*;

mod materials;
pub use materials::*;
use paper_plugin::{BoardOptions, ScoreBoard};
fn despawn<T: Component>(mut cmd: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
struct MenuUI;
#[derive(Component)]
#[component(storage = "SparseSet")]
struct MenuBoardOptions;
#[autodefault]
fn setup_menu(mut cmd: Commands, materials: Res<MenuMaterials>) {
    // Make list of buttons
    let buttons: Vec<Vec<ResourceMap>> = vec![
        vec![Level(true).into(), "Level".into(), Level(false).into()],
        vec![
            Human(true).into(),
            "Human".into(),
            Human(false).into(),
            "  |  \n  |  ".into(),
            Bot(true).into(),
            "Bot".into(),
            Bot(false).into(),
        ],
        [Zebra, SameColor, AnyColor]
            .iter()
            .map(|x| ButtonAction::Mode(*x).into())
            .collect(),
        vec![ButtonAction::Apply.into(), ButtonAction::Save.into()],
    ];
    let p = |parent: &mut ChildBuilder| {
        for lr in buttons {
            parent
                .spawn_bundle(menu_lr(&materials))
                .with_children(|parent| {
                    for button in lr {
                        button.spawn_button(parent, &materials)
                    }
                });
        }
    };
    cmd
        .spawn_bundle(root(&materials))
        .insert(MenuUI)
        .insert(Name::new("MenuUI"))
        .with_children(|parent| {
            parent
                .spawn_bundle(border(&materials))
                .with_children(|parent| {
                    parent
                        .spawn_bundle(menu_td(&materials))
                        .with_children(|parent| {
                            p(parent);
                            parent
                                .spawn_bundle(TextBundle {
            text: Text {
                sections:vec![
                    write_strings("",27.0,Color::WHITE,&materials),
                    write_strings("\nNote: Press Apply to start a new Game with above Options,\nelse just Save and exit Menu",19.0,Color::WHITE,&materials),
                ],
                alignment: TextAlignment {
                    horizontal: HorizontalAlign::Center,
                },
            },
            style: Style {
                align_self: AlignSelf::Baseline,
                margin: Rect::all(Val::Px(10.0)),
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            }
                                })
                                .insert(Name::new("Menu Note"))
                                .insert(MenuBoardOptions);
                        });
                });
        });
}
fn apply_options(
    mut query: Query<&mut Text, With<MenuBoardOptions>>,
    board_options: Res<BoardOptions>,
) {
    let mut opt = query.single_mut();
    if board_options.is_changed() {
        opt.sections[0].value = board_options.to_string();
    }
}
#[derive(Component)]
#[component(storage = "SparseSet")]
struct UI;
#[autodefault]
fn setup_ui(mut cmd: Commands, materials: Res<MenuMaterials>, board_options: Res<BoardOptions>) {
    let mode = board_options.mode;
    cmd.spawn_bundle(NodeBundle {
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
    .insert(UI)
    .with_children(|parent| {
        parent.spawn_bundle(TextBundle {
            text: Text {
                sections: vec![
                    write_strings("Instructions:", 27., Color::WHITE, &materials),
                    write_strings(
                        format!("{:?}: {}", mode, mode.desc()),
                        23.,
                        Color::WHITE,
                        &materials,
                    ),
                    write_strings(format!("{}", mode.example()), 17., Color::WHITE, &materials),
                ],
                alignment: TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Left,
                },
            },
        });
    });
    cmd.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Px(100.)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: Rect::all(Val::Px(8.0)),
        },
        color: materials.border,
    })
    .insert(UI)
    .insert(Name::new("UI"))
    .with_children(|p| {
        p.spawn_bundle(menu_td(&materials)).with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    style: Style {
                        align_self: AlignSelf::Baseline,
                        size: Size::new(Val::Auto, Val::Percent(100.0)),
                    },
                    text: Text {
                        sections: vec![
                            write_strings("", 27., Color::WHITE, &materials),
                            write_strings("", 27., Color::WHITE, &materials),
                            write_strings("", 27., Color::WHITE, &materials),
                            write_strings("", 27., Color::WHITE, &materials),
                        ],
                        alignment: TextAlignment {
                            vertical: VerticalAlign::Center,
                            horizontal: HorizontalAlign::Center,
                        },
                    },
                })
                .insert(Name::new("ScoreBoard"))
                .insert(ScoreBoard);
            ButtonAction::Menu.into().spawn_button(parent, &materials);
        });
    });
}
pub fn write_strings<S: Into<String>>(
    text: S,
    font_size: f32,
    color: Color,
    materials: &Res<MenuMaterials>,
) -> TextSection {
    TextSection {
        value: format!("{}\n", text.into()).into(),
        style: TextStyle {
            font: materials.font.clone(),
            font_size,
            color,
        },
    }
}
pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        use super::AppState::*;
        app.init_resource::<MenuMaterials>()
            .add_system_set(SystemSet::on_enter(InGame).with_system(setup_ui))
            .add_system_set(SystemSet::on_resume(InGame).with_system(setup_ui))
            .add_system_set(SystemSet::on_exit(InGame).with_system(despawn::<UI>))
            .add_system_set(
                SystemSet::on_enter(Menu)
                    .with_system(setup_menu)
                    .with_system(despawn::<UI>),
            )
            .add_system_set(SystemSet::on_update(Menu).with_system(apply_options))
            .add_system(asset_button_server::<BoardOptions>)
            .add_system(asset_button_server::<State<AppState>>)
            .add_system_set(SystemSet::on_exit(Menu).with_system(despawn::<MenuUI>));
    }
}
