use bevy::{
    prelude::*, render::{render_resource::PrimitiveTopology, mesh::Indices},
};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Segment {
    pub a: Vec3,
    pub b: Vec3,
    /// Normal/Up vector of the segment. (usually, the up vector of a car driving on the road)
    pub up: Vec3,
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
) -> Entity {
    let mut position_attributes: Vec<[f32; 3]> = Vec::new();
    let mut normal_attributes: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for (index, segment_data) in road_network.road_segments.iter().enumerate() {
        let i: u32 = index as u32;
        let a: Vec3 = segment_data.a;
        let b: Vec3 = segment_data.b;
        let segment: Vec3 = b - a;
        let up: Vec3 = segment_data.up;
        let right: Vec3 = segment.cross(up).normalize();
        let left: Vec3 = -right.normalize();

        //
        // p3                         p4
        //  +--------------------------+               -> b
        //  |            |             |
        //  |            |             |
        //  |            |             |
        //  +--------------------------+               ->  a
        //  p1         0,0,0           p2
        //
        //      <-------- left
        //         right    --------->
        //

        let p1: Vec3 = a + left;
        let p2: Vec3 = a + right;
        let p3: Vec3 = b + left;
        let p4: Vec3 = b + right;

        position_attributes.push(p1.into());
        position_attributes.push(p2.into());
        position_attributes.push(p3.into());
        position_attributes.push(p4.into());

        let n1:  [f32; 3] = face_normal(p1.into(), p2.into(), p3.into());
        normal_attributes.push(n1);
        normal_attributes.push(n1);
        normal_attributes.push(n1);
        normal_attributes.push(n1);

        indices.push(0 + i * 4);
        indices.push(1 + i * 4);
        indices.push(2 + i * 4);

        indices.push(2 + i * 4);
        indices.push(1 + i * 4);
        indices.push(3 + i * 4);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, position_attributes);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normal_attributes);
    mesh.set_indices(Some(Indices::U32(indices)));

    return commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::rgb(0.9, 0.5, 0.3).into()),
        ..default()
    }).id();
}
