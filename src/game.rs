use bevy::prelude::*;

use crate::road_network_builder::RoadNetwork;
use serde::{Serialize, Deserialize};
use bevy_rapier3d::dynamics::*;

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct CameraTarget {
    pub position: Option<Vec3>,
    pub up: Option<Vec3>,
    pub look_at: Option<Vec3>,
}


#[derive(Default)]
pub struct Game {
    pub player_car: Option<Entity>,
    pub trailer: Option<Entity>,
    pub trailer_joint: Option<ImpulseJoint>,
    pub camera_target: CameraTarget,
    pub camera: Option<Entity>,
    pub road_network: RoadNetwork,
    pub road_network_entity: Option<Entity>,
}
