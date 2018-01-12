#![allow(dead_code)] // not sure why this is necessary?

#[repr(C, packed)]
pub(crate) struct Header {
    pub id: [u8; 10],
    pub version: i32,
}

#[repr(C, packed)]
pub(crate) struct Vertex {
    pub flags: u8,
    pub vertex: [f32; 3],
    pub bone_id: i8,
    pub reference_count: u8,
}

#[repr(C, packed)]
pub(crate) struct Triangle {
    pub flags: u16,
    pub vertex_indices: [u16; 3],
    pub vertex_normals: [[f32; 3]; 3],
    pub s: [f32; 3],
    pub t: [f32; 3],
    pub smoothing_group: u8,
    pub group_index: u8,
}

#[repr(C, packed)]
pub(crate) struct GroupPrefix {
    pub flags: u8,
    pub name: [u8; 32],
    pub num_triangles: u16,
}

#[repr(C, packed)]
pub(crate) struct GroupSuffix {
    pub material_index: i8,
}

#[repr(C, packed)]
pub(crate) struct Material {
    pub name: [u8; 32],
    pub ambient: [f32; 4],
    pub diffuse: [f32; 4],
    pub specular: [f32; 4],
    pub emissive: [f32; 4],
    pub shininess: f32,
    pub transparency: f32,
    pub mode: u8,
    pub texture: [u8; 128],
    pub alphamap: [u8; 128],
}

#[repr(C, packed)]
pub(crate) struct KeyFrameData {
    pub animation_fps: f32,
    pub current_time: f32,
    pub total_frames: i32,
}

#[repr(C, packed)]
pub(crate) struct KeyFrameRot {
    pub time: f32,
    pub rotation: [f32; 3],
}

#[repr(C, packed)]
pub(crate) struct KeyFramePos {
    pub time: f32,
    pub position: [f32; 3],
}

#[repr(C, packed)]
pub(crate) struct JointPrefix {
    pub flags: u8,
    pub name: [u8; 32],
    pub parent_name: [u8; 32],
    pub rotation: [f32; 3],
    pub position: [f32; 3],
    pub num_key_frames_rot: u16,
    pub num_key_frames_trans: u16,
}

#[repr(C, packed)]
pub(crate) struct CommentPrefix {
    pub index: i32,
    pub comment_length: i32,
}

#[repr(C, packed)]
pub(crate) struct VertexEx1 {
    pub bone_ids: [i8; 3],
    pub weights: [u8; 3],
}

#[repr(C, packed)]
pub(crate) struct VertexEx2 {
    pub bone_ids: [i8; 3],
    pub weights: [u8; 3],
    pub extra: u32,
}

#[repr(C, packed)]
pub(crate) struct VertexEx3 {
    pub bone_ids: [i8; 3],
    pub weights: [u8; 3],
    pub extra: [u32; 2],
}

#[repr(C, packed)]
pub(crate) struct JointEx {
    pub color: [f32; 3],
}

#[repr(C, packed)]
pub(crate) struct ModelEx {
    pub joint_size: f32,
    pub transparency_mode: i32,
    pub alpha_ref: f32,
}

#[test]
fn test_sizes() {
    use std::mem::size_of;

    assert_eq!(size_of::<Header>(), 14);
    assert_eq!(size_of::<Vertex>(), 15);
    assert_eq!(size_of::<Triangle>(), 70);
    assert_eq!(size_of::<Material>(), 361);
    assert_eq!(size_of::<KeyFrameRot>(), 16);
    assert_eq!(size_of::<KeyFramePos>(), 16);
}
