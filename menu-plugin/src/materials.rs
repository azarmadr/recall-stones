use {autodefault::autodefault, bevy::prelude::*};

#[derive(Default, Bundle)]
pub struct NamedBundle<T: Bundle> {
    #[bundle]
    node: T,
    name: Name,
}
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
pub struct MenuMaterials {
    pub none: Color,
    pub root: Color,
    pub menu: Color,
    pub border: Color,
    pub button: Color,
    pub hovered: Color,
    pub pressed: Color,
    pub font: Handle<Font>,
    pub button_border: Color,
    pub button_text: Color,
}
impl FromWorld for MenuMaterials {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        MenuMaterials {
            none: Color::NONE,
            root: Color::rgba(0., 0., 0., 0.27),
            menu: Color::rgb(0.15, 0.15, 0.15),
            border: Color::rgb(0.65, 0.65, 0.65),
            button_border: Color::rgb(0.81, 0.65, 0.65),
            button: Color::rgb(0.1, 0.15, 0.1),
            hovered: Color::rgb(0.25, 0.25, 0.25),
            pressed: Color::rgb(0.35, 0.75, 0.35),
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            button_text: Color::rgb(0.9, 0.9, 0.9),
        }
    }
}
impl MenuMaterials {
    pub fn root(&self) -> NamedBundle<NodeBundle> {
        NamedBundle {
            node: NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    flex_direction: FlexDirection::ColumnReverse,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: self.root.into(),
                transform: Transform::from_xyz(0., 0., 1.),
                ..default()
            },
            name: Name::new("Root"),
        }
    }
    pub fn button_border(&self) -> NamedBundle<NodeBundle> {
        NamedBundle {
            node: NodeBundle {
                style: Style {
                    //size: Size::new(Val::Percent(100.), Val::Px(50.)),
                    border: Rect::all(Val::Px(3.0)),
                    flex_basis: Val::Px(0.),
                    ..default()
                },
                color: self.button_border.into(),
                ..default()
            },
            name: Name::new("Button Border"),
        }
    }
    pub fn border(&self) -> NamedBundle<NodeBundle> {
        NamedBundle {
            node: NodeBundle {
                style: Style {
                    //size: Size::new(Val::Px(400.0), Val::Auto),
                    border: Rect::all(Val::Px(3.0)),
                    flex_basis: Val::Px(0.),
                    ..default()
                },
                color: self.border.into(),
                ..default()
            },
            name: Name::new("Border"),
        }
    }
    pub fn button(&self) -> NamedBundle<ButtonBundle> {
        NamedBundle {
            node: ButtonBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: Rect::all(Val::Px(3.0)),
                    flex_basis: Val::Px(0.),
                    ..default()
                },
                color: self.button.into(),
                ..default()
            },
            name: Name::new("Button"),
        }
    }
    pub fn menu_background(&self, flex_direction: FlexDirection) -> NamedBundle<NodeBundle> {
        NamedBundle {
            node: NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: Rect::all(Val::Px(5.0)),
                    flex_direction,
                    flex_basis: Val::Px(0.),
                    ..default()
                },
                color: self.menu.into(),
                ..default()
            },
            name: Name::new("Menu Background"),
        }
    }
    pub fn menu_lr(&self) -> NamedBundle<NodeBundle> {
        NamedBundle{name:Name::new("Menu Left-Right"),..self.menu_background(FlexDirection::RowReverse)}
    }
    pub fn menu_td(&self) -> NamedBundle<NodeBundle> {
        NamedBundle{name:Name::new("Menu Top-Down"),..self.menu_background(FlexDirection::ColumnReverse)}
    }
    #[autodefault]
    pub fn button_text<S: Into<String>>(&self, label: S) -> NamedBundle<TextBundle> {
        NamedBundle {
            node: TextBundle {
                style: Style {
                    margin: Rect {
                        right: Val::Px(10.0),
                        left: Val::Px(10.),
                    },
                    flex_basis: Val::Px(0.),
                },
                text: Text::with_section(
                    label.into(),
                    TextStyle {
                        font: self.font.clone(),
                        font_size: 25.0,
                        color: self.button_text,
                    },
                    TextAlignment {
                        vertical: VerticalAlign::Center,
                        horizontal: HorizontalAlign::Center,
                    },
                ),
            },
            name: Name::new("Button Text"),
        }
    }
    pub fn write_strings<S: Into<String>>(
        &self,
        text: S,
        font_size: f32,
        color: Color,
    ) -> TextSection {
        TextSection {
            value: format!("{}\n", text.into()).into(),
            style: TextStyle {
                font: self.font.clone(),
                font_size,
                color,
            },
        }
    }
}
