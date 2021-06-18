#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use cgmath::*;
use winit::event::*;

use crate::camera;
use crate::bsp;
use crate::shot;
use crate::bsp_types;

//https://github.com/Francesco149/q3playground/blob/953877248bdaee442e57782f5f427283b61e0b14/main.c
const MOVEMENT_JUMP: i32 = 1 << 1;
pub const MOVEMENT_JUMP_THIS_FRAME: i32 = 1 << 2;
pub const MOVEMENT_JUMPING: i32 = 1 << 3;

//pub const DELTA: f32 = 0.006944;
pub const DELTA: f32 = 0.016666;

const GRAVITY: f32 = 800.0;
const STOP_SPEED: f32 = 200.0;
const MAX_SPEED: f32 = 320.0;
const MOVEMENT_ACCELERATION: f32 = 15.0;
const MOVEMENT_FRICTION: f32 = 8.0;
const AIR_STOP_ACCELERATION: f32 = 2.5;
const MOVEMENT_AIR_ACCELERATION: f32 = 7.0;
const CPM_WISH_SPEED: f32 = 30.0;
const CPM_STRAFE_ACCELERATION: f32 = 70.0;
const CPM_AIR_CONTROL_AMOUNT: f32 = 150.0;

const STEP_SIZE: f32 = 18.0;

const MAX_CLIP_PLANES: i32 = 5;
const OVERCLIP: f32 = 1.001;

pub struct Player {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub noclip: bool,
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    speed: f32,
    pub wish_dir: Vector3<f32>,
    pub movement: i32,
    mins: Vector3<f32>,
    maxs: Vector3<f32>,
    ground_normal: Vector3<f32>,
    forward_speed: f32,
    side_speed: f32,
    direction: Vector3<f32>,
    yaw: f32,
    pitch: f32,
    pub shot: shot::Shot,
    pub delta: f32,
}

impl Player {

