use std::fmt;

use crate::{
    environment::{PlayArea, Position, TagPlayerAction},
    simulation::Simulation,
};

/// Render the current state of the simulation and the actions to the canvas
pub fn render_frame<const WIDTH: usize, const HEIGHT: usize>(
    simulation: &Simulation,
    actions: &[TagPlayerAction],
    canvas: &mut TagCanvas<WIDTH, HEIGHT>,
) {
    for (player, action) in simulation.player_state().iter().zip(actions) {
        canvas.set(
            player.position,
            if matches!(action, TagPlayerAction::Tag { .. }) {
                DrawCell::YoureIt
            } else if player.is_it() {
                DrawCell::It
            } else {
                DrawCell::Runner
            },
        );
    }
}

/// Ascii art canvas for a tag game
pub struct TagCanvas<const WIDTH: usize, const HEIGHT: usize> {
    area: PlayArea,
    grid: [[DrawCell; WIDTH]; HEIGHT],
}

impl<const WIDTH: usize, const HEIGHT: usize> TagCanvas<WIDTH, HEIGHT> {
    /// New canvas for drawing a particular playing field
    pub fn new(area: PlayArea) -> Self {
        Self {
            area,
            grid: [[DrawCell::None; WIDTH]; HEIGHT],
        }
    }

    /// Set what should be rendered in a cell. Only overwrites if the cell is more important than the existing cell
    pub fn set(&mut self, position: Position, cell: DrawCell) {
        let x = (position.x / self.area.width() * (WIDTH - 1) as f32) as usize;
        let y = (position.y / self.area.height() * (HEIGHT - 1) as f32) as usize;
        let existing_cell = &mut self.grid[y][x];
        if cell > *existing_cell {
            *existing_cell = cell;
        }
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> fmt::Display for TagCanvas<WIDTH, HEIGHT> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "==================================")?;
        for row in self.grid {
            let mut x = 0;
            while x < row.len() {
                let chars = match row[x] {
                    DrawCell::None => " ",
                    DrawCell::YoureIt => "*-You're It!",
                    DrawCell::It => "*",
                    DrawCell::Runner => ".",
                };
                write!(f, "{}", chars)?;
                x += chars.len();
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// What should be drawn in a cell
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DrawCell {
    None = 0,
    /// a player here isn't it
    Runner = 1,
    /// a player here is it
    It = 2,
    /// a player here was it and just tagged another player
    YoureIt = 3,
}
