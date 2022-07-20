use std::iter;

use agent::TagPlayerAgent;
use environment::{TagPlayerVisibleState, TagStatus};
use euclid::default::{Point2D, Rect};
use rand::{Rng, SeedableRng};
use simulation::Simulation;

mod agent;
mod environment;
mod simulation;

fn main() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);
    let area = Rect::from_points(&[(0., 0.).into(), (100., 100.).into()]);

    let players = iter::once(TagPlayerVisibleState {
        position: random_position(&mut rng),
        status: TagStatus::It { tagged_by: 0 },
    })
    .chain(iter::repeat_with(|| TagPlayerVisibleState {
        position: random_position(&mut rng),
        status: TagStatus::NotIt,
    }))
    .map(|state| (TagPlayerAgent, state))
    .take(5);

    let mut simulation = Simulation::new(area, players);

    for _step in 0..100 {
        simulation.step();
    }
    println!("{:#?}", simulation);
}

fn random_position(rng: &mut rand::rngs::StdRng) -> Point2D<f32> {
    Point2D::new(rng.gen_range(0. ..100.), rng.gen_range(0. ..100.))
}
