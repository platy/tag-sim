use crate::{
    agent::TagPlayerAgent,
    environment::{PlayerDistance, TagEnvironment, TagPlayerVisibleState},
};

/// Simulation runner
#[derive(Debug)]
pub struct Simulation {
    agents: Vec<TagPlayerAgent>,
    environment: TagEnvironment,
    step: u64,
}

impl Simulation {
    /// Create a new simulation specifying the playing area and an iterator to generate all the players
    pub fn new(
        area: euclid::default::Rect<PlayerDistance>,
        players: impl IntoIterator<Item = (TagPlayerAgent, TagPlayerVisibleState)>,
    ) -> Self {
        let (agents, player_state): (Vec<_>, Vec<_>) = players.into_iter().unzip();
        Self {
            agents,
            environment: TagEnvironment::new(area, player_state),
            step: 0,
        }
    }

    /// Step the simulation:
    ///
    /// 1. Ask each agent to choose it's action based on the current environment
    /// 2. Apply the actions to the environment
    /// 3. Increment step counter
    pub fn step(&mut self) {
        let actions = self
            .agents
            .iter_mut()
            .enumerate()
            .map(|(player_id, agent)| agent.act(player_id, &self.environment))
            .collect();

        self.environment.apply_actions(actions);
        self.step += 1;
    }
}
