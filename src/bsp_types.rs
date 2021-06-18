use crate::texture;

pub const TEXTURE_SIZE: u32 = 72;

pub const POLYGON: i32 = 1;
pub const PATCH: i32 = 2;
pub const MESH: i32 = 3;
pub const BILLBOARD: i32 = 4;
pub const BEZIER_LEVEL: i32 = 5;

pub const RAY: i32 = 0;
pub const SPHERE: i32 = 1;
pub const BOX: i32 = 2;

pub const PLANE_X: i32 = 0;
pub const PLANE_Y: i32 = 1;
pub const PLANE_Z: i32 = 2;
pub const PLANE_NONAXIAL: i32 = 3;
pub const LAST_PLANE_TYPE: i32 = 4;

pub const SURF_CLIP_EPSILON: f32 = 0.125;
pub const TW_STARTS_OUT: i32 = 1<<1;
pub const TW_ENDS_OUT: i32 = 1<<2;
pub const TW_ALL_SOLID: i32 = 1<<3;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VisData {
    pub num_vecs: i32,
    pub size_vecs: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightVol {
    pub ambient: [u8; 3],
    pub directional: [u8; 3],
    pub dir: [u8; 2],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightMap {
    pub map: [[[u8; 3]; 128]; 128],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Face {
    pub texture: i32,
    pub effect: i32,
    pub type_draw: i32,
    pub vertex: i32,
    pub num_vertexes: i32,
    pub mesh_vert: i32,
    pub num_mesh_verts: i32,
    pub lightmap_index: i32,
    pub lightmap_start: [i32; 2],
    pub lightmap_size: [i32; 2],
    pub lightmap_origin: [f32; 3],
    pub lightmap_vecs: [[f32; 3]; 2],
    pub normal: [f32; 3],
    pub size: [i32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Effect {
    pub name: [u8; 64],
    pub brush: i32,
    pub unknown: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVert {
    pub offset: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub texcoord_s: [f32; 2],
    pub texcoord_l: [f32; 2],
    pub normal: [f32; 3],
    pub colour: [u8; 4],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 10]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uchar4Norm,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexExt {
    pub position: [f32; 3],
    pub texcoord_s: [f32; 2],
    pub texcoord_l: [f32; 3],
    pub normal: [f32; 3],
    pub colour: [u8; 4],
}

impl VertexExt {
    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<VertexExt>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uchar4Norm,
                },
            ],
        }
    }

    pub fn ext_vertex(vertex: Vertex, tex: f32) -> VertexExt {
        VertexExt { position: vertex.position, texcoord_s: vertex.texcoord_s, texcoord_l: [vertex.texcoord_l[0], vertex.texcoord_l[1], tex], normal: vertex.normal, colour: vertex.colour }
    }

    pub fn new() -> VertexExt {
        VertexExt { position: [0.0, 0.0, 0.0], texcoord_s: [0.0, 0.0], texcoord_l: [0.0, 0.0, 0.0], normal: [0.0, 0.0, 0.0], colour: [255, 255, 255, 255] }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BrushSide {
    pub plane: i32,
    pub texture: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Brush {
    pub brush_side: i32,
    pub num_brush_sides: i32,
    pub texture: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Model {
    pub mins: [i32; 3],
    pub maxs: [i32; 3],
    pub face: i32,
    pub num_faces: i32,
    pub brush: i32,
    pub num_brushes: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LeafBrush {
    pub brush: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LeafFace {
    pub face: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Leaf {
    pub cluster: i32,
    pub area: i32,
    pub mins: [i32; 3],
    pub maxs: [i32; 3],
    pub leaf_face: i32,
    pub num_leaf_faces: i32,
    pub leaf_brush: i32,
    pub num_leaf_brushes: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Node {
    pub plane: i32,
    pub children: [i32; 2],
    pub mins: [i32; 3],
    pub maxs: [i32; 3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Plane {
    pub normal: [f32; 3],
    pub distance: f32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Texture {
    pub name: [u8; 64],
    pub flags: i32,
    pub contents: i32,
}

pub struct Material {
    pub diffuse_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}

pub struct TraceWork {
    pub start: cgmath::Vector3<f32>,
    pub end: cgmath::Vector3<f32>,
    pub end_pos: cgmath::Vector3<f32>,
    pub frac: f32,
    pub flags: i32,
    pub mins: cgmath::Vector3<f32>,
    pub maxs: cgmath::Vector3<f32>,
    pub offsets: [[f32; 3]; 8],
    pub plane: Plane,
    pub sphere: bool,
    pub radius: f32,
    pub sphere_offset: cgmath::Vector3<f32>,
}

impl TraceWork {
    pub fn new() -> TraceWork {
        TraceWork { start: cgmath::Vector3::new(0.0, 0.0, 0.0), end: cgmath::Vector3::new(0.0, 0.0, 0.0), 
            end_pos: cgmath::Vector3::new(0.0, 0.0, 0.0), frac: 1.0,
            flags: 0, mins: cgmath::Vector3::new(0.0, 0.0, 0.0),
            maxs: cgmath::Vector3::new(0.0, 0.0, 0.0),
            offsets: [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
            plane: Plane { normal: [0.0, 0.0, 0.0], distance: 1.0 },
            sphere: false,
            radius: 0.0,
            sphere_offset: cgmath::Vector3::new(0.0, 0.0, 0.0),
        }
    }
}

pub struct Trace {
    pub output_fraction: f32,
    pub output_end: cgmath::Vector3<f32>,
    pub output_starts_out: bool,
    pub output_all_solid: bool,
    pub start: cgmath::Vector3<f32>,
    pub end: cgmath::Vector3<f32>,
    pub radius: f32,
    pub mins: cgmath::Vector3<f32>,
    pub maxs: cgmath::Vector3<f32>,
    pub extents: cgmath::Vector3<f32>,
    pub t_type: i32,
}

impl Trace {

    pub fn new() -> Trace {
        Trace { output_fraction: 1.0, output_end: cgmath::Vector3::new(0.0, 0.0, 0.0), output_starts_out: true, output_all_solid: false, start: cgmath::Vector3::new(0.0, 0.0, 0.0), 
            end: cgmath::Vector3::new(0.0, 0.0, 0.0), radius: 1.0, mins: cgmath::Vector3::new(0.0, 0.0, 0.0), maxs: cgmath::Vector3::new(2.0, 2.0, 2.0), extents: cgmath::Vector3::new(1.0, 1.0, 1.0),
            t_type: RAY }
    }
}

pub struct Bsp {
    pub planes: Vec<Plane>,
    pub nodes: Vec<Node>,
    pub leafs: Vec<Leaf>,
    pub leaf_faces: Vec<LeafFace>,
    pub leaf_brushes: Vec<LeafBrush>,
    pub brushes: Vec<Brush>,
    pub brush_sides: Vec<BrushSide>,
    pub vertexes_ext: Vec<VertexExt>,
    pub mesh_verts: Vec<MeshVert>,
    pub faces: Vec<Face>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub light_maps: Vec<LightMap>,
    pub light_vols: Vec<LightVol>,
    pub t_trace: Trace,
    pub indices_per_texture: Vec<Vec<Vec<u32>>>,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub materials_light: Vec<Material>,
    pub texture_shader: Vec<i32>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Header {
    pub magic: [u8; 4],
    pub version: i32,
    pub entities: Entry,
    pub textures: Entry,
    pub planes: Entry,
    pub nodes: Entry,
    pub leafs: Entry,
    pub leaf_faces: Entry,
    pub leaf_brushes: Entry,
    pub models: Entry,
    pub brushes: Entry,
    pub brush_sides: Entry,
    pub vertexes: Entry,
    pub mesh_verts: Entry,
    pub effects: Entry,
    pub faces: Entry,
    pub light_maps: Entry,
    pub light_vols: Entry,
    pub visdata: Entry,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Entry {
    pub offset: i32,
    pub size: i32,
}