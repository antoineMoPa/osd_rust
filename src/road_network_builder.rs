use bevy::{
    input::{keyboard::KeyCode, Input},
    pbr::DirectionalLightShadowMap,
    prelude::*, render::{render_resource::PrimitiveTopology, mesh::Indices},
};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Segment {
    pub a: Vec3,
    pub b: Vec3
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct RoadNetwork {
    pub last_position: Option<Vec3>,
    pub road_segments: Vec<Segment>
}


pub fn build_road_network(
    road_network: RoadNetwork,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[1.0, 1.0, 1.0], [0.0, 2.0, 1.0], [1.0, 2.0, 1.0]]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[1.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 1.0]]);
    mesh.set_indices(Some(Indices::U32(vec![0,2,1])));

    // add entities to the world
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::rgb(0.9, 0.5, 0.3).into()),
        ..default()
    });

}
