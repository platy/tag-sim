use crate::{agent::TagPlayerAgent, environment::*};

/// Simulation runner
#[derive(Debug)]
pub struct Simulation {
    actions: Vec<TagPlayerAction>,
    agents: Vec<TagPlayerAgent>,
    environment: TagEnvironment,
    step: u64,
}

impl Simulation {
    /// Create a new simulation specifying the playing area and an iterator to generate all the players
    pub fn new(
        area: PlayArea,
        players: impl IntoIterator<Item = (TagPlayerAgent, TagPlayerVisibleState)>,
    ) -> Self {
        let (agents, player_state): (Vec<_>, Vec<_>) = players.into_iter().unzip();
        Self {
            actions: Vec::with_capacity(agents.len()),
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
    pub fn step(&mut self) -> Result<()> {
        self.actions.clear();

        for action in self
            .agents
            .iter_mut()
            .enumerate()
            .map(|(player_id, agent)| agent.act(player_id, &self.environment))
        {
            self.actions.push(action?);
        }

        self.environment.apply_actions(&self.actions);
        self.step += 1;
        Ok(())
    }

    pub fn actions(&self) -> &[TagPlayerAction] {
        &self.actions
    }

    pub fn player_state(&self) -> &[TagPlayerVisibleState] {
        self.environment.player_state()
    }

    pub fn environment(&self) -> &TagEnvironment {
        &self.environment
    }
}
