use {
    super::{AppState, MatchRules, MatchRules::*, MemoryGOpts},
    autodefault::autodefault,
    bevy::{ecs::schedule::StateData, prelude::*},
    enum_dispatch::enum_dispatch,
    menu_plugin::*,
    std::cmp::*,
};

#[enum_dispatch]
pub trait ActionSpawner1: Sized + ActionSpawner {}

#[enum_dispatch(ActionSpawner1)]
pub enum ResourceMap {
    StateRes(Action<State<AppState>>),
    Opts(Action<MemoryGOpts>),
    VolButton(Vol<MemoryGOpts>),
    Check(CheckBox<MemoryGOpts>),
    LabelText(String),
}
use ResourceMap::*;
impl From<&str> for ResourceMap {
    fn from(text: &str) -> ResourceMap {
        LabelText(text.to_string())
    }
}
impl ActionSpawner for ResourceMap {
    fn spawn(self, parent: &mut ChildBuilder, materials: &Res<MenuMaterials>) {
        match self {
            StateRes(x) => x.spawn(parent, materials),
            Opts(x) => x.spawn(parent, materials),
            VolButton(x) => x.spawn(parent, materials),
            Check(x) => x.spawn(parent, materials),
            LabelText(x) => x.spawn(parent, materials),
        }
    }
}

fn despawn<T: Component>(mut cmd: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}

