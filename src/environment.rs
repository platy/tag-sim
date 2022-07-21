use std::error::Error;

use euclid::default::Vector2D;

pub type PlayArea = euclid::default::Rect<PlayerDistance>;
pub type Position = euclid::default::Point2D<f32>;
pub type PlayerDistance = f32;
pub type PlayerId = usize;
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// The state about each player which is visible through the environment to the other players
#[derive(Debug)]
pub struct TagPlayerVisibleState {
    /// The player's current position
    pub position: Position,
    /// Whether the player is it
    pub status: TagStatus,
}

/// Whether a player is it, and if they are who tagged them
#[derive(Copy, Clone, Debug)]
pub enum TagStatus {
    /// The player is not it
    NotIt,
    /// The player is it and was tagged by the indicated player
    It { tagged_by: PlayerId },
}

impl TagStatus {
    pub fn is_it(&self) -> bool {
        matches!(self, Self::It { .. })
    }
}

impl From<TagStatus> for Option<PlayerId> {
    fn from(status: TagStatus) -> Self {
        match status {
            TagStatus::NotIt => None,
            TagStatus::It { tagged_by } => Some(tagged_by),
        }
    }
}

impl TagPlayerVisibleState {
    pub fn is_it(&self) -> bool {
        self.status.is_it()
    }
}

/// Information about the state of the simulation that the player agents have access to
#[derive(Debug)]
pub struct TagEnvironment {
    /// The game should be limited to this area
    area: PlayArea,
    /// Visible state about all the players
    player_state: Vec<TagPlayerVisibleState>,
}

impl TagEnvironment {
    pub fn new(area: PlayArea, player_state: Vec<TagPlayerVisibleState>) -> Self {
        Self { area, player_state }
    }

    /// Get state of one of the players
    pub fn get_state(&self, player_id: PlayerId) -> &TagPlayerVisibleState {
        &self.player_state[player_id]
    }

    /// Get the player closest to a specified player, optionally ignoring a player
    pub fn closest_player_except(
        &self,
        player_id: PlayerId,
        ignore: Option<PlayerId>,
    ) -> Result<(PlayerId, PlayerDistance)> {
        let mut closest_player = None;
        let my_position = self.get_state(player_id).position;

        for (
            i,
            TagPlayerVisibleState {
                position,
                status: _,
            },
        ) in self.player_state.iter().enumerate()
        {
            if i == player_id || Some(i) == ignore {
                continue;
            }
            let square_distance = (my_position - *position).square_length();
            if let Some((_, shortest_distance)) = closest_player {
                if square_distance < shortest_distance {
                    closest_player = Some((i, square_distance));
                }
            } else {
                closest_player = Some((i, square_distance))
            }
        }
        closest_player.ok_or_else(|| "Closest player with less than 2 players".into())
    }

    /// Apply an action for each player to mutate the environment
    pub fn apply_actions(&mut self, actions: &[TagPlayerAction]) {
        assert!(
            self.player_state.len() == actions.len(),
            "Must apply one action for each player known to the environment"
        );
        for (idx, action) in actions.iter().enumerate() {
            self.apply_action(idx, action)
        }
    }

    fn apply_action(&mut self, player_id: PlayerId, action: &TagPlayerAction) {
        match action {
            TagPlayerAction::Run { stretch } => {
                assert!(stretch.is_finite());
                let point2_d = &mut self.player_state[player_id].position;
                *point2_d += *stretch;
                if point2_d.x < self.area.min_x() {
                    point2_d.x = self.area.min_x();
                }
                if point2_d.x > self.area.max_x() {
                    point2_d.x = self.area.max_x();
                }
                if point2_d.y < self.area.min_y() {
                    point2_d.y = self.area.min_y();
                }
                if point2_d.y > self.area.max_y() {
                    point2_d.y = self.area.max_y();
                }
            }
            TagPlayerAction::Tag {
                player_id: other_player_id,
            } => {
                println!("{}: TAG {}", player_id, other_player_id);
                assert!(
                    self.player_state[player_id].is_it(),
                    "Player ({}) can't tag if they're not it",
                    player_id
                );
                self.player_state[player_id].status = TagStatus::NotIt;
                self.player_state[*other_player_id].status = TagStatus::It {
                    tagged_by: player_id,
                };
            }
        }
    }

