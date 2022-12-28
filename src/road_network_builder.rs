use bevy::{
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

/// Compute a triangle's normal
fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let (a, b, c) = (Vec3::from(a), Vec3::from(b), Vec3::from(c));
    (b - a).cross(c - a).normalize().into()
}

use wasm_bindgen::{prelude::*};

pub fn build_road_network(
    road_network: &RoadNetwork,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut position_attributes: Vec<[f32; 3]> = Vec::new();
    let mut normal_attributes: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    web_sys::console::log_1(&JsValue::from(String::from("Will spawn one segment")));

    for (index, segment) in road_network.road_segments.iter().enumerate() {
        let i: u32 = index as u32;
        let a: Vec3 = segment.a;
        let b: Vec3 = segment.b;


        web_sys::console::log_1(&JsValue::from(String::from("Will spawn one segment")));

        let p1: [f32; 3] = [a.x + 1.0, a.y + 1.0, a.z + 1.0];
        let p2: [f32; 3] = [a.x + 0.0, a.y + 2.0, a.z + 1.0];
        let p3: [f32; 3] = [a.x + 1.0, a.y + 2.0, a.z + 0.0];
        position_attributes.push(p1);
        position_attributes.push(p2);
        position_attributes.push(p3);

        normal_attributes.push(face_normal(p1, p2, p3));
        normal_attributes.push(face_normal(p1, p2, p3));
        normal_attributes.push(face_normal(p1, p2, p3));

        indices.push(0 + i * 3);
        indices.push(2 + i * 3);
        indices.push(1 + i * 3);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, position_attributes);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normal_attributes);
    mesh.set_indices(Some(Indices::U32(indices)));

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::rgb(0.9, 0.5, 0.3).into()),
        ..default()
    });

}
