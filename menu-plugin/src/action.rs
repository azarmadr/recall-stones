use {
    super::materials::MenuMaterials,
    bevy::{ecs::system::Resource, prelude::*},
    std::sync::Arc,
};

#[derive(Component)]
pub struct Action<T> {
    name: String,
    action: Box<dyn Fn(&mut T) + Send + Sync + 'static>,
}
impl<T: Resource> Action<T> {
    pub fn new<U>(name: String, action: U) -> Self
    where
        U: Fn(&mut T) + Send + Sync + 'static,
    {
        Self {
            name,
            action: Box::new(action),
        }
    }
}
#[derive(Component)]
pub struct LabelText<T>(Box<dyn Fn(&T) -> String + Send + Sync + 'static>);
impl<T: Resource> LabelText<T> {
    pub fn new<U>(action: U) -> Self
    where
        U: Fn(&T) -> String + Send + Sync + 'static,
    {
        Self(Box::new(action))
    }
}
pub struct Vol<T> {
    label: Box<dyn Fn(&T) -> String + Send + Sync + 'static>,
    action: Box<dyn Fn(&mut T, bool) + Send + Sync + 'static>,
}
impl<T: Resource> Vol<T> {
    pub fn new<U, V>(label: V, action: U) -> Self
    where
        U: Fn(&mut T, bool) + Send + Sync + 'static,
        V: Fn(&T) -> String + Send + Sync + 'static,
    {
        Self {
            label: Box::new(label),
            action: Box::new(action),
        }
    }
}
pub trait ActionSpawner {
    fn spawn(self, parent: &mut ChildBuilder, materials: &Res<MenuMaterials>);
}
impl<T: Resource> ActionSpawner for Vol<T> {
    fn spawn(self, parent: &mut ChildBuilder, materials: &Res<MenuMaterials>) {
        let f = Arc::new(self);
        let m = f.clone();
        let t = f.clone();
        parent
            .spawn_bundle(materials.button_border())
            .with_children(|p| {
                p.spawn_bundle(materials.menu_td())
                    .with_children(|p| {
                        Action::new("+".to_string(), move |o: &mut T| (f.action)(o, true))
                            .spawn(p, materials);
                        p.spawn_bundle(materials.button_text("".to_string()))
                            .insert(LabelText::new(move |o: &T| (t.label)(o)));
                        Action::new("-".to_string(), move |o: &mut T| (m.action)(o, false))
                            .spawn(p, materials);
                    })
                    .insert(Name::new("Volume Buttons"));
            });
    }
}
impl<T: Resource> ActionSpawner for Action<T> {
    fn spawn(self, parent: &mut ChildBuilder, materials: &Res<MenuMaterials>) {
        let name = &self.name.clone();
        parent
            .spawn_bundle(materials.button_border())
            .insert(Name::new("Action"))
            .with_children(|p| {
                p.spawn_bundle(materials.button())
                    .insert(self)
                    .insert(Name::new(format!("Button({:?})", name)))
                    .with_children(|p| {
                        p.spawn_bundle(materials.button_text(name));
                    });
            });
    }
}
impl ActionSpawner for String {
    fn spawn(self, parent: &mut ChildBuilder, materials: &Res<MenuMaterials>) {
        parent.spawn_bundle(materials.button_text(self));
    }
}

pub fn asset_button_server<T: Resource>(
    button_colors: Res<MenuMaterials>,
    mut asset: ResMut<T>,
    mut interaction_query: Query<(&Interaction, &Action<T>, &mut UiColor), Changed<Interaction>>,
    mut labels: Query<(&LabelText<T>, &mut Text)>,
) {
    for (interaction, action, mut color) in interaction_query.iter_mut() {
        if *interaction == Interaction::Clicked {
            (action.action)(asset.as_mut());
        }
        *color = match *interaction {
            Interaction::Clicked => button_colors.pressed.into(),
            Interaction::Hovered => button_colors.hovered.into(),
            Interaction::None => button_colors.button.into(),
        }
    }
    for (label, mut text) in labels.iter_mut() {
        text.sections[0].value = (label.0)(&asset);
    }
}
