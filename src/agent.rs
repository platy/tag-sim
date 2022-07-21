use euclid::Angle;

use crate::environment::*;

type RunStretch = euclid::default::Vector2D<f32>;
/// How far a player can reach to tag another player
const ARM_LENGTH: PlayerDistance = 1.;
/// How far a player can run each step
const MAX_SPEED: PlayerDistance = 2.;

/// Logic and internal state for the player agent
#[derive(Debug)]
pub struct TagPlayerAgent;

impl TagPlayerAgent {
    /// Decide what action to take on this step based on looking at the environment
    pub fn act(
        &mut self,
        player_id: usize,
        environment: &TagEnvironment,
    ) -> Result<TagPlayerAction> {
        let TagPlayerVisibleState {
            position,
            status: tagged_by,
        } = environment.get_state(player_id);

        let action = if tagged_by.is_it() {
            let (closest_player, sq_distance) =
                environment.closest_player_except(player_id, (*tagged_by).into())?;
            if sq_distance < ARM_LENGTH * ARM_LENGTH {
                TagPlayerAction::Tag {
                    player_id: closest_player,
                }
            } else {
                let vector = environment.get_state(closest_player).position - *position;
                TagPlayerAction::Run {
                    stretch: RunStretch::from_angle_and_length(
                        vector.angle_from_x_axis(),
                        MAX_SPEED,
                    ),
                }
            }
        } else {
            let it = environment.get_it();
            let vector = it.position - *position;
            let mut angle = vector.angle_from_x_axis();
            if !angle.is_finite() {
                angle = Angle::radians(0.);
            }
            let stretch = -RunStretch::from_angle_and_length(angle, MAX_SPEED);
            let stretch = turn_at_edges(&environment.area(), *position, stretch);
            TagPlayerAction::Run { stretch }
        };
        Ok(action)
    }
}

/// keeps the player running at full speed by turning them along the edge of the play area
fn turn_at_edges(area: &PlayArea, from: Position, stretch: RunStretch) -> RunStretch {
    let target = from + stretch;
    let x_in_bounds = area.x_range().contains(&target.x);
    let y_in_bounds = area.y_range().contains(&target.y);

    match (x_in_bounds, y_in_bounds) {
        // not effected by edges
        (true, true) => stretch,
        // headed into a corner
        (false, false) => {
            if stretch.x > stretch.y {
                redirect_out_of_x_bounds(from, area, stretch)
            } else {
                redirect_out_of_y_bounds(from, area, stretch)
            }
        }
        // headed off the top or bottom
        (true, false) => redirect_out_of_y_bounds(from, area, stretch),
        // here, to avoid duplication, the same function is used with the axes swapped
        (false, true) => redirect_out_of_x_bounds(from, area, stretch),
    }
}

fn redirect_out_of_x_bounds(
    from: euclid::Point2D<f32, euclid::UnknownUnit>,
    area: &euclid::Rect<f32, euclid::UnknownUnit>,
    stretch: euclid::Vector2D<f32, euclid::UnknownUnit>,
) -> euclid::Vector2D<f32, euclid::UnknownUnit> {
    redirect_out_of_y_bounds(
        from.yx(),
        &PlayArea::from_points(&[area.min().yx(), area.max().yx()]),
        stretch.yx(),
    )
    .yx()
}

fn redirect_out_of_y_bounds(
    from: euclid::Point2D<f32, euclid::UnknownUnit>,
    area: &euclid::Rect<f32, euclid::UnknownUnit>,
    stretch: euclid::Vector2D<f32, euclid::UnknownUnit>,
) -> euclid::Vector2D<f32, euclid::UnknownUnit> {
    // y broken
    let margin = 0.1;
    let broken_y_bound = if (from + stretch).y > area.max_y() {
        area.max_y() - margin
    } else {
        area.min_y() + margin
    };
    let redirectable_x_offset = ((from.y - broken_y_bound).powi(2) - stretch.length().powi(2))
        .abs()
        .sqrt();
    let redirected_x = from.x + stretch.x.signum() * redirectable_x_offset;
    let new_target = if redirected_x > area.max_x() {
        let limited_x = area.max_x() - margin;
        let redirectable_y_offset = ((from.x - limited_x).powi(2) - stretch.length().powi(2))
            .abs()
            .sqrt();
        let redirected_y = from.y - redirectable_y_offset;
        (limited_x, redirected_y)
    } else if redirected_x < area.min_x() {
        let limited_x = area.min_x() + margin;
        let redirectable_y_offset = ((from.x - limited_x).powi(2) - stretch.length().powi(2))
            .abs()
            .sqrt();
        let redirected_y = from.y + redirectable_y_offset;
        (limited_x, redirected_y)
    } else {
        (redirected_x, broken_y_bound)
    };
    Position::from(new_target) - from
}

