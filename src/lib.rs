//! This crate provides utilities working with ms3d models. The main entry
//! point for this crate is the [`Model::from_reader`](struct.Model.html#method.from_reader)
//! function which parses a model file.

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate failure;
extern crate memchr;

mod de;
mod model;
mod read;

pub use model::*;
pub use failure::Error;

use read::{BufReadExact, IoReader, SliceReader};

use memchr::memchr;

use std::io;
use std::{mem, ptr, str, u8};
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

struct Reader<R: BufReadExact> {
    rdr: R,
}

impl<R: io::Read> Reader<IoReader<R>> {
    fn from_io_reader(rdr: R) -> Self {
        Reader { rdr: IoReader::new(rdr) }
    }
}

impl<'a> Reader<SliceReader<'a>> {
    fn from_slice(slice: &'a [u8]) -> Self {
        Reader { rdr: SliceReader::new(slice) }
    }
}

impl<R: BufReadExact> Reader<R> {
    fn read_model(&mut self) -> Result<Model> {
        let header = self.read_header()?;
        let vertices = self.read_vertices()?;
        let triangles = self.read_triangles()?;
        let groups = self.read_groups()?;
        let materials = self.read_materials()?;
        let key_frame_data = self.read_key_frame_data()?;
        let joints = self.read_joints()?;
        let comments = self.read_comments()?;
        let vertex_ex_info = self.read_vertex_ex_info(vertices.len())?;
        let joint_ex_info = self.read_joint_ex_info(joints.len())?;
        let model_ex_info = self.read_model_ex_info()?;

        Ok(Model {
            header,
            vertices,
            triangles,
            groups,
            materials,
            key_frame_data,
            joints,
            comments,
            vertex_ex_info,
            joint_ex_info,
            model_ex_info,
        })
    }

    fn read_header(&mut self) -> Result<Header> {
        let de::Header { id, version } = unsafe { self.read_type()? };
        ensure!(id == "MS3D000000".as_bytes(), "invalid header");
        ensure!(version == 4, "unsupported version {}", version);
        Ok(Header { version })
    }

    fn read_vertices(&mut self) -> Result<Vec<Vertex>> {
        let len = self.read_u16()? as usize;
        self.read_vec(len, Self::read_vertex)
    }

    fn read_vertex(&mut self) -> Result<Vertex> {
        let de::Vertex {
            flags,
            vertex,
            bone_id,
            reference_count,
        } = unsafe { self.read_type()? };
        let flags = convert_flags(flags, Vertex::ALLOWED_FLAGS)?;
        Ok(Vertex {
            flags,
            vertex,
            bone_id,
            reference_count,
        })
    }

    fn read_triangles(&mut self) -> Result<Vec<Triangle>> {
        let len = self.read_u16()? as usize;
        self.read_vec(len, Self::read_triangle)
    }

    fn read_triangle(&mut self) -> Result<Triangle> {
        let de::Triangle {
            flags,
            vertex_indices,
            vertex_normals,
            s,
            t,
            smoothing_group,
            group_index,
        } = unsafe { self.read_type()? };
        let flags = convert_flags(flags as u8, Triangle::ALLOWED_FLAGS)?;
        Ok(Triangle {
            flags,
            vertex_indices,
            vertex_normals,
            s,
            t,
            smoothing_group,
            group_index,
        })
    }

    fn read_groups(&mut self) -> Result<Vec<Group>> {
        let len = self.read_u16()? as usize;
        self.read_vec(len, Self::read_group)
    }

    fn read_group(&mut self) -> Result<Group> {
        let de::GroupPrefix {
            flags,
            name,
            num_triangles,
        } = unsafe { self.read_type()? };

        let flags = convert_flags(flags, Group::ALLOWED_FLAGS)?;
        let name = convert_string(&name)?;
        let triangle_indices = self.read_vec(num_triangles as usize, Self::read_u16)?;

        let de::GroupSuffix { material_index } = unsafe { self.read_type()? };

        Ok(Group {
            flags,
            name,
            triangle_indices,
            material_index,
        })
    }