    pub fn new(device: &wgpu::Device) -> Player {
        Player { position: Vector3::new(0.0, 0.0, 0.0), velocity: Vector3::new(0.0, 0.0, 0.0), noclip: true, amount_left: 0.0, amount_right: 0.0, amount_forward: 0.0, amount_backward: 0.0, amount_up: 0.0, amount_down: 0.0, speed: 4.0,
            wish_dir: Vector3::new(0.0, 0.0, 0.0), movement: 0,
            mins: Vector3::new(-15.0, -15.0, -24.0),
            maxs: Vector3::new(15.0, 15.0, 32.0),
            ground_normal: Vector3::new(0.0, 0.0, 0.0),
            forward_speed: 400.0,
            side_speed: 350.0,
            direction: Vector3::new(1.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            shot: shot::Shot::new(device),
            delta: 0.0,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) {

        //let amount = if state == ElementState::Pressed { 1.0 } else { 0.0 };

        match key {
            VirtualKeyCode::W => {
                if state == ElementState::Pressed {
                    self.wish_dir.x = self.forward_speed;
                }
                else if state == ElementState::Released {
                    if self.wish_dir.x > 0.0 {
                        self.wish_dir.x = 0.0;
                    }
                }
            }
            VirtualKeyCode::S => {
                if state == ElementState::Pressed {
                    self.wish_dir.x = -self.forward_speed;
                }
                else if state == ElementState::Released {
                    if self.wish_dir.x < 0.0 {
                        self.wish_dir.x = 0.0;
                    }
                }
            }
            VirtualKeyCode::A => {
                if state == ElementState::Pressed {
                    self.wish_dir.y = self.side_speed;
                }
                else if state == ElementState::Released {
                    if self.wish_dir.y > 0.0 {
                        self.wish_dir.y = 0.0;
                    }
                }
            }
            VirtualKeyCode::D => {
                if state == ElementState::Pressed {
                    self.wish_dir.y = -self.side_speed;
                }
                else if state == ElementState::Released {
                    if self.wish_dir.y < 0.0 {
                        self.wish_dir.y = 0.0;
                    }
                }
            }
            VirtualKeyCode::Space => {
                if state == ElementState::Pressed {
                    self.movement |= MOVEMENT_JUMP;
                }
                else if state == ElementState::Released {
                    self.movement &= !MOVEMENT_JUMP;
                }
            }
            _ => {},
        }
    }

    pub fn shoot(&mut self, device: &wgpu::Device, bsp: &mut bsp_types::Bsp, camera: &camera::Camera) {
        let mut work = bsp_types::TraceWork::new();
        let end = self.position + (camera.forward * 2048.0);
        let mins = cgmath::Vector3::new(0.0, 0.0, 0.0);
        let maxs = cgmath::Vector3::new(0.0, 0.0, 0.0);
        bsp.trace(&mut work, self.position, end, mins, maxs);
        self.shot.update(self.position, work.end_pos, device);
    }

    pub fn update(&mut self, camera: &camera::Camera) {

        self.yaw = -camera.yaw;
        self.pitch = -camera.pitch;
    }

    pub fn apply_inputs(&mut self) {

        let pitch_sin = 0.0;
        let pitch_cos = 1.0;

        let yaw_sin = ((2.0 * std::f32::consts::PI) - self.yaw).sin();
        let yaw_cos = ((2.0 * std::f32::consts::PI) - self.yaw).cos();


        let pitch_x = self.wish_dir.x * pitch_cos + self.wish_dir.z * (-pitch_sin);
        let mut direction = cgmath::Vector3::new(pitch_x * yaw_cos + self.wish_dir.y * (-yaw_sin),
                                            pitch_x * yaw_sin + self.wish_dir.y * yaw_cos,
                                            self.wish_dir.x * pitch_sin + self.wish_dir.z * pitch_cos);

        let mut wish_speed = cgmath::dot(direction, direction).sqrt();
        if wish_speed >= 0.0001 {
            direction = direction / wish_speed;
        }
        wish_speed = wish_speed.min(MAX_SPEED);

        self.apply_jump();
        self.apply_friction();

        let mut selected_acceleration = MOVEMENT_ACCELERATION;
        let base_wishspeed = wish_speed;

        if (self.movement & MOVEMENT_JUMPING) != 0 || (self.movement & MOVEMENT_JUMP_THIS_FRAME) != 0 {

            if cgmath::dot(self.velocity, direction) < 0.0 {
                selected_acceleration = AIR_STOP_ACCELERATION;
            }
            else {
                selected_acceleration = MOVEMENT_AIR_ACCELERATION;
            }

            if self.wish_dir.y != 0.0 && self.wish_dir.x == 0.0 {
                wish_speed = wish_speed.min(CPM_WISH_SPEED);
                selected_acceleration = CPM_STRAFE_ACCELERATION;
            }
        }

        self.apply_acceleration(&mut direction, wish_speed, selected_acceleration);
        self.apply_air_control(&mut direction, base_wishspeed);
    }

    fn apply_acceleration(&mut self, direction: &mut cgmath::Vector3<f32>, mut wish_speed: f32, acceleration: f32) {

        if (self.movement & MOVEMENT_JUMPING) > 0 {
            wish_speed = wish_speed.min(CPM_WISH_SPEED);
        }

        let cur_speed = cgmath::dot(self.velocity, direction.clone());
        let add_speed = wish_speed - cur_speed;

        if add_speed <= 0.0 {
            return;
        }

        let mut accel_speed = acceleration * wish_speed * self.delta;
        accel_speed = accel_speed.min(add_speed);

        let amount = direction.clone() * accel_speed;
        self.velocity += amount;
    }

    //https://github.com/lambdacube3d/lambdacube-quake3/blob/2fbc17eeea7567f2876d86a7ccce4bd8c8e1927c/game/Movers.hs
    fn apply_air_control(&mut self, direction: &mut cgmath::Vector3<f32>, wish_speed: f32) {

        if self.wish_dir.x == 0.0 || wish_speed == 0.0 {
            return;
        }

        let zspeed = self.velocity.z;
        self.velocity.z = 0.0;
        let speed = cgmath::dot(self.velocity, self.velocity).sqrt();
        if speed > 0.0001 {
            self.velocity /= speed;
        }
        let dot = cgmath::dot(self.velocity, direction.clone());

        if dot > 0.0 {

            let k = 32.0 * CPM_AIR_CONTROL_AMOUNT * dot * dot * self.delta;
            self.velocity *= speed;
            let amount = direction.clone() * k;
            self.velocity = self.velocity.normalize();
        }

        self.velocity *= speed;
        self.velocity.z = zspeed;
    }

    fn apply_jump(&mut self) {

        if ((self.movement & MOVEMENT_JUMP)) == 0 {
            return;
        }
        if (self.movement & MOVEMENT_JUMPING) != 0 {
            return;
        }

        self.movement |= MOVEMENT_JUMP_THIS_FRAME;
        self.velocity.z = 270.0;
    }

    fn apply_friction(&mut self) {

        if (self.movement & MOVEMENT_JUMPING) != 0 || (self.movement & MOVEMENT_JUMP_THIS_FRAME) != 0 {
            return;
        }

        let speed = cgmath::dot(self.velocity, self.velocity).sqrt();
        if speed < 1.0 {
            self.velocity.x = 0.0;
            self.velocity.y = 0.0;
            return;
        }

        let mut control = speed;
        if speed < STOP_SPEED {
            control = STOP_SPEED;
        }

        let mut new_speed = speed - control * MOVEMENT_FRICTION * self.delta;
        new_speed = new_speed.max(0.0);
        self.velocity *= (new_speed / speed);
    }

    pub fn trace_ground(&mut self, bsp: &mut bsp_types::Bsp) {

        let mut end = self.position;
        end.z = end.z - 0.25;

        let mut work = bsp_types::TraceWork::new();
        bsp.trace(&mut work, self.position, end, self.mins, self.maxs);

        if work.frac == 1.0 || (self.movement & MOVEMENT_JUMP_THIS_FRAME) != 0 {
            self.movement |= MOVEMENT_JUMPING;
            self.ground_normal = Vector3::new(0.0, 0.0, 0.0);
        }
        else {

            self.movement &= !MOVEMENT_JUMPING;
            self.ground_normal.x = work.plane.normal[0];
            self.ground_normal.y = work.plane.normal[1];
            self.ground_normal.z = work.plane.normal[2];
            //self.velocity.z = 0.0;
        }
    }

    pub fn slide_step(&mut self, bsp: &mut bsp_types::Bsp) {

        let mut start_o = self.position;
        let mut start_v = self.velocity;

        let gravity = (self.movement & MOVEMENT_JUMPING != 0);

        if self.slide(bsp) == 0 {
            return;
        }

        let mut down = start_o;
        down.z -= STEP_SIZE;
        let mut work = bsp_types::TraceWork::new();
        bsp.trace(&mut work, start_o, down, self.mins, self.maxs);
        let mut plane_n = cgmath::Vector3::new(work.plane.normal[0], work.plane.normal[1], work.plane.normal[2]);
        if self.velocity.z > 0.0 && (work.frac == 1.0 || cgmath::dot(plane_n, Vector3::unit_z()) < 0.7) {
            return;
        }

        let mut down_o = self.position;
        let mut down_v = self.velocity;

        let mut up = start_o;
        up.z += STEP_SIZE;
        work = bsp_types::TraceWork::new();
        bsp.trace(&mut work, start_o, up, self.mins, self.maxs);

        if work.flags == 0 {

            return;
        }

        let step_size = work.end_pos.z - start_o.z;

        self.position = work.end_pos;
        self.velocity = start_v;

        let waste = self.slide(bsp);

        down = self.position;
        down.z -= step_size;
        work = bsp_types::TraceWork::new();
        bsp.trace(&mut work, self.position, down, self.mins, self.maxs);
        if work.flags != 0 {
            self.position = work.end_pos;
        }
        if work.frac < 1.0 {
            plane_n = cgmath::Vector3::new(work.plane.normal[0], work.plane.normal[1], work.plane.normal[2]);
            Player::clip_velocity(&self.velocity.clone(), &plane_n, &mut self.velocity, OVERCLIP);
        }
    }

    pub fn slide(&mut self, bsp: &mut bsp_types::Bsp) -> i32 {

        let gravity = (self.movement & MOVEMENT_JUMPING != 0);

        let mut n_planes = 0;
        let mut end_velocity = cgmath::Vector3::new(0.0, 0.0, 0.0);
        let mut planes: [cgmath::Vector3<f32>; MAX_CLIP_PLANES as usize] = [cgmath::Vector3::new(0.0, 0.0, 0.0); MAX_CLIP_PLANES as usize];

        let mut time_left = self.delta;

        let mut out_bumps = 0;

        if gravity {

            end_velocity = self.velocity;
            end_velocity.z -= GRAVITY * self.delta;

            self.velocity.z = (end_velocity.z + self.velocity.z) * 0.5;

            if self.ground_normal != cgmath::Vector3::new(0.0, 0.0, 0.0) {

                Player::clip_velocity(&self.velocity.clone(), &self.ground_normal, &mut self.velocity, OVERCLIP);
            }
        }

        if self.ground_normal != cgmath::Vector3::new(0.0, 0.0, 0.0) {
            planes[n_planes as usize] = self.ground_normal;
            n_planes = n_planes + 1;
        }

        planes[n_planes as usize] = self.velocity.normalize();
        n_planes = n_planes + 1;

        for n_bumps in 0..4 {
            out_bumps = n_bumps;
            let mut work = bsp_types::TraceWork::new();
            let mut work2 = bsp_types::TraceWork::new();

            let mut ic = 0;

            let mut end = self.velocity * time_left;
            end = end + self.position;
            bsp.trace(&mut work, self.position, end, self.mins, self.maxs);

            if work.frac > 0.0 {
                self.position = work.end_pos;
            }

            if work.frac == 1.0 {
                break;
            }

            time_left -= time_left * work.frac;

            if n_planes >= MAX_CLIP_PLANES {
                self.velocity = cgmath::Vector3::new(0.0, 0.0, 0.0);
                return 1;
            }

            let plane_normal = cgmath::Vector3::new(work.plane.normal[0], work.plane.normal[1], work.plane.normal[2]);

            planes[n_planes as usize] = plane_normal;
            n_planes = n_planes + 1;

            for i in 0..n_planes {

                let mut clipped = cgmath::Vector3::new(0.0, 0.0, 0.0);
                let mut end_clipped = cgmath::Vector3::new(0.0, 0.0, 0.0);


                if cgmath::dot(self.velocity, planes[i as usize]) >= 0.1 {
                    continue;
                }

                Player::clip_velocity(&self.velocity, &planes[i as usize], &mut clipped, OVERCLIP);
                Player::clip_velocity(&end_velocity, &planes[i as usize], &mut end_clipped, OVERCLIP);

                for j in 0..n_planes {

                    let mut dir = cgmath::Vector3::new(0.0, 0.0, 0.0);
                    let mut speed = 0.0;

                    if j == i {
                        continue;
                    }

                    if cgmath::dot(clipped, planes[j as usize]) >= 0.1 {
                        continue;
                    }

                    Player::clip_velocity(&clipped.clone(), &planes[j as usize], &mut clipped, OVERCLIP);
                    Player::clip_velocity(&end_clipped.clone(), &planes[j as usize], &mut end_clipped, OVERCLIP);
                    
                    if cgmath::dot(clipped, planes[i as usize]) >= 0.0 {
                        continue;
                    }

                    dir = planes[i as usize].cross(planes[j as usize]);
                    dir = dir.normalize();

                    speed = cgmath::dot(dir, self.velocity);
                    clipped = dir;
                    clipped *= speed;

                    speed = cgmath::dot(dir, end_velocity);
                    end_clipped = dir;
                    end_clipped *= speed;
                    for k in 0..n_planes {
                        if k == j || j == i {
                            continue;
                        }

                        if cgmath::dot(clipped, planes[k as usize]) >= 0.1 {
                            continue;
                        }
                        self.velocity = cgmath::Vector3::new(0.0, 0.0, 0.0);
                        return 1;
                    }
                }

                self.velocity = clipped;
                end_velocity = end_clipped;
                break;
            }
        }

        if gravity {
            self.velocity = end_velocity;
        }

        if out_bumps != 0 {
            out_bumps = 1;
        }

        out_bumps
    }

    fn clip_velocity(inv: &cgmath::Vector3<f32>, normal: &cgmath::Vector3<f32>, out: &mut cgmath::Vector3<f32>, overbounce: f32) {

        let mut backoff = cgmath::dot(inv.clone(), normal.clone());

        if backoff < 0.0 {
            backoff *= overbounce;
        }
        else {
            backoff /= overbounce;
        }

        for i in 0..3 {

            let change = normal[i] * backoff;
            out[i] = inv[i] - change;
        }
    }
}