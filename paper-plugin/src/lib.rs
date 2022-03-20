//use crate::resources::card::Card;
use crate::deck::Deck;
use bevy::ecs::schedule::StateData;
use bevy::log;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::render::view::Visibility;
use bevy::text::Text2dSize;
use bevy::utils::{AHashExt, HashMap};

use crate::components::*;
use crate::events::{CardFlipEvent, DeckCompletedEvent};
pub use bounds::*;
pub use resources::*;

mod bounds;
pub mod components;
pub mod events;
mod resources;
mod systems;

pub struct PaperPlugin<T> {
    pub running_state: T,
}

impl<T: StateData> Plugin for PaperPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(self.running_state.clone()).with_system(Self::create_board),
        )
        // We handle input and trigger events only if the state is active
        .add_system_set(
            SystemSet::on_update(self.running_state.clone())
                .with_system(systems::input::input_handling)
                .with_system(systems::uncover::render_revealed)
                .with_system(systems::uncover::trigger_event_handler),
        )
        .add_system_set(
            SystemSet::on_in_stack_update(self.running_state.clone())
                .with_system(systems::uncover::flip_cards),
        )
        .add_system_set(
            SystemSet::on_exit(self.running_state.clone()).with_system(Self::cleanup_board),
        )
        .add_event::<CardFlipEvent>()
        .add_event::<DeckCompletedEvent>();
    }
}

