use bevy::{
    input::{keyboard::KeyCode, Input},
    prelude::*,
};


use crate::{game::Game, road_network_builder::*};
use crate::road_network_builder::Segment;
use bevy_rapier3d::prelude::*;


// There are probably conceptual errors in there, but it works.
// This is a mechanism similar to a PID.
// P: adjustement of correction based on current position difference
// R: adjustement of correction based on rate of change
// D: adjustement of correction based on current variation of error
fn apply_control(
    delta: Vec3,
    rate_of_change: Vec3,// -> velocity.angvel
    p: f32,
    r: f32, // (old i)
) -> Vec3 {
    let correction: Vec3 = delta * p + rate_of_change * r;
    return correction;
}

pub fn refresh_road_network(
    mut game: ResMut<Game>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
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
    mut ext_forces: Query<&mut ExternalForce>,
    keyboard_input: Res<Input<KeyCode>>,
    mut game: ResMut<Game>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    commands: Commands,
) {
    let vehicle_entity = match game.player_car {
        Some(vehicle_entity) => vehicle_entity,
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

    let mut vehicle_transform = match transforms.get_mut(vehicle_entity) {
        Ok(vehicle_transform) => vehicle_transform,
        _ => {
            return;
        }
    };

    // E: Insert road segment. (E because it is close to WASD)
    if keyboard_input.just_released(KeyCode::E) {
        let translation = vehicle_transform.translation;
        let current_point = translation.clone();

        let last_position = match game.road_network.last_position {
            Some(position) => position,
            _ => {
                game.road_network.last_position = Some(current_point.clone());
                return;
            }
        };

        game.road_network.last_position = Some(current_point.clone());

        let segment = Segment { a: last_position.clone(), b: current_point, up: vehicle_transform.up() };

        game.road_network.road_segments.push(segment);
        refresh_road_network(game, meshes, materials, commands);

        return;
    }

    // O: Output/Dump road network
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

    // X: Delete everything and go back to 0,0
    if keyboard_input.just_released(KeyCode::X) {
        game.road_network.road_segments.clear();
        game.road_network.last_position = Some(Vec3::ZERO);

        vehicle_transform.translation = Vec3::ZERO;
        ext_force.force = Vec3::ZERO;
        ext_force.torque = Vec3::ZERO;
        refresh_road_network(game, meshes, materials, commands);
    }

    // R: Record current state as macro
    else if keyboard_input.just_released(KeyCode::R) {
        let mut m: Macro =  Macro::default();
        m.road_segments = game.road_network.road_segments.clone();
        game.road_network.macros.push(m);
    }

    // P: Play macro
    else if keyboard_input.just_released(KeyCode::P) {
        if game.road_network.macros.len() <= 0 {
            return;
        }
        let segments = game.road_network.macros.last().unwrap().road_segments.clone();
        for segment in &segments {
            let t: Vec3 = vehicle_transform.translation;
            let r: Quat = vehicle_transform.rotation;
            let a = r * segment.a + t;
            let b = r * segment.b + t;
            let up = r * segment.up;
            game.road_network.road_segments.push(Segment {
                a: a,
                b: b,
                up: up
            });
        }

        // move vehicle to last point
        let last_segment = game.road_network.road_segments.last().unwrap();
        vehicle_transform.translation = last_segment.b;
        let t = vehicle_transform.translation;
        vehicle_transform.look_at(t + (last_segment.b - last_segment.a).normalize(), last_segment.up);

        refresh_road_network(game, meshes, materials, commands);
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
    game: ResMut<Game>,
    mut ext_forces: Query<&mut ExternalForce>,
    mut velocities: Query<&mut Velocity>,

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

    struct ClosestPointInfo {
        segment: Vec3,
        up: Vec3,
    }

    let mut closest_segment: Option<ClosestPointInfo> = None;
    let mut closest_segment_index: Option<usize> = None;
    let mut closest_point: Option<Vec3> = None;
    let mut closest_dist: Option<f32> = None;

    // In this game, vehicles float above roads
    let offset: Vec3 = Vec3::Y * 1.5;

    // find close segments to vehicle (dumb, not efficient)
    for (index, segment_data) in game.road_network.road_segments.iter().enumerate() {
        let p1: Vec3 = segment_data.a + offset;
        let p2: Vec3 = segment_data.b + offset;
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
                closest_segment = Some(ClosestPointInfo {
                    segment: segment_data.b - segment_data.a,
                    up: segment_data.up
                });
            },
            _ => {}
        }
    }

    let velocity = match velocities.get_mut(vehicle_entity) {
        Ok(velocity) => velocity,
        _ => return
    };

    match closest_point {
        Some(closest_point) => {

            // Too far: outside of road force field.
            if closest_dist.unwrap() > 10.0 {
                return;
            }

            if let Some(closest_segment) = closest_segment {
                const P: f32 = 40.0;
                const R: f32 = -3.0;
                const SUB_TARGET_FRACTION: f32 = 0.8;

                // Make vehicle more aligned with road
                let delta_forward = -closest_segment.segment
                    .normalize()
                    .cross(vehicle_transform.forward())
                    * SUB_TARGET_FRACTION;

                let delta_up = -closest_segment.up
                    .normalize()
                    .cross(vehicle_transform.up())
                    * SUB_TARGET_FRACTION;

                ext_force.torque += apply_control(
                    delta_forward,
                    velocity.angvel,
                    P,
                    R,
                );

                ext_force.torque += apply_control(
                    delta_up,
                    velocity.angvel,
                    P,
                    R,
                );

                {
                    const P: f32 = 4.0;
                    const R: f32 = -2.0;

                    let delta_position: Vec3 = closest_point - vehicle_position;

                    let centering_force = apply_control(
                        delta_position,
                        velocity.linvel,
                        P,
                        R,
                    );

                    // This force will only act on the plane perpendicular to the segment.
                    ext_force.force += centering_force
                        - centering_force.project_onto(closest_segment.segment);
                }
            };
        },
        _ => {}
    }
}
