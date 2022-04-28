use {bevy::prelude::*,autodefault::autodefault};

#[derive(Bundle)]
pub struct NamedNodeBundle{
#[bundle]
    node:NodeBundle,
    name:Name}
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
impl MenuMaterials {
    pub fn root(&self) -> NodeBundle {
        NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: self.root,
            transform: Transform::from_xyz(0., 0., 1.),
            ..Default::default()
        }
    }
    pub fn button_border(&self) -> NamedNodeBundle {
        NamedNodeBundle{
            node:NodeBundle {
            style: Style {
                //size: Size::new(Val::Percent(100.), Val::Px(50.)),
                border: Rect::all(Val::Px(3.0)),
                flex_basis: Val::Px(0.),
                ..Default::default()
            },
            color: self.button_border,
            ..Default::default()
        },name:Name::new("Button Border")}
    }
    pub fn border(&self) -> NodeBundle {
        NodeBundle {
            style: Style {
                //size: Size::new(Val::Px(400.0), Val::Auto),
                border: Rect::all(Val::Px(3.0)),
                flex_basis: Val::Px(0.),
                ..Default::default()
            },
            color: self.border,
            ..Default::default()
        }
    }
    pub fn button(&self) -> ButtonBundle {
        ButtonBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: Rect::all(Val::Px(3.0)),
                flex_basis: Val::Px(0.),
                ..Default::default()
            },
            color: self.button,
            ..Default::default()
        }
    }
    pub fn menu_background(&self, flex_direction: FlexDirection) -> NodeBundle {
        NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: Rect::all(Val::Px(5.0)),
                flex_direction,
                flex_basis: Val::Px(0.),
                ..Default::default()
            },
            color: self.menu,
            ..Default::default()
        }
    }
    pub fn menu_lr(&self) -> NodeBundle {
        self.menu_background(FlexDirection::RowReverse)
    }
    pub fn menu_td(&self) -> NodeBundle {
        self.menu_background(FlexDirection::ColumnReverse)
    }
    #[autodefault]
    pub fn button_text<S: Into<String>>(&self, label: S) -> TextBundle {
        TextBundle {
            style: Style {
                margin: Rect{right:Val::Px(10.0),left:Val::Px(10.)},
                flex_basis: Val::Px(0.),
            },
            text: Text::with_section(
                label.into(),
                TextStyle {
                    font: self.font.clone(),
                    font_size: 30.0,
                    color: self.button_text.0,
                },
                TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
            ),
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