    fn read_materials(&mut self) -> Result<Vec<Material>> {
        let len = self.read_u16()? as usize;
        self.read_vec(len, Self::read_material)
    }

    fn read_material(&mut self) -> Result<Material> {
        let de::Material {
            name,
            ambient,
            diffuse,
            specular,
            emissive,
            shininess,
            transparency,
            mode,
            texture,
            alphamap,
        } = unsafe { self.read_type()? };

        let name = convert_string(&name)?;
        let texture = convert_path(&texture)?;
        let alphamap = convert_path(&alphamap)?;

        Ok(Material {
            name,
            ambient,
            diffuse,
            specular,
            emissive,
            shininess,
            transparency,
            mode,
            texture,
            alphamap,
        })
    }

    fn read_key_frame_data(&mut self) -> Result<KeyFrameData> {
        let de::KeyFrameData {
            animation_fps,
            current_time,
            total_frames,
        } = unsafe { self.read_type()? };
        Ok(KeyFrameData {
            animation_fps,
            current_time,
            total_frames,
        })
    }

    fn read_joints(&mut self) -> Result<Vec<Joint>> {
        let len = self.read_u16()? as usize;
        self.read_vec(len, Self::read_joint)
    }

    fn read_joint(&mut self) -> Result<Joint> {
        let de::JointPrefix {
            flags,
            name,
            parent_name,
            rotation,
            position,
            num_key_frames_rot,
            num_key_frames_trans,
        } = unsafe { self.read_type()? };

        let flags = convert_flags(flags, Joint::ALLOWED_FLAGS)?;
        let name = convert_string(&name)?;
        let parent_name = convert_string(&parent_name)?;

        let key_frames_rot = self.read_vec(num_key_frames_rot as usize, Self::read_key_frame_rot)?;
        let key_frames_trans =
            self.read_vec(num_key_frames_trans as usize, Self::read_key_frame_pos)?;

        Ok(Joint {
            flags,
            name,
            parent_name,
            rotation,
            position,
            key_frames_rot,
            key_frames_trans,
        })
    }

    fn read_key_frame_rot(&mut self) -> Result<KeyFrameRot> {
        let de::KeyFrameRot { time, rotation } = unsafe { self.read_type()? };
        Ok(KeyFrameRot { time, rotation })
    }

    fn read_key_frame_pos(&mut self) -> Result<KeyFramePos> {
        let de::KeyFramePos { time, position } = unsafe { self.read_type()? };
        Ok(KeyFramePos { time, position })
    }

    fn read_comments(&mut self) -> Result<Comments> {
        let sub_version = self.read_i32()?;
        ensure!(
            sub_version == 1,
            "unsupported comment sub-version {}",
            sub_version
        );
        let len = self.read_u32()? as usize;
        let group_comments = self.read_vec(len, Self::read_comment)?;
        let len = self.read_i32()? as usize;
        let material_comments = self.read_vec(len, Self::read_comment)?;
        let len = self.read_i32()? as usize;
        let joint_comments = self.read_vec(len, Self::read_comment)?;
        let len = self.read_i32()? as usize;
        let model_comment = match len {
            0 => None,
            1 => Some(self.read_comment()?),
            _ => bail!("invalid number of model comments"),
        };

        Ok(Comments {
            sub_version,
            group_comments,
            material_comments,
            joint_comments,
            model_comment,
        })
    }

    fn read_comment(&mut self) -> Result<Comment> {
        let de::CommentPrefix {
            index,
            comment_length,
        } = unsafe { self.read_type()? };
        let comment = self.read_string(comment_length as usize)?;
        Ok(Comment { index, comment })
    }

    fn read_vertex_ex_info(&mut self, len: usize) -> Result<VertexExInfo> {
        use VertexExInfo::*;

        let sub_version = self.read_i32()?;
        match sub_version {
            1 => Ok(SubVersion1(self.read_vec(len, Self::read_vertex_ex_1)?)),
            2 => Ok(SubVersion2(self.read_vec(len, Self::read_vertex_ex_2)?)),
            3 => Ok(SubVersion3(self.read_vec(len, Self::read_vertex_ex_3)?)),
            v => bail!("unsupported vertex ex sub-version {}", v),
        }
    }

