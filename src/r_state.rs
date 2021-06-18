#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{Window, WindowBuilder},
};
use futures::executor::block_on;
use wgpu::util::DeviceExt;

use cgmath::SquareMatrix;

use crate::texture;
use crate::uniform;
use crate::player;
use crate::bsp;
use crate::camera;
use crate::shot;
use crate::r_pipeline;
use crate::bsp_types;

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    bsp_model_render_pipeline: wgpu::RenderPipeline,
    sky_pipeline: wgpu::RenderPipeline,
    shot_pipeline: wgpu::RenderPipeline,
    camera: camera::Camera,
    projection: camera::Projection,
    camera_controller: camera::CameraController,
    uniforms: uniform::Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    depth_texture: texture::Texture,
    bsp: bsp_types::Bsp,
    player: player::Player,
}

impl State {

    pub async fn new(window: &Window) -> Self {

        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default, //HighPerformance
                compatible_surface: Some(&surface),
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                shader_validation: true,
            },
            None,
        ).await.unwrap();

        //Fifo or Immediate (vsync on and off) Mailbox (double buffering)
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let mut camera = camera::Camera::new();
        let projection = camera::Projection::new(sc_desc.width, sc_desc.height, cgmath::Deg(90.0), 0.1, 4000.0);
        let camera_controller = camera::CameraController::new(400.0, 3.0);

        let mut uniforms = uniform::Uniforms::new();
        uniforms.update_view_proj(&camera, &projection);

        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            }
        );

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("uniform_bind_group_layout"),
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..)),
                }
            ],
            label: Some("uniform_bind_group"),
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Uint,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let lightmap_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2Array,
                        component_type: wgpu::TextureComponentType::Uint,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let mut bsp = bsp_types::Bsp::new(&device, &queue, &texture_bind_group_layout, &lightmap_bind_group_layout);

        let depth_texture = texture::Texture::create_depth_texture(&device, &sc_desc, "depth_texture");

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &lightmap_bind_group_layout, &uniform_bind_group_layout],
            push_constant_ranges: &[],
        });
        let render_pipeline = r_pipeline::pipeline(&device, &sc_desc, &render_pipeline_layout, wgpu::include_spirv!("bsp.vert.spv"), wgpu::include_spirv!("bsp.frag.spv"), bsp_types::VertexExt::desc(), wgpu::PrimitiveTopology::TriangleList);

        let bsp_model_render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
            push_constant_ranges: &[],
        });
        let bsp_model_render_pipeline = r_pipeline::pipeline(&device, &sc_desc, &bsp_model_render_pipeline_layout, wgpu::include_spirv!("bsp_model.vert.spv"), wgpu::include_spirv!("bsp_model.frag.spv"), bsp_types::VertexExt::desc(), wgpu::PrimitiveTopology::TriangleList);
        let sky_pipeline = r_pipeline::pipeline(&device, &sc_desc, &bsp_model_render_pipeline_layout, wgpu::include_spirv!("bsp_sky.vert.spv"), wgpu::include_spirv!("bsp_sky.frag.spv"), bsp_types::VertexExt::desc(), wgpu::PrimitiveTopology::TriangleList);
        let shot_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });
        let shot_pipeline = r_pipeline::pipeline(&device, &sc_desc, &shot_layout, wgpu::include_spirv!("shot.vert.spv"), wgpu::include_spirv!("shot.frag.spv"), shot::Vertex::desc(), wgpu::PrimitiveTopology::LineList);

        let player = player::Player::new(&device);

        //camera.position = cgmath::Point3::new(466.8673, 234.07431, 99.91692);

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            render_pipeline,
            bsp_model_render_pipeline,
            sky_pipeline,
            shot_pipeline,
            camera,
            projection,
            camera_controller,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            depth_texture,
            bsp,
            player,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {

        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.projection.resize(new_size.width, new_size.height);
        self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.sc_desc, "depth_texture");
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    pub fn input(&mut self, event: &WindowEvent, window: &Window) {

        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                if *state == ElementState::Pressed { 
                    if *keycode == VirtualKeyCode::F {
                        self.player.noclip = true;
                    }
                    else if *keycode == VirtualKeyCode::G {
                        self.player.noclip = false;
                        self.player.position = cgmath::Vector3::new(self.camera.position[0], self.camera.position[1], self.camera.position[2]);
                    }
                    else if *keycode == VirtualKeyCode::H {
                        self.player.noclip = true;
                    }
                }
                if *keycode == VirtualKeyCode::E {
                    self.player.shoot(&self.device, &mut self.bsp, &self.camera);
                }
                if self.player.noclip {
                    self.camera_controller.process_keyboard(*keycode, *state);
                }
                else {
                    self.player.process_keyboard(*keycode, *state);
                }
            }
            WindowEvent::CursorMoved  { position, .. } => {
                self.camera_controller.process_mouse((position.x as f32 / self.sc_desc.width as f32) - 0.5, 0.5 - (position.y as f32 / self.sc_desc.height as f32), &mut self.camera);
                self.player.update(&self.camera);
                window.set_cursor_position(winit::dpi::PhysicalPosition::new(self.sc_desc.width as f32 / 2.0, self.sc_desc.height as f32 / 2.0));
            }
            WindowEvent::MouseInput { state, button, .. } => {
                //mouse button press
                if *state == ElementState::Pressed {
                    self.player.shoot(&self.device, &mut self.bsp, &self.camera);
                }
            }
            _ => {},
        }
    }

    pub fn update(&mut self, delta: f32) {

        self.player.delta = delta;
        self.camera_controller.update_camera(&mut self.camera, delta);
        self.uniforms.update_view_proj(&self.camera, &self.projection);
        self.uniforms.update_time(player::DELTA);
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniforms]));

        if !self.player.noclip {
            
            self.player.trace_ground(&mut self.bsp);
            self.player.apply_inputs();
            self.player.slide_step(&mut self.bsp);
            self.player.movement &= !player::MOVEMENT_JUMP_THIS_FRAME;
            self.camera.position = cgmath::Point3::new(self.player.position[0], self.player.position[1], self.player.position[2] + 24.0);
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {

        let frame = self.swap_chain.get_current_frame()?.output;
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        let mut offset = 0usize;
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: true,
                        }
                    }
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            //Draw bsp
            let model_index = (self.bsp.indices_per_texture.len() - 1);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.bsp.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.bsp.index_buffer.slice(..));
            render_pass.set_bind_group(2, &self.uniform_bind_group, &[]);
            for j in 0..(self.bsp.indices_per_texture.len() - 1) {
                render_pass.set_bind_group(1, &self.bsp.materials_light[j].bind_group, &[]);
                for i in 0..self.bsp.indices_per_texture[j].len() {
                    if self.bsp.indices_per_texture[j][i].len() != 0 {
                        render_pass.set_bind_group(0, &self.bsp.materials[i].bind_group, &[]);
                        render_pass.draw_indexed((offset as u32)..((offset + self.bsp.indices_per_texture[j][i].len()) as u32), 0, 0..1);
                        offset += self.bsp.indices_per_texture[j][i].len();
                    }
                }
            }

            //Draw models
            render_pass.set_pipeline(&self.bsp_model_render_pipeline);
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
            for i in 0..self.bsp.indices_per_texture[model_index].len() {
                if self.bsp.indices_per_texture[model_index][i].len() != 0 /*&& self.bsp.texture_shader[i] == 0*/ {
                    render_pass.set_bind_group(0, &self.bsp.materials[i].bind_group, &[]);
                    render_pass.draw_indexed((offset as u32)..((offset + self.bsp.indices_per_texture[model_index][i].len()) as u32), 0, 0..1);
                    offset += self.bsp.indices_per_texture[model_index][i].len();
                }
            }

            //Draw bullets
            render_pass.set_pipeline(&self.shot_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.player.shot.buf.slice(..));
            render_pass.draw(0..(self.player.shot.verts.len() as u32), 0..1);
                                 
        }
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}