//! Field of View calculation
//!
//! Uses symmetric shadowcasting for realistic FOV.

use super::Map;
use crate::ecs::Position;

/// Compute field of view from a position with given radius
pub fn compute_fov(map: &mut Map, origin: Position, radius: i32) -> Vec<Position> {
    let mut visible = Vec::new();

    // Clear previous visibility
    map.clear_visibility();

    // Origin is always visible
    map.set_visible(origin.x, origin.y, true);
    visible.push(origin);

    // Cast shadows in all 8 octants
    for octant in 0..8 {
        cast_light(map, &mut visible, origin, radius, 1, 1.0, 0.0, octant);
    }

    visible
}

/// Recursive shadowcasting for a single octant
fn cast_light(
    map: &mut Map,
    visible: &mut Vec<Position>,
    origin: Position,
    radius: i32,
    row: i32,
    mut start_slope: f64,
    end_slope: f64,
    octant: u8,
) {
    if start_slope < end_slope {
        return;
    }

    let mut next_start_slope = start_slope;

    for j in row..=radius {
        let mut blocked = false;

        let dy = -j;
        for dx in dy..=0 {
            let (map_x, map_y) = transform_octant(dx, dy, octant);
            let cur_x = origin.x + map_x;
            let cur_y = origin.y + map_y;

            let left_slope = (dx as f64 - 0.5) / (dy as f64 + 0.5);
            let right_slope = (dx as f64 + 0.5) / (dy as f64 - 0.5);

            if start_slope < right_slope {
                continue;
            }
            if end_slope > left_slope {
                break;
            }

            // Check if within circular radius
            let distance_squared = dx * dx + dy * dy;
            if distance_squared <= radius * radius {
                if map.in_bounds(cur_x, cur_y) {
                    map.set_visible(cur_x, cur_y, true);
                    visible.push(Position::new(cur_x, cur_y));
                }
            }

            if blocked {
                if map.is_opaque(cur_x, cur_y) {
                    next_start_slope = right_slope;
                } else {
                    blocked = false;
                    start_slope = next_start_slope;
                }
            } else if map.is_opaque(cur_x, cur_y) && j < radius {
                blocked = true;
                cast_light(
                    map,
                    visible,
                    origin,
                    radius,
                    j + 1,
                    start_slope,
                    left_slope,
                    octant,
                );
                next_start_slope = right_slope;
            }
        }

        if blocked {
            break;
        }
    }
}

/// Transform coordinates based on octant
fn transform_octant(col: i32, row: i32, octant: u8) -> (i32, i32) {
    match octant {
        0 => (col, row),
        1 => (row, col),
        2 => (row, -col),
        3 => (col, -row),
        4 => (-col, -row),
        5 => (-row, -col),
        6 => (-row, col),
        7 => (-col, row),
        _ => (col, row),
    }
}