    fn read_vertex_ex_1(&mut self) -> Result<VertexEx1> {
        let de::VertexEx1 { bone_ids, weights } = unsafe { self.read_type()? };
        Ok(VertexEx1 { bone_ids, weights })
    }

    fn read_vertex_ex_2(&mut self) -> Result<VertexEx2> {
        let de::VertexEx2 {
            bone_ids,
            weights,
            extra,
        } = unsafe { self.read_type()? };
        Ok(VertexEx2 {
            bone_ids,
            weights,
            extra,
        })
    }

    fn read_vertex_ex_3(&mut self) -> Result<VertexEx3> {
        let de::VertexEx3 {
            bone_ids,
            weights,
            extra,
        } = unsafe { self.read_type()? };
        Ok(VertexEx3 {
            bone_ids,
            weights,
            extra,
        })
    }

    fn read_joint_ex_info(&mut self, len: usize) -> Result<JointExInfo> {
        let sub_version = self.read_i32()?;
        ensure!(
            sub_version == 1,
            "unsupported joint ex sub-version {}",
            sub_version
        );
        let joint_ex = self.read_vec(len, Self::read_joint_ex)?;
        Ok(JointExInfo {
            sub_version,
            joint_ex,
        })
    }

    fn read_joint_ex(&mut self) -> Result<JointEx> {
        let de::JointEx { color } = unsafe { self.read_type()? };
        Ok(JointEx { color })
    }

    fn read_model_ex_info(&mut self) -> Result<ModelExInfo> {
        let sub_version = self.read_i32()?;
        ensure!(
            sub_version == 1,
            "unsupported model ex sub-version {}",
            sub_version
        );
        let model_ex = self.read_model_ex()?;
        Ok(ModelExInfo {
            sub_version,
            model_ex,
        })
    }

    fn read_model_ex(&mut self) -> Result<ModelEx> {
        let de::ModelEx {
            joint_size,
            transparency_mode,
            alpha_ref,
        } = unsafe { self.read_type()? };
        Ok(ModelEx {
            joint_size,
            transparency_mode,
            alpha_ref,
        })
    }

    fn read_string(&mut self, len: usize) -> Result<String> {
        Ok(str::from_utf8(self.rdr.buf_read_exact(len)?)?.to_owned())
    }

    fn read_vec<T, F>(&mut self, len: usize, f: F) -> Result<Vec<T>>
    where
        F: Fn(&mut Self) -> Result<T>,
    {
        (0..len).map(|_| f(self)).collect()
    }

    fn read_u16(&mut self) -> Result<u16> {
        unsafe { self.read_type() }
    }

    fn read_u32(&mut self) -> Result<u32> {
        unsafe { self.read_type() }
    }

    fn read_i32(&mut self) -> Result<i32> {
        unsafe { self.read_type() }
    }

    unsafe fn read_type<T>(&mut self) -> Result<T> {
        Ok(ptr::read_unaligned(
            self.rdr.buf_read_exact(mem::size_of::<T>())? as *const [u8] as *const T,
        ))
    }
}

fn convert_string(bytes: &[u8]) -> Result<String> {
    let vec = if let Some(i) = memchr(0, bytes) {
        bytes[..i].to_owned()
    } else {
        bytes.to_owned()
    };
    Ok(String::from_utf8(vec)?)
}

fn convert_path(bytes: &[u8]) -> Result<PathBuf> {
    convert_string(bytes).map(Into::into)
}

fn convert_flags(bits: u8, allowed: Flags) -> Result<Flags> {
    if let Some(flags) = Flags::from_bits(bits) {
        if allowed.contains(flags) {
            return Ok(flags);
        }
    }
    Err(format_err!("invalid flags {}", bits))
}