impl<T> PaperPlugin<T> {
    /// System to generate the complete board
    pub fn create_board(
        mut commands: Commands,
        board_options: Option<Res<BoardOptions>>,
        window: Res<WindowDescriptor>,
        asset_server: Res<AssetServer>,
    ) {
        log::info!("padding: {:?}", board_options);
        let options = match board_options {
            None => BoardOptions::default(), // If no options is set we use the default one
            Some(o) => {
                commands.remove_resource::<BoardOptions>(); // After this system the options are no longer relevant
                o.clone()
            }
        };

        // Cardmap generation
        let mut deck = Deck::empty(options.deck_size.0);
        deck.set_cards(options.max_limit);
        #[cfg(feature = "debug")]
        // Cardmap debugging
        log::info!("{}", deck.console_output());

        // Setup

        // We define the size of our cards in world space
        let card_size = match options.card_size {
            CardSize::Fixed(v) => v,
            CardSize::Adaptive { min, max } => {
                Self::adaptative_card_size(window, (min, max), (deck.width(), deck.width() + 1))
            }
        };
        log::info!("card_size: {}", card_size);
        log::info!("padding: {:?}", options);
        // We deduce the size of the complete board
        let board_size = Vec2::new(
            deck.width() as f32 * card_size,
            (2. * deck.count() as f32/ deck.width() as f32).ceil() * card_size,
        );
        log::info!(
            "width: {}, count: {}, board size: {}",
            deck.width(),
            deck.count(),
            board_size
        );
        // We define the board anchor position (bottom left)
        let board_position = match options.position {
            BoardPosition::Centered { offset } => {
                Vec3::new(-(board_size.x / 2.), -(board_size.y / 2.), 0.) + offset
            }
            BoardPosition::Custom(p) => p,
        };

        // TODO: refactor this
        let font = asset_server.load("fonts/minecraft.ttf");
        let bomb_image = asset_server.load("sprites/bomb.png");
        //

        let mut hidden_cards = HashMap::with_capacity((deck.width() * (1 + deck.width())).into());
        let board_entity = commands
            .spawn()
            .insert(Name::new("Board"))
            .insert(Transform::from_translation(board_position))
            // This component is required until https://github.com/bevyengine/bevy/pull/2331 is merged
            .insert(GlobalTransform::default())
            .with_children(|parent| {
                // We spawn the board background sprite at the center of the board, since the sprite pivot is centered
                parent
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: Color::WHITE,
                            custom_size: Some(board_size),
                            ..Default::default()
                        },
                        transform: Transform::from_xyz(board_size.x / 2., board_size.y / 2., 0.),
                        ..Default::default()
                    })
                    .insert(Name::new("Background"));
                parent
                    .spawn_bundle(Text2dBundle {
                        text: Text::with_section(
                            format!(
                                "Luck: {}\nPerfect Memory: {}",
                                deck.count(),
                                deck.count() * 2 - 1
                            ),
                            TextStyle {
                                font: font.clone(),
                                color: Color::WHITE,
                                font_size: 30.0,
                            },
                            Default::default(),
                        ),
                        ..Default::default() //style:
                    })
                    .insert(Name::new("Score"))
                    .insert(Score);
                Self::spawn_cards(
                    parent,
                    &deck,
                    card_size,
                    options.card_padding,
                    Color::GRAY,
                    Color::GRAY,
                    bomb_image,
                    font,
                    &mut hidden_cards,
                );
            })
            .id();
        // We add the main resource of the game, the board
        commands.insert_resource(Board {
            deck,
            bounds: Bounds2 {
                position: board_position.xy(),
                size: board_size,
            },
            score: 0,
            completed: false,
            card_size,
            hidden_cards,
            entity: board_entity,
        })
    }

    fn spawn_cards(
        parent: &mut ChildBuilder,
        deck: &Deck,
        size: f32,
        padding: f32,
        _color: Color,
        _hidden_card_color: Color,
        _bomb_image: Handle<Image>,
        font: Handle<Font>,
        hidden_cards: &mut HashMap<Idx, Entity>,
    ) {
        // Cards
        for (i, card) in deck.iter().enumerate() {
            let (x, y) = (i % deck.width() as usize, i / deck.width() as usize);
            let coordinates = Idx(i as u16);
            let mut cmd = parent.spawn();
            cmd.insert_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::GRAY,
                    custom_size: Some(Vec2::splat(size - padding)),
                    ..Default::default()
                },
                transform: Transform::from_xyz(
                    (x as f32 * size) + (size / 2.),
                    (y as f32 * size) + (size / 2.),
                    1.,
                ),
                ..Default::default()
            })
            .insert(Name::new(format!("Card ({}, {})", x, y)));
            cmd.with_children(|parent| {
                let entity = parent
                    .spawn_bundle(Self::card_to_text_bundle(
                        card.val(),
                        font.clone(),
                        size - padding,
                    ))
                    .insert(Name::new("Card"))
                    .insert(coordinates)
                    .id();
                hidden_cards.insert(coordinates, entity);
            });
        }
    }

    /// Generates the card value text 2D Bundle
    fn card_to_text_bundle(value: u16, font: Handle<Font>, size: f32) -> Text2dBundle {
        // We retrieve the text and the correct color
        let (text, color) = (
            value.to_string(),
            match value {
                0..=9 => Color::WHITE,
                10..=18 => Color::GREEN,
                19..=27 => Color::YELLOW,
                28..=36 => Color::ORANGE,
                _ => Color::PURPLE,
            },
        );
        // We generate a text bundle
        Text2dBundle {
            text: Text {
                sections: vec![TextSection {
                    value: text,
                    style: TextStyle {
                        color,
                        font,
                        font_size: size,
                    },
                }],
                alignment: TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
            },
            text_2d_size: Text2dSize {
                size: Size {
                    width: size,
                    height: size,
                },
            },
            transform: Transform::from_xyz(0., 0., 1.),
            visibility: Visibility { is_visible: false },
            ..Default::default()
        }
    }

    /// Computes a card size that matches the window according to the card map size
    fn adaptative_card_size(
        window: Res<WindowDescriptor>,
        (min, max): (f32, f32),
        (width, height): (u16, u16),
    ) -> f32 {
        let max_width = window.width / width as f32;
        let max_heigth = window.height / height as f32;
        max_width.min(max_heigth).clamp(min, max)
    }

    fn cleanup_board(board: Res<Board>, mut commands: Commands) {
        commands.entity(board.entity).despawn_recursive();
        commands.remove_resource::<Board>();
    }
}
