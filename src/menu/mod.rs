use super::AppState;
use autodefault::autodefault;
use bevy::prelude::*;
use paper_plugin::MatchRules::*;

mod action;
mod materials;
use paper_plugin::{BoardOptions, ScoreBoard};
pub use {action::*, materials::*};
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
        vec![Level.into(),Human.into(),Bot.into()],
        [Zebra, TwoDecks, SameColor, AnyColor]
            .iter()
            .map(|x| ButtonAction::Mode(*x).into())
            .collect(),
        vec![ButtonAction::Apply.into(), ButtonAction::Save.into()],
    ];
    let p = |parent: &mut ChildBuilder| {
        for lr in buttons {
            parent
                .spawn_bundle(materials.menu_lr())
                .with_children(|parent| {
                    for button in lr {
                        button.spawn_button(parent, &materials)
                    }
                });
        }
    };
    cmd
        .spawn_bundle(materials.root())
        .insert(MenuUI)
        .insert(Name::new("MenuUI"))
        .with_children(|parent| {
            parent
                .spawn_bundle(materials.border())
                .with_children(|parent| {
                    parent
                        .spawn_bundle(materials.menu_td())
                        .with_children(|parent| {
                            p(parent);
                            parent
                                .spawn_bundle(TextBundle {
            text: Text {
                sections:vec![
                    materials.write_strings("",27.0,Color::WHITE),
                    materials.write_strings("\nNote: Press Apply to start a new Game with above Options,\nelse just Save and exit Menu",19.0,Color::WHITE),
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
    opts: Res<BoardOptions>,
) {
    let mut opt = query.single_mut();
    if opts.is_changed() {
        opt.sections[0].value = format!( "Rule: {:?}\nCombo: {}\nFull Plate: {}", opts.mode.rule,opts.mode.combo,opts.mode.full_plate);
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
                    materials.write_strings("Instructions:", 27., Color::WHITE),
                    materials.write_strings(
                        format!("{:?}: {}", mode, mode.desc()),
                        23.,
                        Color::WHITE,
                    ),
                    materials.write_strings(format!("{}", mode.example()), 17., Color::WHITE),
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
        p.spawn_bundle(materials.menu_td()).with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    style: Style {
                        align_self: AlignSelf::Baseline,
                        size: Size::new(Val::Auto, Val::Percent(100.0)),
                    },
                    text: Text {
                        sections: vec![
                            materials.write_strings("", 27., Color::WHITE),
                            materials.write_strings("", 27., Color::WHITE),
                            materials.write_strings("", 27., Color::WHITE),
                            materials.write_strings("", 27., Color::WHITE),
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
