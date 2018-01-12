use std::path::PathBuf;
use std::io::Read;

use super::{Reader, Result};

/// Represents an ms3d model file.
#[derive(Clone, Debug)]
pub struct Model {
    pub header: Header,
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<Triangle>,
    pub groups: Vec<Group>,
    pub materials: Vec<Material>,
    pub key_frame_data: KeyFrameData,
    pub joints: Vec<Joint>,
    pub comments: Comments,
    pub vertex_ex_info: VertexExInfo,
    pub joint_ex_info: JointExInfo,
    pub model_ex_info: ModelExInfo,
}

impl Model {
    /// Read an ms3d model file from a reader.
    pub fn from_reader<R: Read>(rdr: R) -> Result<Self> {
        Reader::new(rdr).read_model()
    }
}

#[derive(Clone, Debug)]
pub struct Header {
    pub version: i32,
}

bitflags! {
    pub struct Flags: u8 {
        const SELECTED = 1;
        const HIDDEN = 2;
        const SELECTED2 = 4;
        const DIRTY = 8;
    }
}

#[derive(Clone, Debug)]
pub struct Vertex {
    pub flags: Flags,
    pub vertex: [f32; 3],
    pub bone_id: i8,
    pub reference_count: u8,
}

impl Vertex {
    pub(crate) const ALLOWED_FLAGS: Flags = Flags {
        bits: Flags::SELECTED.bits | Flags::SELECTED2.bits | Flags::HIDDEN.bits,
    };
}

#[derive(Clone, Debug)]
pub struct Triangle {
    pub flags: Flags,
    pub vertex_indices: [u16; 3],
    pub vertex_normals: [[f32; 3]; 3],
    pub s: [f32; 3],
    pub t: [f32; 3],
    pub smoothing_group: u8,
    pub group_index: u8,
}

impl Triangle {
    pub(crate) const ALLOWED_FLAGS: Flags = Flags {
        bits: Flags::SELECTED.bits | Flags::SELECTED2.bits | Flags::HIDDEN.bits,
    };
}

#[derive(Clone, Debug)]
pub struct Group {
    pub flags: Flags,
    pub name: String,
    pub triangle_indices: Vec<u16>,
    pub material_index: i8,
}

impl Group {
    pub(crate) const ALLOWED_FLAGS: Flags = Flags {
        bits: Flags::SELECTED.bits | Flags::HIDDEN.bits,
    };
}

#[derive(Clone, Debug)]
pub struct Material {
    pub name: String,
    pub ambient: [f32; 4],
    pub diffuse: [f32; 4],
    pub specular: [f32; 4],
    pub emissive: [f32; 4],
    pub shininess: f32,
    pub transparency: f32,
    pub mode: u8,
    pub texture: PathBuf,
    pub alphamap: PathBuf,
}

#[derive(Clone, Debug)]
pub struct KeyFrameData {
    pub animation_fps: f32,
    pub current_time: f32,
    pub total_frames: i32,
}

#[derive(Clone, Debug)]
pub struct KeyFrameRot {
    pub time: f32,
    pub rotation: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct KeyFramePos {
    pub time: f32,
    pub position: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct Joint {
    pub flags: Flags,
    pub name: String,
    pub parent_name: String,
    pub rotation: [f32; 3],
    pub position: [f32; 3],
    pub key_frames_rot: Vec<KeyFrameRot>,
    pub key_frames_trans: Vec<KeyFramePos>,
}

impl Joint {
    pub(crate) const ALLOWED_FLAGS: Flags = Flags {
        bits: Flags::SELECTED.bits | Flags::DIRTY.bits,
    };
}

#[derive(Clone, Debug)]
pub struct Comment {
    pub index: i32,
    pub comment: String,
}

#[derive(Clone, Debug)]
pub struct Comments {
    pub sub_version: i32,
    pub group_comments: Vec<Comment>,
    pub material_comments: Vec<Comment>,
    pub joint_comments: Vec<Comment>,
    pub model_comment: Option<Comment>,
}

#[derive(Clone, Debug)]
pub enum VertexExInfo {
    SubVersion1(Vec<VertexEx1>),
    SubVersion2(Vec<VertexEx2>),
    SubVersion3(Vec<VertexEx3>),
}

#[derive(Clone, Debug)]
pub struct VertexEx1 {
    pub bone_ids: [i8; 3],
    pub weights: [u8; 3],
}

#[derive(Clone, Debug)]
pub struct VertexEx2 {
    pub bone_ids: [i8; 3],
    pub weights: [u8; 3],
    pub extra: u32,
}

#[derive(Clone, Debug)]
pub struct VertexEx3 {
    pub bone_ids: [i8; 3],
    pub weights: [u8; 3],
    pub extra: [u32; 2],
}

#[derive(Clone, Debug)]
pub struct JointExInfo {
    pub sub_version: i32,
    pub joint_ex: Vec<JointEx>,
}

#[derive(Clone, Debug)]
pub struct JointEx {
    pub color: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct ModelExInfo {
    pub sub_version: i32,
    pub model_ex: ModelEx,
}

#[derive(Clone, Debug)]
pub struct ModelEx {
    pub joint_size: f32,
    pub transparency_mode: i32,
    pub alpha_ref: f32,
}
