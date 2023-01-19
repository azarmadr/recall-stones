use crate::{Deck, GameState, MemoryGOpts, RuleSet};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_quickmenu::{style::Stylesheet, MenuItem, *};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Screens {
    Game,
    Pause,
    NewGame,
    GameOver,
    RuleSet,
    Levels,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Actions {
    Resume,
    Pause,
    Quit,
    NewGame,
    SetHumanFirst,
    SetDuel,
    SetCombo,
    SetFullPlate,
    SetAutoStart,
    SetRule(RuleSet),
    SetLevel(u8),
    _SetBots(u8),
}

impl Actions {
    fn handle_events(
        mut action_event_reader: EventReader<Self>,
        mut app_exit_event: EventWriter<AppExit>,
        mut commands: Commands,
        mut state: ResMut<State<GameState>>,
        menu_state: Option<Res<MenuState<Screens>>>,
    ) {
        if let Some(menu_state) = menu_state {
            if !action_event_reader.is_empty() {
                commands.insert_resource(menu_state.state().clone());
            }
        }
        for event in action_event_reader.iter() {
            match event {
                Self::NewGame => {
                    state.overwrite_replace(GameState::Game).unwrap();
                }
                Self::Resume => state.pop().unwrap(),
                Self::Pause => state.push(GameState::Menu).unwrap(),
                Self::Quit => app_exit_event.send(AppExit),
                _ => (),
            }
        }
    }
}

impl ActionTrait for Actions {
    type State = MemoryGOpts;
    type Event = Self;

    fn handle(&self, state: &mut Self::State, event_writer: &mut EventWriter<Self::Event>) {
        match self {
            Self::SetHumanFirst => state.human_first ^= true,
            Self::SetDuel => state.mode.duel ^= true,
            Self::SetCombo => state.mode.combo ^= true,
            Self::SetFullPlate => state.mode.full_plate ^= true,
            Self::SetAutoStart => state.auto_start ^= true,
            Self::SetRule(rs) => state.mode.rule = *rs,
            Self::SetLevel(l) => state.level = *l,
            Self::_SetBots(count) => state.players.1 = *count,
            _ => event_writer.send(*self),
        }
    }
}

impl ScreenTrait for Screens {
    type Action = Actions;
    type State = MemoryGOpts;

    fn resolve(
        &self,
        state: &<<Self as ScreenTrait>::Action as bevy_quickmenu::ActionTrait>::State,
    ) -> bevy_quickmenu::Menu<Self> {
        let level_action =
            |l| MenuItem::action(format!("{l}"), Actions::SetLevel(l)).checked(state.level == l);
        let rule_action = |rs| {
            MenuItem::action(format!("{rs:?}"), Actions::SetRule(rs)).checked(state.mode.rule == rs)
        };
        let mut menu_items = match self {
            Self::Game => vec![MenuItem::action("Pause", Actions::Pause)],
            Self::Pause => vec![
                MenuItem::headline("Paused"),
                MenuItem::action("Resume", Actions::Resume),
                MenuItem::screen("New Game", Screens::NewGame),
                MenuItem::action("Quit", Actions::Quit),
            ],
            Self::NewGame => vec![
                MenuItem::headline("Recall Stones"),
                MenuItem::action("New Game", Actions::NewGame),
                MenuItem::label(""),
                MenuItem::label("Configuration"),
                MenuItem::action("Auto Start", Actions::SetAutoStart)
                    .checked(state.auto_start),
                MenuItem::screen("Levels", Screens::Levels),
                MenuItem::screen("Rule Set", Screens::RuleSet),
                MenuItem::action("Player First", Actions::SetHumanFirst).checked(state.human_first),
                MenuItem::action("Duel", Actions::SetDuel).checked(state.mode.duel),
                MenuItem::action("Combo", Actions::SetCombo).checked(state.mode.combo),
                MenuItem::action("Full Plate", Actions::SetFullPlate)
                    .checked(state.mode.full_plate),
            ],
            Self::Levels => vec![
                MenuItem::headline("Levels"),
                level_action(0),
                level_action(1),
                level_action(2),
                level_action(3),
                level_action(4),
                level_action(5),
            ],
            Self::RuleSet => vec![
                MenuItem::headline("Rule Sets"),
                rule_action(RuleSet::AnyColor),
                rule_action(RuleSet::SameColor),
                rule_action(RuleSet::Zebra),
                rule_action(RuleSet::TwoDecks),
                rule_action(RuleSet::CheckeredDeck),
            ],
            Self::GameOver => vec![
                MenuItem::headline(state.outcome()),
                MenuItem::screen("New Game", Screens::NewGame),
                MenuItem::action("Quit", Actions::Quit),
            ],
        };
        menu_items.reverse();
        Menu::new(format!("{self:?}"), menu_items)
    }
}

fn menu(
    mut commands: Commands,
    state: Res<State<GameState>>,
    opts: Option<Res<MemoryGOpts>>,
    deck: Option<Res<Deck>>,
    mut prev: Local<String>,
) {
    let current_state = format!("{:?}{:?}",state.current(),state.inactives());
    if *prev == current_state {return;}
    *prev = current_state;

    let cfg = opts.map_or(MemoryGOpts::default(), |x| x.clone());
    let screen = if state.current() == &GameState::Game {
        Screens::Game
    } else if cfg.outcome.is_some() {
        Screens::GameOver
    } else if deck.is_none() {
        Screens::NewGame
    } else {
        Screens::Pause
    };
    let sheet = Stylesheet::default()
        .with_background(BackgroundColor(Color::BLACK))
        .with_style(Style {
            align_self: AlignSelf::FlexEnd,
            align_items: AlignItems::FlexEnd,
            ..default()
        });
    commands.insert_resource(MenuState::new(cfg, screen, Some(sheet)));
}

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(QuickMenuPlugin::<Screens>::new())
            .add_event::<Actions>()
            .add_system(Actions::handle_events)
            .add_system(menu);
    }
}
