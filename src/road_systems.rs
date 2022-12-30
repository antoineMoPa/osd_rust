use std::str;

use bevy::{
    input::{keyboard::KeyCode, Input},
    prelude::*,
};


use crate::{game::Game, road_network_builder::build_road_network};
use crate::road_network_builder::Segment;

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
        let current_point = Vec3 { x: translation.x, y: translation.y, z: translation.z };

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