    pub fn player_state(&self) -> &[TagPlayerVisibleState] {
        &self.player_state
    }

    pub fn area(&self) -> PlayArea {
        self.area
    }
}

/// Action each player agent can choose to take after each step
pub enum TagPlayerAction {
    /// Player can run a stretch
    Run { stretch: Vector2D<PlayerDistance> },
    /// Player can tag a player near to them
    Tag { player_id: PlayerId },
}

#[cfg(test)]
mod test {
    use euclid::default::Rect;

    use super::*;

    #[test]
    fn apply_run() {
        let mut e = TagEnvironment {
            area: Rect::from_points(&[(0., 0.).into(), (100., 100.).into()]),
            player_state: vec![
                TagPlayerVisibleState {
                    position: (0., 0.).into(),
                    status: TagStatus::NotIt,
                },
                TagPlayerVisibleState {
                    position: (1., 1.).into(),
                    status: TagStatus::It { tagged_by: 1 },
                },
            ],
        };
        assert_eq!(e.get_state(0).position, (0., 0.).into());
        assert_eq!(e.get_state(1).position, (1., 1.).into());
        e.apply_action(
            1,
            &TagPlayerAction::Run {
                stretch: (10., 10.).into(),
            },
        );
        assert_eq!(e.get_state(0).position, (0., 0.).into());
        assert_eq!(e.get_state(1).position, (11., 11.).into());
        e.apply_action(
            0,
            &TagPlayerAction::Run {
                stretch: (20., 20.).into(),
            },
        );
        assert_eq!(e.get_state(0).position, (20., 20.).into());
        assert_eq!(e.get_state(1).position, (11., 11.).into());
    }

    #[test]
    fn apply_run_out_of_area() {
        let mut e = TagEnvironment {
            area: Rect::from_points(&[(0., 0.).into(), (100., 100.).into()]),
            player_state: vec![TagPlayerVisibleState {
                position: (95., 0.).into(),
                status: TagStatus::NotIt,
            }],
        };
        assert_eq!(e.get_state(0).position, (95., 0.).into());
        e.apply_action(
            0,
            &TagPlayerAction::Run {
                stretch: (10., 10.).into(),
            },
        );
        assert_eq!(e.get_state(0).position, (100., 10.).into());
    }

    #[test]
    fn apply_tag() {
        let mut e = TagEnvironment {
            area: Rect::from_points(&[(0., 0.).into(), (100., 100.).into()]),
            player_state: vec![
                TagPlayerVisibleState {
                    position: (0., 0.).into(),
                    status: TagStatus::NotIt,
                },
                TagPlayerVisibleState {
                    position: (1., 1.).into(),
                    status: TagStatus::It { tagged_by: 1 },
                },
            ],
        };
        assert!(!e.get_state(0).is_it());
        assert!(e.get_state(1).is_it());
        e.apply_action(1, &TagPlayerAction::Tag { player_id: 0 });
        assert!(e.get_state(0).is_it());
        assert!(!e.get_state(1).is_it());
    }

    #[test]
    fn test_closest_player() -> Result<()> {
        let e = TagEnvironment {
            area: Rect::from_points(&[(0., 0.).into(), (100., 100.).into()]),
            player_state: vec![
                TagPlayerVisibleState {
                    position: (0., 0.).into(),
                    status: TagStatus::NotIt,
                },
                TagPlayerVisibleState {
                    position: (5., 0.).into(),
                    status: TagStatus::NotIt,
                },
                TagPlayerVisibleState {
                    position: (10., 10.).into(),
                    status: TagStatus::It { tagged_by: 2 },
                },
            ],
        };
        assert_eq!(e.closest_player_except(0, None)?.0, 1);
        assert_eq!(e.closest_player_except(1, None)?.0, 0);
        assert_eq!(e.closest_player_except(2, None)?.0, 1);
        assert_eq!(e.closest_player_except(0, Some(1))?.0, 2);
        assert_eq!(e.closest_player_except(1, Some(0))?.0, 2);
        assert_eq!(e.closest_player_except(2, Some(1))?.0, 0);

        Ok(())
    }
}