/// Button action type
//#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Component)]
pub enum ButtonAction {
    Level,
    Mode(MatchRules),
    Human,
    Bot,
    FullPlate,
    Combo,
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
            if i {
                min(x + 1, ub)
            } else {
                max(lb, x.saturating_sub(1))
            }
        };
        match self {
            Apply => StateRes(Action::new(self.name(), |state: &mut State<AppState>| {
                if *state.current() == AppState::Menu {
                    state.overwrite_replace(AppState::InGame).unwrap();
                }
            })),
            Save => StateRes(Action::new(self.name(), |state: &mut State<AppState>| {
                if !state.inactives().is_empty() && *state.current() == AppState::Menu {
                    state.overwrite_pop().unwrap();
                }
            })),
            Menu => StateRes(Action::new(self.name(), |state: &mut State<AppState>| {
                if *state.current() == AppState::InGame {
                    state.overwrite_push(AppState::Menu).unwrap();
                }
            })),
            Mode(x) => Opts(Action::new(self.name(), move |opts: &mut MemoryGOpts| {
                opts.mode.rule = x
            })),
            Level => VolButton(Vol::new(
                |o: &MemoryGOpts| format!("Level: {}", o.level),
                move |opts: &mut MemoryGOpts, x| opts.level = set(x, opts.level, 0, 5),
            )),
            Human => VolButton(Vol::new(
                |o: &MemoryGOpts| format!("Human: {}", o.players.0),
                move |opts: &mut MemoryGOpts, x| opts.players.0 = set(x, opts.players.0, 1, 2),
            )),
            Bot => VolButton(Vol::new(
                |o: &MemoryGOpts| format!("Bot: {}", o.players.1),
                move |opts: &mut MemoryGOpts, x| opts.players.1 = set(x, opts.players.1, 0, 1),
            )),
            FullPlate => Check(CheckBox::new(
                self.name(),
                |o: &MemoryGOpts| o.mode.full_plate,
                |o: &mut MemoryGOpts| &mut o.mode.full_plate,
            )),
            Combo => Check(CheckBox::new(
                self.name(),
                |o: &MemoryGOpts| o.mode.combo,
                |o: &mut MemoryGOpts| &mut o.mode.combo,
            )),
        }
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
struct MenuUI;
#[derive(Component)]
#[component(storage = "SparseSet")]
struct MenuGOpts;
#[autodefault]
fn setup_menu(mut cmd: Commands, materials: Res<MenuMaterials>) {
    // Make list of buttons
    let buttons: Vec<Vec<ResourceMap>> = vec![
        vec![Level.into(), Human.into(), Bot.into()],
        [CheckeredDeck, TwoDecks, Zebra, SameColor, AnyColor]
            .iter()
            .map(|x| ButtonAction::Mode(*x).into())
            .collect(),
        vec![ButtonAction::FullPlate.into(), ButtonAction::Combo.into()],
        vec![ButtonAction::Apply.into(), ButtonAction::Save.into()],
    ];
    let p = |parent: &mut ChildBuilder| {
        for lr in buttons {
            let mut menu = materials.menu_lr();
            menu.node.style.flex_wrap = FlexWrap::Wrap;
            parent.spawn_bundle(menu).with_children(|parent| {
                for button in lr {
                    button.spawn(parent, &materials)
                }
            });
        }
    };
    let mut menu_border = materials.border();
    menu_border.node.style.max_size.width = Val::Percent(81.);
    cmd
        .spawn_bundle(materials.root())
        .insert(MenuUI)
        .insert(Name::new("MenuUI"))
        .with_children(|parent| {
            parent
                .spawn_bundle(menu_border)
                .with_children(|parent| {
                    parent
                        .spawn_bundle(materials.menu_td())
                        .with_children(|parent| {
                            p(parent);
                            parent
                                .spawn_bundle(TextBundle {
            text: Text {
                sections:vec![
                    materials.write_strings("",1.,Color::WHITE),
                    materials.write_strings("\nNote: Press Apply to start a new Game with above Options,\nelse just Save and exit Menu",0.7,Color::WHITE),
                ],
                alignment: TextAlignment {
                    horizontal: HorizontalAlign::Center,
                },
            },
            style: Style {
                align_self: AlignSelf::Baseline,
                margin: UiRect::all(Val::Px(10.0)),
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            }
                                })
                                .insert(Name::new("Menu Note"))
                                .insert(MenuGOpts);
                        });
                });
        });
}
fn apply_options(mut query: Query<&mut Text, With<MenuGOpts>>, opts: Res<MemoryGOpts>) {
    let mut opt = query.single_mut();
    if opts.is_changed() {
        opt.sections[0].value = format!(
            "Rule: {:?}\nCombo: {}\nFull Plate: {}",
            opts.mode.rule, opts.mode.combo, opts.mode.full_plate
        );
    }
}
#[autodefault]
fn setup_ui(mut cmd: Commands, materials: Res<MenuMaterials>/*, board_options: Res<MemoryGOpts>*/) {
    /* TODO refractor Instructions into another window
    let mode = board_options.mode;
    cmd.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Px(150.)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            position: UiRect { top: Val::Px(0.) },
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
    */
    // Players
    cmd.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Undefined),
            align_items: AlignItems::Center,
            align_self: AlignSelf::FlexEnd,
            justify_content: JustifyContent::FlexStart,
            border: UiRect::all(Val::Px(8.0)),
        },
        color: materials.border.into(),
    })
    .insert(UI)
    .insert(Name::new("UI"))
    .with_children(|p| {
        ButtonAction::Menu.into().spawn(p, &materials);
        /*
        p.spawn_bundle(materials.menu_td()).with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    style: Style {
                        align_self: AlignSelf::Baseline,
                        size: Size::new(Val::Auto, Val::Percent(100.0)),
                    },
                    text: Text {
                        sections: vec![
                            materials.write_strings("", 1., Color::WHITE),
                            materials.write_strings("", 1., Color::WHITE),
                            materials.write_strings("", 1., Color::WHITE),
                            materials.write_strings("", 1., Color::WHITE),
                        ],
                        alignment: TextAlignment {
                            vertical: VerticalAlign::Center,
                            horizontal: HorizontalAlign::Center,
                        },
                    },
                })
                .insert(Name::new("ScoreBoard"))
                .insert(ScoreBoard);
        */
    });
}
#[derive(Component)]
#[component(storage = "SparseSet")]
struct UI;
pub struct MenuPlugin<T> {
    pub game: T,
    pub menu: T,
}
impl<T: Copy + StateData> Plugin for MenuPlugin<T> {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuMaterials>()
            .add_system_set(SystemSet::on_enter(self.game).with_system(setup_ui))
            .add_system_set(SystemSet::on_resume(self.game).with_system(setup_ui))
            .add_system_set(SystemSet::on_exit(self.game).with_system(despawn::<UI>))
            .add_system_set(
                SystemSet::on_enter(self.menu)
                    .with_system(setup_menu)
                    .with_system(despawn::<UI>),
            )
            .add_system_set(SystemSet::on_update(self.menu).with_system(apply_options))
            .add_system(asset_button_server::<MemoryGOpts>)
            .add_system(asset_button_server::<State<T>>)
            .add_system_set(SystemSet::on_exit(self.menu).with_system(despawn::<MenuUI>));
    }
}
