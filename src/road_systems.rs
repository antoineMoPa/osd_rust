use std::str;

use bevy::{
    input::{keyboard::KeyCode, Input},
    prelude::*,
};


use crate::{game::Game, road_network_builder::build_road_network};
use crate::road_network_builder::Segment;
use bevy_rapier3d::prelude::*;

const ROAD_NETWORK_DATA_CHANNEL: &str = "ROAD_NETWORK_DATA";

pub fn refresh_road_network(
    mut game: ResMut<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    // destroy previous meshes if they exist
    match game.road_network_entity {
        Some(entity) => {
            commands.entity(entity).despawn();
        },
        _ => {}
    };

    game.road_network_entity = Some(build_road_network(&game.road_network, commands, meshes, materials));

}

pub fn road_network_creation_system(
    mut transforms: Query<&mut Transform>,
    keyboard_input: Res<Input<KeyCode>>,
    mut game: ResMut<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    if keyboard_input.just_released(KeyCode::E) {
        let entity = match game.player_car {
                Some(entity) => entity,
                _ => {
                    return;
                }
            };

        let transform = match transforms.get_mut(entity) {
            Ok(transform) => transform,
            _ => {
                return;
            }
        };
        let translation = transform.translation;
        let current_point = translation.clone();

        let last_position = match game.road_network.last_position {
            Some(position) => position,
            _ => {
                game.road_network.last_position = Some(current_point.clone());
                return;
            }
        };

        game.road_network.last_position = Some(current_point.clone());

        let segment = Segment { a: last_position.clone(), b: current_point, up: transform.up() };

        game.road_network.road_segments.push(segment);
        refresh_road_network(game, meshes, materials, commands);

        return;
    }

    // Output/Dump road network
    if keyboard_input.just_released(KeyCode::O) {
        let serialized = serde_json::to_string(&game.road_network).unwrap();

        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(&serialized.into());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            println!("serialized = {}", serialized);
        }
    }
}

/// Finds the closest point on a segment to a point.
/// Returns None if the projection lands outside of the segment.
fn find_closest_point_on_segment_capped(segment_a: Vec3, segment_b: Vec3, p: Vec3) -> Option<Vec3> {
    let v12 = segment_b - segment_a;
    let v13 = p - segment_a;
    let dot_product = v12.dot(v13);
    let length_squared = v12.length_squared();

    if dot_product < 0.0 {
        return None;
    } else if dot_product > length_squared {
        return None;
    } else {
        return Some(segment_a + v12 * (dot_product / length_squared));
    }
}

pub fn road_physics_system(
    mut transforms: Query<&mut Transform>,
    mut game: ResMut<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    mut ext_forces: Query<&mut ExternalForce>,
) {
    let vehicle_entity = match game.player_car {
        Some(entity) => entity,
        _ => {
            return;
        }
    };

    let mut ext_force = match ext_forces.get_mut(vehicle_entity) {
        Ok(ext_force) => ext_force,
        _ => {
            return;
        }
    };

    let vehicle_transform = match transforms.get_mut(vehicle_entity) {
        Ok(vehicle_transform) => vehicle_transform,
        _ => {
            return;
        }
        };
    let vehicle_position = vehicle_transform.translation;

    let mut closest_segment_index: Option<usize> = None;
    let mut closest_point: Option<Vec3> = None;
    let mut closest_dist: Option<f32> = None;

    // find close segments to vehicle (dumb, not efficient)
    for (index, segment_data) in game.road_network.road_segments.iter().enumerate() {
        let i: u32 = index as u32;
        let p1: Vec3 = segment_data.a;
        let p2: Vec3 = segment_data.b;
        let closest_point_to_segment: Option<Vec3> = find_closest_point_on_segment_capped(p1, p2, vehicle_transform.translation);


        match closest_point_to_segment {
            Some(closest_point_to_segment) => {
                let dist: f32 = (closest_point_to_segment - vehicle_position).length();

                match closest_dist {
                    Some(closest_dist) => {
                        if closest_segment_index.is_none() || closest_point.is_none() {
                            panic!("If dist is set, then we also expect closest_point and closest_segment_index.");
                        }

                        if dist < closest_dist {
                            // We become the new closest dist
                        } else {
                           // Continue, so that we don't set the closest dist.
                            continue;
                        }
                    },
                    _ => {}
                }

                // 0: This first point becomes the new closest dist
                // 1..n: This point is the closest now. Update!
                closest_dist = Some(dist);
                closest_point = Some(closest_point_to_segment);
                closest_segment_index = Some(index);
            },
            _ => {}
        }

    }

    match closest_point {
        Some(closest_point) => {
            let force_direction = closest_point - vehicle_position;
            ext_force.force += force_direction * 20.0;
        },
        _ => {}
    }
}
