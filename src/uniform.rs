#![allow(unused_variables)]
#![allow(unused_imports)]

use wgpu::util::DeviceExt;
use cgmath::SquareMatrix;
use cgmath::InnerSpace;
use cgmath::Rotation3;
use cgmath::Zero;

use crate::camera;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    time: f32,
}

impl Uniforms {
    pub fn new() -> Self {
        Self {
            proj: cgmath::Matrix4::identity().into(),
            view: cgmath::Matrix4::identity().into(),
            model: cgmath::Matrix4::identity().into(),
            time: 0.0,
        }
    }

    pub fn update_view_proj(&mut self, camera: &camera::Camera, projection: &camera::Projection) {
        self.proj = projection.calc_matrix().into();
        self.view =  camera.view.into();
    }

    pub fn update_time(&mut self, time: f32) {
        self.time += time;
    } 

    pub fn update_model(&mut self, model: cgmath::Matrix4<f32>) {
        self.model = model.into();
    }
}