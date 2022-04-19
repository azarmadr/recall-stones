use {
    crate::components::*,
    bevy::prelude::*,
    rand::{distributions::WeightedIndex, prelude::*},
    serde::{Deserialize, Serialize},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Panel {
    pub opened: Vec<u8>,
    pub entity: Entity,
}
impl Panel {
    pub fn init(cmd: &mut Commands, (flesh, bolts): (u8, u8)) -> Vec<Self> {
        let mut weights = [flesh, bolts];
        let mut panels: Vec<Self> = vec![];
        let mut rng = thread_rng();
        let mut idx = 0u8;
        while !weights.iter().all(|&x| x == 0) {
            let dist = WeightedIndex::new(&weights).unwrap();
            let choice = if idx == 0 { 0 } else { dist.sample(&mut rng) };
            weights[choice] -= 1;
            let player = if choice == 1 {
                Player::Bolts(Bolts(idx, 0))
            } else {
                Player::Flesh(Flesh(idx, 0))
            };
            panels.push(Panel {
                opened: vec![],
                entity: cmd
                    .spawn()
                    .insert(Name::new(format!("{:?}", &player)))
                    .insert(player)
                    .id(),
            });
            idx += 1;
        }
        panels
    }
}
