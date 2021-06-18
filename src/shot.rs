use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pos: [f32; 3],
    col: [f32; 3],
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
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}

pub struct Shot {
    pub verts: Vec<Vertex>,
    pub buf: wgpu::Buffer,
}

impl Shot {

    pub fn new(device: &wgpu::Device) -> Shot {

        let verts: Vec<Vertex> = Vec::new();

        let buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&verts),
                usage: wgpu::BufferUsage::VERTEX,
            }
        );

        Shot { verts, buf }
    }

    pub fn update(&mut self, start: cgmath::Vector3<f32>, end: cgmath::Vector3<f32>, device: &wgpu::Device) {

        self.verts.push(Vertex { pos: [start.x, start.y, start.z], col: [1.0, 0.0, 0.0] });
        self.verts.push(Vertex { pos: [end.x, end.y, end.z], col: [1.0, 0.0, 0.0] });

        self.buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.verts),
                usage: wgpu::BufferUsage::VERTEX,
            }
        );
    }
}