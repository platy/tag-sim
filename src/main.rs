use std::{env, iter, thread, time::Duration};

use agent::TagPlayerAgent;
use environment::{PlayArea, TagPlayerVisibleState, TagStatus};
use euclid::default::{Point2D, Rect};
use rand::{Rng, SeedableRng};
use simulation::Simulation;

use crate::viewer::{render_frame, TagCanvas};

mod agent;
mod environment;
mod simulation;
mod viewer;

fn main() {
    let player_count: usize = env::args()
        .nth(1)
        .map(|s| {
            s.parse()
                .expect("parameters are [player_count [step_limit]]")
        })
        .unwrap_or(5);
    let step_limit: usize = env::args()
        .nth(2)
        .map(|s| {
            s.parse()
                .expect("parameters are [player_count [step_limit]]")
        })
        .unwrap_or(100);

    let mut rng = rand::rngs::StdRng::seed_from_u64(0);
    let area = Rect::from_points(&[(0., 0.).into(), (100., 100.).into()]);

    let players = iter::once(TagPlayerVisibleState {
        position: random_position(&mut rng, &area),
        status: TagStatus::It { tagged_by: 0 },
    })
    .chain(iter::repeat_with(|| TagPlayerVisibleState {
        position: random_position(&mut rng, &area),
        status: TagStatus::NotIt,
    }))
    .map(|state| (TagPlayerAgent, state))
    .take(player_count);

    let mut simulation = Simulation::new(area, players);

    let mut canvas;
    for _step in 0..step_limit {
        let actions = simulation.step().expect("Simulation failed");
        canvas = TagCanvas::<25, 25>::new(simulation.environment().area());
        render_frame(&simulation, actions, &mut canvas);
        println!("{}", canvas);
        thread::sleep(Duration::from_millis(20));
    }
}

/// Select a random position within the play area
fn random_position(rng: &mut rand::rngs::StdRng, area: &PlayArea) -> Point2D<f32> {
    Point2D::new(rng.gen_range(area.x_range()), rng.gen_range(area.y_range()))
}
