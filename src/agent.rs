use euclid::Vector2D;

use crate::environment::*;

/// How far a player can reach to tag another player
const ARM_LENGTH: PlayerDistance = 5.;
/// How far a player can run each step
const MAX_SPEED: PlayerDistance = 10.;

/// Logic and internal state for the player agent
#[derive(Debug)]
pub struct TagPlayerAgent;

impl TagPlayerAgent {
    /// Decide what action to take on this step based on looking at the environment
    pub fn act(&mut self, player_id: usize, environment: &TagEnvironment) -> TagPlayerAction {
        let TagPlayerVisibleState {
            position,
            status: tagged_by,
        } = environment.get_state(player_id);

        let (closest_player, sq_distance) =
            environment.closest_player_except(player_id, (*tagged_by).into());
        if tagged_by.is_it() {
            if sq_distance < ARM_LENGTH * ARM_LENGTH {
                TagPlayerAction::Tag {
                    player_id: closest_player,
                }
            } else {
                let vector = environment.get_state(closest_player).position - *position;
                TagPlayerAction::Run {
                    stretch: Vector2D::from_angle_and_length(vector.angle_from_x_axis(), MAX_SPEED),
                }
            }
        } else {
            let vector = environment.get_state(closest_player).position - *position;
            TagPlayerAction::Run {
                stretch: Vector2D::from_angle_and_length(vector.angle_from_x_axis(), MAX_SPEED),
            }
        }
    }
}
