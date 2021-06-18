#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use std::io;
use std::io::prelude::*;
use std::fs::File;
use wgpu::util::DeviceExt;
use std::io::{stdin,stdout,Write};

use crate::texture;
use crate::bsp_look_up;
use crate::bsp_types::*;

//http://www.mralligator.com/q3/#Nodes
//https://web.archive.org/web/20071010003301/http://www.devmaster.net/articles/quake3collision/

impl Bsp {

    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, layout: &wgpu::BindGroupLayout, light_layout: &wgpu::BindGroupLayout) -> Bsp {

        //Optimaztion 
        let mut textures: Vec<Texture> = Vec::new();
        let mut found_textures: Vec<bool> = Vec::new();
        let mut texture_shader: Vec<i32> = Vec::new();

        //Could load from user input, but dont use at the moment
        let res_dir = std::path::Path::new("res");
        let mut s="q3dm7".to_string();//String::new();
        /*print!("Please enter map name: ");
        let _=stdout().flush();
        stdin().read_line(&mut s).expect("Did not enter a correct string");
        if let Some('\n')=s.chars().next_back() {
            s.pop();
        }
        if let Some('\r')=s.chars().next_back() {
            s.pop();
        }*/

        let mut baseq3_pak0 = "baseq3/pak0.pk3".to_string();
        let mut f = std::fs::File::open(res_dir.join(baseq3_pak0)).unwrap();
        let mut reader = std::io::BufReader::new(f);
        let mut zip = zip::ZipArchive::new(reader).unwrap();

        let mut map = "maps/".to_string();
        map.push_str(&s);
        map.push_str(".bsp");
        let bytes = zip.by_name(&map).unwrap().bytes().map(|x| x.unwrap()).collect::<Vec<u8>>();

        let header = *bytemuck::from_bytes::<Header>(&bytes[..std::mem::size_of::<Header>()]);

        if !(header.magic[0] == 'I' as u8 && header.magic[1] == 'B' as u8 && header.magic[2] == 'S' as u8 && header.magic[3] == 'P' as u8) {
            panic!("Invalid BSP");
        }

        let mut vertexes = parse_data::<Vertex>(header.vertexes, &bytes);
        let mut mesh_verts = parse_data::<MeshVert>(header.mesh_verts, &bytes);
        let mut planes = parse_data::<Plane>(header.planes, &bytes);
        let mut nodes = parse_data::<Node>(header.nodes, &bytes);
        let mut leafs = parse_data::<Leaf>(header.leafs, &bytes);
        let mut leaf_faces = parse_data::<LeafFace>(header.leaf_faces, &bytes);
        let mut leaf_brushes = parse_data::<LeafBrush>(header.leaf_brushes, &bytes);
        let mut brushes = parse_data::<Brush>(header.brushes, &bytes);
        let mut brush_sides = parse_data::<BrushSide>(header.brush_sides, &bytes);
        let mut faces = parse_data::<Face>(header.faces, &bytes);
        let mut light_maps = parse_data::<LightMap>(header.light_maps, &bytes);
        let mut light_vols = parse_data::<LightVol>(header.light_vols, &bytes);
        let mut effects = parse_data::<Effect>(header.effects, &bytes);
        let mut entities = std::str::from_utf8(&parse_data::<u8>(header.entities, &bytes)).unwrap();
        
        //Textures
        let textures_offset = unsafe { std::mem::transmute::<[u8; 4], u32>([bytes[16], bytes[17], bytes[18], bytes[19]]) }.to_le();
        let textures_length = unsafe { std::mem::transmute::<[u8; 4], u32>([bytes[20], bytes[21], bytes[22], bytes[23]]) }.to_le();
        
        for i in 0..(textures_length / TEXTURE_SIZE) {
            let mut temp: [u8; TEXTURE_SIZE as usize] = [0; TEXTURE_SIZE as usize];
            for j in 0..TEXTURE_SIZE {
                temp[j as usize] = bytes[(textures_offset + (i * TEXTURE_SIZE) + j) as usize];
            }
            let texture = bytemuck::from_bytes::<Texture>(&temp).clone();
            let tex: String = std::str::from_utf8(&texture.name).unwrap().chars().filter(|c| *c != 0 as char).collect();
            if tex.contains("skies") {
                texture_shader.push(1);
            }
            else {
                texture_shader.push(0);
            }
            found_textures.push(false);
            textures.push(texture);
        }

        //Start of mesh building
        let mut vertexes_ext: Vec<VertexExt> = vec![VertexExt::new(); vertexes.len()];
        let mut indices_per_texture: Vec<Vec<Vec<u32>>> = vec![vec![Vec::new(); textures.len()]; /*light_maps.len() + 1*/ 2];
        let num_per_width = (light_maps.len() as f32).sqrt().ceil() as usize; //Light map optimization value
        for i in 0..(faces.len()) {
            
            let mut li = faces[i].lightmap_index as usize;
            let truli = li;
            if li >= light_maps.len() {
                li = 1;
            }else {
                li = 0;
            }

            if faces[i].type_draw == POLYGON {
                for j in 0..(faces[i].num_mesh_verts) {
                    let indice = (faces[i as usize].vertex + mesh_verts[(faces[i as usize].mesh_vert + j) as usize].offset) as u32;
                    let x = ((truli % num_per_width) as f32) / (num_per_width as f32);
                    let y = ((truli / num_per_width) as f32) / (num_per_width as f32);
                    let aa = vertexes[indice as usize].texcoord_l;
                    let new_vert = vertexes[indice as usize];
                    vertexes_ext[indice as usize] = VertexExt::ext_vertex(new_vert, truli as f32);
                    indices_per_texture[li][faces[i].texture as usize].push(indice);
                }
            }
            else if faces[i].type_draw == PATCH {
                
                //https://github.com/mikezila/uQuake3/blob/master/uQuake/Scripts/uQuake/GenerateMap.cs
                //https://github.com/mikezila/uQuake3/blob/master/uQuake/Scripts/uQuake/Types/BezierMesh.cs
                let num_patches = ((faces[i].size[0] - 1) / 2) * ((faces[i].size[1] - 1) / 2);
                for j in 0..num_patches {
                    let (i_vertexes, i_inds) = Bsp::gen_bez_mesh(&faces[i], j, &vertexes);

                    let offset = vertexes.len() as u32;
                    for l in 0..i_vertexes.len() {
                        vertexes.push(i_vertexes[l]);
                        vertexes_ext.push(VertexExt::ext_vertex(i_vertexes[l], truli as f32));
                    }
                    for l in 0..i_inds.len() {
                        indices_per_texture[li][faces[i].texture as usize].push(offset + i_inds[l]);
                    }
                }
            }
            else if faces[i].type_draw == MESH {
                for j in 0..(faces[i].num_mesh_verts) {
                    let indice = (faces[i as usize].vertex + mesh_verts[(faces[i as usize].mesh_vert + j) as usize].offset) as u32;
                    indices_per_texture[li][faces[i].texture as usize].push(indice);
                    let new_vert = vertexes[indice as usize];
                    vertexes_ext[indice as usize] = VertexExt::ext_vertex(new_vert, truli as f32);
                }
            }
            else if faces[i].type_draw == BILLBOARD {
                //Todo
            }
        }

        //Mesh building
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertexes_ext),
                usage: wgpu::BufferUsage::VERTEX,
            }
        );

        let mut indices_p_t: Vec<u32> = Vec::new();
        for j in 0..indices_per_texture.len() {
            for i in 0..indices_per_texture[j].len() {
                for l in 0..indices_per_texture[j][i].len() {
                    indices_p_t.push(indices_per_texture[j][i][l]);
                }
            }
        }

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices_p_t),
                usage: wgpu::BufferUsage::INDEX,
            }
        );

        //Lightmaps
        let mut materials_light: Vec<Material> = Vec::new();
        let mut mega_light_map: Vec<u8> = Vec::new();
        for i in 0..light_maps.len() {
            for x in 0..128 {
                for y in 0..128 {
                    let pixel = light_maps[i].map[x][y];
                    mega_light_map.push(pixel[0]);
                    mega_light_map.push(pixel[1]);
                    mega_light_map.push(pixel[2]);
                    mega_light_map.push(255);
                }
            }
        }
        let tex = texture::Texture::from_array_to_array(device, queue, bytemuck::cast_slice(&mega_light_map), 128, light_maps.len() as u32, "lightmap").unwrap();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: light_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&tex.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&tex.sampler),
                },
            ],
            label: None,
        });
        materials_light.push(Material { diffuse_texture: tex, bind_group });

        let mut materials: Vec<Material> = Vec::new();

        for i in 0..textures.len() {
            let tex_t = texture::Texture::load(device, queue, res_dir.join("debug.jpg")).unwrap();

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&tex_t.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&tex_t.sampler),
                    },
                ],
                label: None,
            });

            materials.push(Material { diffuse_texture: tex_t, bind_group });
        }

        //Textures
        Bsp::load_from_pak("pak0.pk3", &textures, &mut materials, device, queue, layout, &mut found_textures);
        Bsp::load_from_pak("pak1.pk3", &textures, &mut materials, device, queue, layout, &mut found_textures);
        Bsp::load_from_pak("pak2.pk3", &textures, &mut materials, device, queue, layout, &mut found_textures);
        Bsp::load_from_pak("pak3.pk3", &textures, &mut materials, device, queue, layout, &mut found_textures);
        Bsp::load_from_pak("pak4.pk3", &textures, &mut materials, device, queue, layout, &mut found_textures);
        Bsp::load_from_pak("pak5.pk3", &textures, &mut materials, device, queue, layout, &mut found_textures);
        Bsp::load_from_pak("pak6.pk3", &textures, &mut materials, device, queue, layout, &mut found_textures);
        Bsp::load_from_pak("pak7.pk3", &textures, &mut materials, device, queue, layout, &mut found_textures);
        Bsp::load_from_pak("pak8.pk3", &textures, &mut materials, device, queue, layout, &mut found_textures);

        for i in 0..found_textures.len() {
            if found_textures[i] == false {
                let tex: String = std::str::from_utf8(&textures[i].name).unwrap().chars().filter(|c| *c != 0 as char).collect();
                //Uncomment for to see failed loads
                //println!("Failed to find texture {}", tex);
            }
        }


        let t_trace = Trace::new();
        Bsp { planes, nodes, leafs, leaf_faces, leaf_brushes, brushes, brush_sides, vertexes_ext, mesh_verts, faces, vertex_buffer, 
            index_buffer, light_maps, light_vols, t_trace, indices_per_texture, materials, textures, materials_light, texture_shader }
    }

    //https://github.com/Francesco149/q3playground/blob/master/main.c
    pub fn trace(&mut self, work: &mut TraceWork, start: cgmath::Vector3<f32>, end: cgmath::Vector3<f32>, mut mins: cgmath::Vector3<f32>, mut maxs: cgmath::Vector3<f32>) {

        work.frac = 1.0;
        work.flags = 0;

        if work.sphere {
            maxs = cgmath::Vector3::new(work.radius, work.radius, work.radius);
            mins = cgmath::Vector3::new(-work.radius, -work.radius, -work.radius);
        }

        for i in 0..3 {

            let offset = (mins[i] + maxs[i]) * 0.5;
            work.mins[i] = mins[i] - offset;
            work.maxs[i] = maxs[i] - offset;
            work.start[i] = start[i] + offset;
            work.end[i] = end[i] + offset;
        }

        if work.sphere {
            work.sphere_offset = cgmath::Vector3::new(0.0, 0.0, work.maxs[2] - work.radius);
        }

        work.offsets[0][0] = work.mins[0];
        work.offsets[0][1] = work.mins[1];
        work.offsets[0][2] = work.mins[2];

        work.offsets[1][0] = work.maxs[0];
        work.offsets[1][1] = work.mins[1];
        work.offsets[1][2] = work.mins[2];

        work.offsets[2][0] = work.mins[0];
        work.offsets[2][1] = work.maxs[1];
        work.offsets[2][2] = work.mins[2];

        work.offsets[3][0] = work.maxs[0];
        work.offsets[3][1] = work.maxs[1];
        work.offsets[3][2] = work.mins[2];

        work.offsets[4][0] = work.mins[0];
        work.offsets[4][1] = work.mins[1];
        work.offsets[4][2] = work.maxs[2];

        work.offsets[5][0] = work.maxs[0];
        work.offsets[5][1] = work.mins[1];
        work.offsets[5][2] = work.maxs[2];

        work.offsets[6][0] = work.mins[0];
        work.offsets[6][1] = work.maxs[1];
        work.offsets[6][2] = work.maxs[2];

        work.offsets[7][0] = work.maxs[0];
        work.offsets[7][1] = work.maxs[1];
        work.offsets[7][2] = work.maxs[2];

        self.trace_node(work, 0, 0.0, 1.0, work.start, work.end);

        if work.frac == 1.0 {
            work.end_pos = end;
        }
        else {
            
            for i in 0..3 {
                work.end_pos[i] = start[i] + work.frac * (end[i] - start[i]);
            }
        }
    }

    fn trace_node(&mut self, work: &mut TraceWork, index: i32, start_frac: f32, end_frac: f32, start: cgmath::Vector3<f32>, end: cgmath::Vector3<f32>) {

        if index < 0 {
            self.trace_leaf(work, (-index) - 1);
            return;
        }

        let mut offset = 0.0;
        let mut start_distance = 0.0;
        let mut end_distance = 0.0;
        let mut side = 0;
        let mut idistance = 0.0;
        let mut frac1: f32 = 0.0;
        let mut frac2: f32 = 0.0;
        let mut mid_frac = 0.0;
        let mut mid = cgmath::Vector3::new(0.0, 0.0, 0.0);

        let node = self.nodes[index as usize];
        let plane = self.planes[node.plane as usize];
        let mut plane_type = PLANE_NONAXIAL;

        if plane.normal[0] == 1.0 || plane.normal[0] == -1.0 {
            plane_type = PLANE_X;
        }
    
        if plane.normal[1] == 1.0 || plane.normal[1] == -1.0 {
            plane_type = PLANE_Y;
        }
    
        if plane.normal[2] == 1.0 || plane.normal[2] == -1.0 {
            plane_type = PLANE_Z;
        }

        if plane_type < 3 {
            start_distance = start[plane_type as usize] - plane.distance;
            end_distance = end[plane_type as usize] - plane.distance;
            offset = work.maxs[plane_type as usize];
        }
        else {
            start_distance = cgmath::dot(start, cgmath::Vector3::new(plane.normal[0], plane.normal[1], plane.normal[2])) - plane.distance;
            end_distance = cgmath::dot(end, cgmath::Vector3::new(plane.normal[0], plane.normal[1], plane.normal[2])) - plane.distance;

            if work.mins == work.maxs {
                offset = 0.0;
            }
            else {
                offset = 2048.0;
            }
        }

        if start_distance < end_distance {
            side = 1;
            idistance = 1.0 / (start_distance - end_distance);
            frac1 = (start_distance - offset + SURF_CLIP_EPSILON) * idistance; 
            frac2 = (start_distance + offset + SURF_CLIP_EPSILON) * idistance;
        }
        else if start_distance > end_distance {
            side = 0;
            idistance = 1.0 / (start_distance - end_distance);
            frac1 = (start_distance + offset + SURF_CLIP_EPSILON) * idistance;
            frac2 = (start_distance - offset - SURF_CLIP_EPSILON) * idistance;
        }
        else {
            side = 0;
            frac1 = 1.0;
            frac2 = 0.0;
        }

        frac1 = 0.0_f32.max(1.0_f32.min(frac1));
        frac2 = 0.0_f32.max(1.0_f32.min(frac2));

        mid_frac = start_frac + (end_frac - start_frac) * frac1;

        for i in 0..3 {
            mid[i] = start[i] + (end[i] - start[i]) * frac1;
        }

        self.trace_node(work, node.children[side as usize], start_frac, mid_frac, start, mid);
    
        mid_frac = start_frac + (end_frac - start_frac) * frac2;

        for i in 0..3 {
            mid[i] = start[i] + (end[i] - start[i]) * frac2;
        }

        self.trace_node(work, node.children[(side^1) as usize], mid_frac, end_frac, mid, end);
    }

    fn trace_leaf(&mut self, work: &mut TraceWork, index: i32) {

        let leaf = self.leafs[index as usize];

        for i in 0..leaf.num_leaf_brushes {

            let brush_index = self.leaf_brushes[(leaf.leaf_brush + i) as usize].brush;
            let brush = self.brushes[brush_index as usize];
            let contents = self.textures[brush.texture as usize].contents;

            if brush.num_brush_sides > 0 && contents & 1 != 0 {

                self.trace_brush(work, brush);

                if work.frac == 0.0 {

                    return;
                }
            }
        }
    }

    fn trace_brush(&mut self, work: &mut TraceWork, brush: Brush) {

        let mut start_frac: f32 = -1.0;
        let mut end_frac: f32 = 1.0;

        let mut closest_plane = Plane { normal: [0.0, 0.0, 0.0], distance: 1.0 };

        for i in 0..brush.num_brush_sides {

            let side_index = brush.brush_side + i;
            let plane_index = self.brush_sides[side_index as usize].plane;
            let plane = self.planes[plane_index as usize];

            let mut frac: f32 = 0.0;

            let mut signbits = 0;

            for i in 0..3 {
                if plane.normal[i] < 0.0 {
                    signbits |= 1<<i;
                }
            }

            let mut dist = plane.distance - cgmath::dot(cgmath::Vector3::new(work.offsets[signbits][0], work.offsets[signbits][1], work.offsets[signbits][2]), cgmath::Vector3::new(plane.normal[0], plane.normal[1], plane.normal[2]));

            let mut startp = work.start.clone();
            let mut endp = work.end.clone();

            if work.sphere {
                dist = plane.distance + work.radius;
                let t = cgmath::dot(cgmath::Vector3::new(plane.normal[0], plane.normal[1], plane.normal[2]), work.sphere_offset);
                if t > 0.0 {
                    startp = work.start - work.sphere_offset;
                    endp = work.end - work.sphere_offset;
                }
                else {
                    startp = work.start + work.sphere_offset;
                    endp = work.end + work.sphere_offset;
                }
            }

            let start_distance = cgmath::dot(startp, cgmath::Vector3::new(plane.normal[0], plane.normal[1], plane.normal[2])) - dist;
            let end_distance = cgmath::dot(endp, cgmath::Vector3::new(plane.normal[0], plane.normal[1], plane.normal[2])) - dist;

            if start_distance > 0.0 {

                work.flags = 1;
            }

            if end_distance > 0.0 {

                if work.flags == 1 {
                    work.flags = 3;
                }
                else {
                    work.flags = 2;
                }
            }

            if start_distance > 0.0 && (end_distance >= SURF_CLIP_EPSILON || end_distance >= start_distance) {
                return;
            }

            if start_distance <= 0.0 && end_distance <= 0.0 {
                continue;
            }

            if start_distance > end_distance {

                frac = (start_distance - SURF_CLIP_EPSILON) / (start_distance - end_distance);

                if frac > start_frac {
                    start_frac = frac;
                    closest_plane = plane;
                }
            }
            else {

                frac = (start_distance + SURF_CLIP_EPSILON) / (start_distance - end_distance);

                end_frac = end_frac.min(frac);
            }
        }

        if start_frac < end_frac && start_frac > -1.0 && start_frac < work.frac {
            work.frac = start_frac.max(0.0_f32);
            work.plane = closest_plane;
        }

        if work.flags == 0 {
            work.frac = 0.0;
        }
    }

    fn trace_patch(&mut self, work: &mut TraceWork) {

    }

    //Patch mesh builder
    fn bezier_curve(t: f32, p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {

        let a = 1.0 - t;
        let tt = t * t;

        let mut t_points: [f32; 3] = [0.0; 3];
        for i in 0..3 {
            t_points[i] = ((a * a) * p0[i]) + (2.0 * a) * (t * p1[i]) + (tt * p2[i]);
        }

        t_points
    }

    fn bezier_curve_uv(t: f32, p0: [f32; 2], p1: [f32; 2], p2: [f32; 2]) -> [f32; 2] {

        let a = 1.0 - t;
        let tt = t * t;

        let mut t_points: [f32; 2] = [0.0; 2];
        for i in 0..2 {
            t_points[i] = ((a * a) * p0[i]) + (2.0 * a) * (t * p1[i]) + (tt * p2[i]);
        }

        t_points
    }

    fn tessellate(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> Vec<[f32; 3]> {

        let mut vects: Vec<[f32; 3]> = Vec::new();
        
        let step_delta = 1.0 / (BEZIER_LEVEL as f32);
        let mut step = step_delta;

        vects.push(p0);
        for i in 0..(BEZIER_LEVEL - 1) {
            vects.push(Bsp::bezier_curve(step, p0, p1, p2));
            step += step_delta;
        }
        vects.push(p2);
        vects
    }

    fn tessellate_uv(p0: [f32; 2], p1: [f32; 2], p2: [f32; 2]) -> Vec<[f32; 2]> {

        let mut uvs: Vec<[f32; 2]> = Vec::new();
        
        let step_delta = 1.0 / (BEZIER_LEVEL as f32);
        let mut step = step_delta;

        uvs.push(p0);
        for i in 0..(BEZIER_LEVEL - 1) {
            uvs.push(Bsp::bezier_curve_uv(step, p0, p1, p2));
            step += step_delta;
        }
        uvs.push(p2);
        uvs
    }

    fn gen_bezier_mesh(control_points: &Vec<Vertex>) -> (Vec<Vertex>, Vec<u32>) {

        let mut vertexes: Vec<Vertex> = Vec::new();
        let mut verts: Vec<Vertex> = Vec::new();
        let mut inds: Vec<u32> = Vec::new();

        let p0s = Bsp::tessellate(control_points[0].position, control_points[3].position, control_points[6].position);
        let p0s_uvs = Bsp::tessellate_uv(control_points[0].texcoord_s, control_points[3].texcoord_s, control_points[6].texcoord_s);
        let p0s_uvl = Bsp::tessellate_uv(control_points[0].texcoord_l, control_points[3].texcoord_l, control_points[6].texcoord_l);
        let p0s_col = Bsp::tessellate([control_points[0].colour[0] as f32, control_points[0].colour[1] as f32, control_points[0].colour[2] as f32], 
                                    [control_points[3].colour[0] as f32, control_points[3].colour[1] as f32, control_points[3].colour[2] as f32], 
                                    [control_points[6].colour[0] as f32, control_points[6].colour[1] as f32, control_points[6].colour[2] as f32]);

        let p1s = Bsp::tessellate(control_points[1].position, control_points[4].position, control_points[7].position);
        let p1s_uvs = Bsp::tessellate_uv(control_points[1].texcoord_s, control_points[4].texcoord_s, control_points[7].texcoord_s);
        let p1s_uvl = Bsp::tessellate_uv(control_points[1].texcoord_l, control_points[4].texcoord_l, control_points[7].texcoord_l);
        let p1s_col = Bsp::tessellate([control_points[1].colour[0] as f32, control_points[1].colour[1] as f32, control_points[1].colour[2] as f32], 
            [control_points[4].colour[0] as f32, control_points[4].colour[1] as f32, control_points[4].colour[2] as f32], 
            [control_points[7].colour[0] as f32, control_points[7].colour[1] as f32, control_points[7].colour[2] as f32]);

        let p2s = Bsp::tessellate(control_points[2].position, control_points[5].position, control_points[8].position);
        let p2s_uvs = Bsp::tessellate_uv(control_points[2].texcoord_s, control_points[5].texcoord_s, control_points[8].texcoord_s);
        let p2s_uvl = Bsp::tessellate_uv(control_points[2].texcoord_l, control_points[5].texcoord_l, control_points[8].texcoord_l);
        let p2s_col = Bsp::tessellate([control_points[4].colour[0] as f32, control_points[4].colour[1] as f32, control_points[4].colour[2] as f32], 
            [control_points[4].colour[0] as f32, control_points[4].colour[1] as f32, control_points[4].colour[2] as f32], 
            [control_points[8].colour[0] as f32, control_points[8].colour[1] as f32, control_points[8].colour[2] as f32]);
        
        for i in 0..(BEZIER_LEVEL+1) {
            let pfs = Bsp::tessellate(p0s[i as usize], p1s[i as usize], p2s[i as usize]);
            let pfs_uvs = Bsp::tessellate_uv(p0s_uvs[i as usize], p1s_uvs[i as usize], p2s_uvs[i as usize]);
            let pfs_uvl = Bsp::tessellate_uv(p0s_uvl[i as usize], p1s_uvl[i as usize], p2s_uvl[i as usize]);
            let pfs_col = Bsp::tessellate(p0s_col[i as usize], p1s_col[i as usize], p2s_col[i as usize]);
            for j in 0..pfs.len() {
                let pfs_col_u: [u8; 4] = [pfs_col[j][0].max(0.0).min(255.0) as u8, pfs_col[j][1].max(0.0).min(255.0) as u8, pfs_col[j][2].max(0.0).min(255.0) as u8, control_points[0].colour[3]];
                vertexes.push(Vertex { position: pfs[j as usize], texcoord_s: pfs_uvs[j as usize], texcoord_l: pfs_uvl[j as usize],
                    normal: control_points[0].normal, colour: pfs_col_u });
            }
        }

        let num_verts = ((BEZIER_LEVEL + 1) * (BEZIER_LEVEL + 1)) as usize;
        let mut x_step = 1;
        let width = (BEZIER_LEVEL + 1) as usize;
        for i in 0..(num_verts - width) {

            if x_step == 1 {
                inds.push(i as u32);
                inds.push((i + width) as u32);
                inds.push((i + 1) as u32);
                x_step += 1;
                continue;
            }
            else if (x_step == width) {
                inds.push(i as u32);
                inds.push((i + (width - 1)) as u32);
                inds.push((i + width) as u32);
                x_step = 1;
                continue;
            }
            else {
                inds.push(i as u32);
                inds.push((i + (width - 1)) as u32);
                inds.push((i + width) as u32);

                inds.push(i as u32);
                inds.push((i + width) as u32);
                inds.push((i + 1) as u32);
                x_step += 1;
                continue;
            }
        }

        (vertexes, inds)
    }

    fn gen_bez_mesh(face: &Face, patch_num: i32, vertexes: &Vec<Vertex>) -> (Vec<Vertex>, Vec<u32>) {

        let num_patches_x = ((face.size[0]) - 1) / 2;
        let num_patches_y = ((face.size[1]) - 1) / 2;
        let mut step_x = 0;
        let mut step_y = 0;
        for i in 0..patch_num {

            step_x += 1;
            if step_x == num_patches_x {

                step_x = 0;
                step_y += 1;
            }
        }

        let mut vert_grid: Vec<Vec<Vertex>> = vec![vec![Vertex { position: [0.0, 0.0, 0.0],
            texcoord_s: [0.0, 0.0],
            texcoord_l: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
            colour: [0u8, 0u8, 0u8, 255u8] }; face.size[1] as usize]; face.size[0] as usize];

        let mut grid_x_step = 0;
        let mut grid_y_step = 0;
        let mut vert_step = face.vertex;
        for i in 0..face.num_vertexes {
            vert_grid[grid_x_step][grid_y_step] = vertexes[vert_step as usize];
            vert_step += 1;
            grid_x_step += 1;
            if grid_x_step as i32 == face.size[0] {
                grid_x_step = 0;
                grid_y_step += 1;
            }
        }
        let vi = (2 * step_x) as usize;
        let vj = (2 * step_y) as usize;

        let mut b_verts: Vec<Vertex> = Vec::new();
        b_verts.push(vert_grid[vi][vj]);
        b_verts.push(vert_grid[vi + 1][vj]);
        b_verts.push(vert_grid[vi + 2][vj]);
        b_verts.push(vert_grid[vi][vj + 1]);
        b_verts.push(vert_grid[vi + 1][vj + 1]);
        b_verts.push(vert_grid[vi + 2][vj + 1]);
        b_verts.push(vert_grid[vi][vj + 2]);
        b_verts.push(vert_grid[vi + 1][vj + 2]);
        b_verts.push(vert_grid[vi + 2][vj + 2]);

        Bsp::gen_patch_collide(&b_verts);
        Bsp::gen_bezier_mesh(&b_verts)
    }

    //TODO
    fn gen_patch_collide(control_points: &Vec<Vertex>) {

        for i in 0..2 {
            for j in 0..2 {

            }
        }

    }

    //Loading from pak
    fn load_from_pak(pak: &str, textures: &Vec<Texture>, materials: &mut Vec<Material>, device: &wgpu::Device, queue: &wgpu::Queue, layout: &wgpu::BindGroupLayout, found_textures: &mut Vec<bool>) {

        let res_dir = std::path::Path::new("res");

        let mut baseq3_pak = "baseq3/".to_string();
        baseq3_pak.push_str(pak);

        let mut f = std::fs::File::open(res_dir.join(baseq3_pak)).unwrap();
        let mut reader = std::io::BufReader::new(f);
        
        let mut zip = zip::ZipArchive::new(reader).unwrap();

        for i in 0..textures.len() {

            if found_textures[i] {
                continue;
            }

            let mut tex: String = std::str::from_utf8(&textures[i].name).unwrap().chars().filter(|c| *c != 0 as char).collect();
            let mut tex_j = tex.clone();
            tex_j.push_str(".jpg");

            let mut check_tga = false;
            let mut use_look_up = false;

            match zip.by_name(&tex_j) {
                Ok(file) => {
                    let tex = texture::Texture::from_bytes_format(device, queue, bytemuck::cast_slice(&(file.bytes().map(|x| x.unwrap()).collect::<Vec<u8>>())), image::ImageFormat::Jpeg, "Tex").unwrap();
                
                    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(&tex.view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(&tex.sampler),
                            },
                        ],
                        label: None,
                    });

                    materials[i] = Material { diffuse_texture: tex, bind_group };
                    found_textures[i] = true;
                    continue;
                },
                Err(e) => {
                    check_tga = true;
                },
            };

            if check_tga {
                let mut tex_t = tex.clone();
                tex_t.push_str(".tga");

                match zip.by_name(&tex_t) {
                    Ok(file) => {
                        let tex = texture::Texture::from_bytes_format(device, queue, bytemuck::cast_slice(&(file.bytes().map(|x| x.unwrap()).collect::<Vec<u8>>())), image::ImageFormat::Tga, "Tex").unwrap();
                    
                        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                            layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&tex.view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(&tex.sampler),
                                },
                            ],
                            label: None,
                        });
    
                        materials[i] = Material { diffuse_texture: tex, bind_group };
                        found_textures[i] = true;
                        continue;
                    },
                    Err(e) => {
                        //println!("Error cant find {}", tex);
                        use_look_up = true;
                    },
                };
            }

            //Look up request
            if use_look_up {
                tex = bsp_look_up::look_up_table(&tex);
                tex_j = tex.clone();
                tex_j.push_str(".jpg");

                check_tga = false;

                match zip.by_name(&tex_j) {
                    Ok(file) => {
                        let tex = texture::Texture::from_bytes_format(device, queue, bytemuck::cast_slice(&(file.bytes().map(|x| x.unwrap()).collect::<Vec<u8>>())), image::ImageFormat::Jpeg, "Tex").unwrap();
                    
                        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                            layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&tex.view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(&tex.sampler),
                                },
                            ],
                            label: None,
                        });

                        materials[i] = Material { diffuse_texture: tex, bind_group };
                        found_textures[i] = true;
                        continue;
                    },
                    Err(e) => {
                        check_tga = true;
                    },
                };

                if check_tga {
                    let mut tex_t = tex.clone();
                    tex_t.push_str(".tga");

                    match zip.by_name(&tex_t) {
                        Ok(file) => {
                            let tex = texture::Texture::from_bytes_format(device, queue, bytemuck::cast_slice(&(file.bytes().map(|x| x.unwrap()).collect::<Vec<u8>>())), image::ImageFormat::Tga, "Tex").unwrap();
                        
                            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                                layout,
                                entries: &[
                                    wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: wgpu::BindingResource::TextureView(&tex.view),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 1,
                                        resource: wgpu::BindingResource::Sampler(&tex.sampler),
                                    },
                                ],
                                label: None,
                            });
        
                            materials[i] = Material { diffuse_texture: tex, bind_group };
                            found_textures[i] = true;
                            continue;
                        },
                        Err(e) => {
                            //println!("Error cant find {}", tex);
                        },
                    };
                }
            }
        }
    }
}

fn parse_data<T>(entry: Entry, bytes: &[u8]) -> Vec<T> where T:
bytemuck::Pod
{
    bytes[(entry.offset as usize)..((entry.offset + entry.size) as usize)].chunks_exact(std::mem::size_of::<T>()).map(|chunk| *bytemuck::from_bytes::<T>(chunk)).collect()
}