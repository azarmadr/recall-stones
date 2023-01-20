use crate::{Deck, GameState, MemoryGAssts, MemoryGOpts, RuleSet};
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
    Bots,
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
    SetBots(u8),
}

impl Actions {
    fn handle_events(
        mut action_event_reader: EventReader<Self>,
        mut app_event: EventWriter<AppExit>,
        mut commands: Commands,
        mut state: ResMut<State<GameState>>,
        menu_state: Option<Res<MenuState<Screens>>>,
    ) {
        if let Some(menu_state) = menu_state {
            if !action_event_reader.is_empty() {
                commands.insert_resource(menu_state.state().0.clone());
            }
        }
        for event in action_event_reader.iter() {
            match event {
                Self::NewGame => {
                    state.overwrite_replace(GameState::Game).unwrap();
                }
                Self::Resume => state.pop().unwrap(),
                Self::Pause => state.push(GameState::Menu).unwrap(),
                Self::Quit => app_event.send(AppExit),
                _ => (),
            }
        }
    }
}

impl ActionTrait for Actions {
    type State = (MemoryGOpts, Handle<Image>);
    type Event = Self;

    fn handle(&self, state: &mut Self::State, event_writer: &mut EventWriter<Self::Event>) {
        let (state, ..) = state;
        match self {
            Self::SetHumanFirst => state.human_first ^= true,
            Self::SetDuel => state.mode.duel ^= true,
            Self::SetCombo => state.mode.combo ^= true,
            Self::SetFullPlate => state.mode.full_plate ^= true,
            Self::SetAutoStart => state.auto_start ^= true,
            Self::SetRule(rs) => state.mode.rule = *rs,
            Self::SetLevel(l) => state.level = *l,
            Self::SetBots(count) => {
                state.players.1 = *count;
                state.human_first |= *count == 0
            }
            _ => event_writer.send(*self),
        }
    }
}

impl ScreenTrait for Screens {
    type Action = Actions;
    type State = (MemoryGOpts, Handle<Image>);

    fn resolve(
        &self,
        state: &<<Self as ScreenTrait>::Action as bevy_quickmenu::ActionTrait>::State,
    ) -> bevy_quickmenu::Menu<Self> {
        let (state, img) = state;
        let bots_action =
            |l| MenuItem::action(format!("{l}"), Actions::SetBots(l)).checked(state.players.1 == l);
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
            Self::NewGame => [
                MenuItem::headline("Recall Stones"),
                MenuItem::action("Start!!", Actions::NewGame)
                    .with_icon(MenuIcon::Other(img.clone())),
                MenuItem::label(""),
                MenuItem::label("Settings"),
                MenuItem::action("Auto Start", Actions::SetAutoStart).checked(state.auto_start),
                MenuItem::screen("Levels", Screens::Levels),
                MenuItem::screen("Bots", Screens::Bots),
                MenuItem::screen("Rule Set", Screens::RuleSet).with_icon(MenuIcon::Controls),
                MenuItem::action("Full Plate", Actions::SetFullPlate)
                    .checked(state.mode.full_plate),
            ]
            .into_iter()
            .chain(
                [
                    MenuItem::action("Player First", Actions::SetHumanFirst)
                        .checked(state.human_first),
                    MenuItem::action("Duel", Actions::SetDuel).checked(state.mode.duel),
                    MenuItem::action("Combo", Actions::SetCombo).checked(state.mode.combo),
                ]
                .into_iter()
                .take(if state.players.1 > 0 { 3 } else { 0 }),
            )
            .collect(),
            Self::Bots => [MenuItem::headline("Bots")]
                .into_iter()
                .chain((0..2).map(|x| bots_action(x)))
                .collect(),
            Self::Levels => [MenuItem::headline("Levels")]
                .into_iter()
                .chain((0..6).map(|x| level_action(x)))
                .collect(),
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
    assets: Res<MemoryGAssts>,
    mut prev_state: Local<Option<GameState>>,
) {
    if prev_state.map_or(false, |x| x == *state.current()) {
        return;
    }
    *prev_state = Some(*state.current());

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

    commands.insert_resource(MenuState::new(
        (cfg, assets.icon.clone()),
        screen,
        Some(sheet),
    ));
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
