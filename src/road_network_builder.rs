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
pub struct Macro {
    pub road_segments: Vec<Segment>,
}


#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct RoadNetwork {
    pub last_position: Option<Vec3>,
    pub road_segments: Vec<Segment>,
    pub macros: Vec<Macro>,
}

/// Compute a triangle's normal
fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let (a, b, c) = (Vec3::from(a), Vec3::from(b), Vec3::from(c));
    (b - a).cross(c - a).normalize().into()
}

pub fn build_road_network(
    road_network: &RoadNetwork,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) -> Entity {
    let mut position_attributes: Vec<[f32; 3]> = Vec::new();
    let mut normal_attributes: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let len: usize = road_network.road_segments.len();

    for (index, segment_data) in road_network.road_segments.iter().enumerate() {
        let i: u32 = index as u32;
        let a: Vec3 = segment_data.a;
        let b: Vec3 = segment_data.b;


        let segment: Vec3 = b - a;
        let up: Vec3 = segment_data.up;
        let right: Vec3 = segment.cross(up).normalize();
        let left: Vec3 = -right;
        let mut next_right = right;
        let mut next_left = left;

        if index < len - 1 {
            let next_segment_data = &road_network.road_segments[index + 1];
            let next_segment: Vec3 = next_segment_data.b - next_segment_data.a;
            next_right = next_segment.cross(next_segment_data.up).normalize();
            next_left = -next_right;
        }

        //
        //       next right --------->
        //    <---------- next_left
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

        let p1: Vec3 = a + left * 1.5;
        let p2: Vec3 = a + right * 1.5;
        let p3: Vec3 = b + next_left * 1.5;
        let p4: Vec3 = b + next_right * 1.5;

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

    let material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.9, 0.5, 0.3),
        double_sided: true,
        cull_mode: None,
        ..Default::default()
    });

    return commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        material: material,
        ..default()
    }).id();
}