#[cfg(test)]
macro_rules! assert_valid_stretch {
    ($start:expr, $stretch:expr, *, *, $area:expr) => {
        assert!(
            $area.contains($start + $stretch),
            "stretch {}: Area {:?} didn't contain {:?} + {:?} = {:?}",
            stringify!($stretch),
            $area,
            $start,
            $stretch,
            $start + $stretch
        );
    };
    ($start:expr, $stretch:expr, *, $length:expr, $area:expr) => {
        assert!(euclid::approxeq::ApproxEq::approx_eq(
            &$stretch.length(),
            &$length
        ));
        assert!(
            $area.contains($start + $stretch),
            "stretch {}: Area {:?} didn't contain {:?} + {:?} = {:?}",
            stringify!($stretch),
            $area,
            $start,
            $stretch,
            $start + $stretch
        );
    };
    ($start:expr, $stretch:expr, $degrees:expr, $length:expr, $area:expr) => {
        assert!(
            euclid::approxeq::ApproxEq::approx_eq(
                &$stretch.angle_from_x_axis(),
                &Angle::degrees($degrees)
            ),
            "stretch {}: angle {} != {:?}",
            stringify!($stretch),
            $degrees,
            $stretch.angle_from_x_axis().to_degrees()
        );
        assert!(euclid::approxeq::ApproxEq::approx_eq(
            &$stretch.length(),
            &$length
        ));
        assert!(
            $area.contains($start + $stretch),
            "stretch {}: Area {:?} didn't contain {:?} + {:?} = {:?}",
            stringify!($stretch),
            $area,
            $start,
            $stretch,
            $start + $stretch
        );
    };
}

#[test]
fn test_avoid_corners() {
    let area = PlayArea::from_points(&[(0., 0.).into(), (10., 10.).into()]);
    let length = 3.;
    let close_to_right_top: Position = (9.0, 8.0).into();

    let past_top = turn_at_edges(
        &area,
        close_to_right_top,
        RunStretch::from_angle_and_length(Angle::degrees(91.), length),
    );
    assert_valid_stretch!(close_to_right_top, past_top, *, length, area);
    let ok_1 = turn_at_edges(
        &area,
        close_to_right_top,
        RunStretch::from_angle_and_length(Angle::degrees(-91.), length),
    );
    assert_valid_stretch!(close_to_right_top, ok_1, -91., length, area);
    let past_right = turn_at_edges(
        &area,
        close_to_right_top,
        RunStretch::from_angle_and_length(Angle::degrees(1.), length),
    );
    assert_valid_stretch!(close_to_right_top, past_right, *, length, area);
    let ok_2 = turn_at_edges(
        &area,
        close_to_right_top,
        RunStretch::from_angle_and_length(Angle::degrees(-179.), length),
    );
    assert_valid_stretch!(close_to_right_top, ok_2, -179., length, area);
    let past_top_and_right = turn_at_edges(
        &area,
        close_to_right_top,
        RunStretch::from_angle_and_length(Angle::degrees(45.), length),
    );
    assert_valid_stretch!(close_to_right_top, past_top_and_right, *, *, area);

    let close_to_left_bottom: Position = (3.0, 1.0).into();

    let ok_3 = turn_at_edges(
        &area,
        close_to_left_bottom,
        RunStretch::from_angle_and_length(Angle::degrees(90.), length),
    );
    assert_valid_stretch!(close_to_left_bottom, ok_3, 90., length, area);
    let past_bottom = turn_at_edges(
        &area,
        close_to_left_bottom,
        RunStretch::from_angle_and_length(Angle::degrees(-90.), length),
    );
    assert_valid_stretch!(close_to_left_bottom, past_bottom, *, length, area);
    let ok_4 = turn_at_edges(
        &area,
        close_to_left_bottom,
        RunStretch::from_angle_and_length(Angle::degrees(0.), length),
    );
    assert_valid_stretch!(close_to_left_bottom, ok_4, 0., length, area);
    let past_left = turn_at_edges(
        &area,
        close_to_left_bottom,
        RunStretch::from_angle_and_length(Angle::degrees(180.), length),
    );
    assert_valid_stretch!(close_to_left_bottom, past_left, *, length, area);
    let past_bottom_and_left = turn_at_edges(
        &area,
        close_to_left_bottom,
        RunStretch::from_angle_and_length(Angle::degrees(180. + 45.), length),
    );
    assert_valid_stretch!(close_to_left_bottom, past_bottom_and_left, *, *, area);
}
