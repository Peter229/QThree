pub fn look_up_table(val: &str) -> String {
    match val {
        "textures/skies/pj_dm9sky" => "textures/skies/bluedimclouds".to_string(), //q3dm11
        "textures/base_trim/border12b_pj" => "textures/base_trim/border12b".to_string(), //border12bfx
        "textures/base_wall/glass01" => "textures/effects/tinfx".to_string(),
        "textures/base_button/shootme2" => "textures/base_support/metal3_3".to_string(),
        "textures/base_support/support2rust" => "textures/base_support/support1rust".to_string(),
        "textures/gothic_light/gothic_light3_2K" => "textures/gothic_light/gothic_light3".to_string(), //gothic_light2_blend
        "textures/gothic_light/gothic_light2_2K" => "textures/gothic_light/gothic_light2".to_string(),
        "textures/gothic_light/gothic_light3_3k" => "textures/gothic_light/gothic_light3".to_string(),
        "textures/gothic_light/gothic_light2_lrg_2k" => "textures/gothic_light/gothic_light2_lrg".to_string(),
        "textures/gothic_light/goth_lt2_lrg2k" => "textures/gothic_light/gothic_light2_lrg".to_string(),
        "textures/base_light/proto_light_2k" => "textures/base_light/proto_light".to_string(),
        "textures/base_light/baslt4_1_2k" => "textures/base_light/baslt4_1".to_string(),
        "textures/base_light/patch10_pj_lite2_1000" => "textures/base_light/patch10_pj_lite2".to_string(),
        "textures/sfx/flameanim_green_pj" => "textures/sfx/g_flame1".to_string(),
        "textures/sfx/q3dm9fog" => "textures/liquids/kc_fogcloud3".to_string(),
        "textures/sfx/diamond2cjumppad" => "textures/sfx/bouncepad01b_layer1".to_string(),
        "textures/sfx/teslacoil3" => "textures/sfx/cabletest2".to_string(),
        "textures/liquids/slime1" => "textures/liquids/slime7".to_string(),
        //"textures/skies/pj_ctf2_sky" => "textures/skies/bluedimclouds".to_string(), //q3ctf2 Start //scroll 0.015, 0.016, scale 3 3
        _ => val.to_string(),
    }
}

/*
//Player Clipping
    pub fn trace_ray(&mut self, start: cgmath::Vector3<f32>, end: cgmath::Vector3<f32>) {

        self.t_trace = Trace { output_fraction: 1.0, output_end: cgmath::Vector3::new(0.0, 0.0, 0.0), output_starts_out: true, output_all_solid: false, start, 
            end, radius: 1.0, mins: cgmath::Vector3::new(0.0, 0.0, 0.0), maxs: cgmath::Vector3::new(2.0, 2.0, 2.0), extents: cgmath::Vector3::new(1.0, 1.0, 1.0),
            t_type: RAY };

        self.trace();
    }

    pub fn trace_sphere(&mut self, start: cgmath::Vector3<f32>, end: cgmath::Vector3<f32>, radius: f32) {

        self.t_trace = Trace { output_fraction: 1.0, output_end: cgmath::Vector3::new(0.0, 0.0, 0.0), output_starts_out: true, output_all_solid: false, start, 
            end, radius, mins: cgmath::Vector3::new(0.0, 0.0, 0.0), maxs: cgmath::Vector3::new(2.0, 2.0, 2.0), extents: cgmath::Vector3::new(1.0, 1.0, 1.0),
            t_type: SPHERE };

        self.trace();
    }

    pub fn trace_box(&mut self, start: cgmath::Vector3<f32>, end: cgmath::Vector3<f32>, mins: cgmath::Vector3<f32>, maxs: cgmath::Vector3<f32>) {

        let mut extents = cgmath::Vector3::new(0.0, 0.0, 0.0);

        if mins[0] == 0.0 && mins[1] == 0.0 && mins[2] == 0.0 &&
            maxs[0] == 0.0 && maxs[1] == 0.0 && maxs[2] == 0.0 {
                self.trace_ray(start, end);
        }
        else {
            if -mins[0] > maxs[0] {
                extents[0] = -mins[0];
            }
            else {
                extents[0] = -maxs[0];
            }
            
            if -mins[1] > maxs[1] {
                extents[1] = -mins[1];
            }
            else {
                extents[1] = -maxs[1];
            }

            if -mins[2] > maxs[2] {
                extents[2] = -mins[2];
            }
            else {
                extents[2] = -maxs[2];
            }
        }

        self.t_trace = Trace { output_fraction: 1.0, output_end: cgmath::Vector3::new(0.0, 0.0, 0.0), output_starts_out: true, output_all_solid: false, start, 
            end, radius: 1.0, mins, maxs, extents,
            t_type: BOX };

        self.trace();
    }

fn trace(&mut self) {

        //self.trace = Trace { output_fraction: 1.0, output_end: cgmath::Vector3::new(0.0, 0.0, 0.0), output_starts_out: true, output_all_solid: false, start, end };
        let output_starts_out = true;
        let output_all_solid = false;
        let output_fraction = 1.0;

        self.check_node(0, 0.0, 1.0, self.t_trace.start, self.t_trace.end);

        if self.t_trace.output_fraction == 1.0 {
            self.t_trace.output_end = self.t_trace.end;
        }
        else {
            println!("COLLISION");
            for i in 0..3 {
                self.t_trace.output_end[i] = self.t_trace.start[i] + self.t_trace.output_fraction * (self.t_trace.end[i] - self.t_trace.start[i]);
            }
        }
    }

    fn check_node(&mut self, node_index: i32, start_fraction: f32, end_fraction: f32, start: cgmath::Vector3<f32>, end: cgmath::Vector3<f32>) {

        if node_index < 0 {
            let leaf = self.leafs[(-(node_index + 1)) as usize];
            for i in 0..leaf.num_leaf_brushes {
                let brush = self.brushes[self.leaf_brushes[(leaf.leaf_brush + i) as usize].brush as usize];
                //println!("{}", (self.textures[brush.texture as usize].flags) & 1);
                if brush.num_brush_sides > 0 && (self.textures[brush.texture as usize].contents & 1) == 1 {
                    self.check_brush(brush);
                }
            }

            return;
        }

        let node = self.nodes[node_index as usize];
        let plane = self.planes[node.plane as usize];

        let start_distance = cgmath::dot(start, cgmath::Vector3::new(plane.normal[0], plane.normal[1], plane.normal[2])) - plane.distance;
        let end_distance = cgmath::dot(end, cgmath::Vector3::new(plane.normal[0], plane.normal[1], plane.normal[2])) - plane.distance;
    
        let mut offset = 0.0;

        if self.t_trace.t_type == RAY {
            offset = 0.0;
        }
        else if self.t_trace.t_type == SPHERE {
            offset = self.t_trace.radius;
        }
        else if self.t_trace.t_type == BOX {
            offset = (self.t_trace.extents[0] * plane.normal[0]).abs() +
                    (self.t_trace.extents[1] * plane.normal[1]).abs() +
                    (self.t_trace.extents[2] * plane.normal[2]).abs();
        }

        if start_distance >= offset && end_distance >= offset {
            self.check_node(node.children[0], start_fraction, end_fraction, start, end);
        }
        else if start_distance < -offset && end_distance < -offset {
            self.check_node(node.children[1], start_fraction, end_fraction, start, end);
        }
        else {
            let mut side: i32 = 0;
            let mut fraction_1: f32 = 0.0;
            let mut fraction_2: f32 = 0.0;
            let mut middle_fraction: f32 = 0.0;
            let mut middle: cgmath::Vector3<f32> = cgmath::Vector3::new(0.0, 0.0, 0.0);

            if start_distance < end_distance {
                side = 1;
                let inverse_distance = 1.0 / (start_distance - end_distance);
                fraction_1 = (start_distance - offset + EPSILON) * inverse_distance;
                fraction_2 = (start_distance + offset + EPSILON) * inverse_distance;
            }
            else if end_distance < start_distance {
                side = 0;
                let inverse_distance = 1.0 / (start_distance - end_distance);
                fraction_1 = (start_distance + offset + EPSILON) * inverse_distance;
                fraction_2 = (start_distance - offset - EPSILON) * inverse_distance;
            }
            else {
                side = 0;
                fraction_1 = 1.0;
                fraction_2 = 0.0;
            }

            if fraction_1 < 0.0 {
                fraction_1 = 0.0;
            }
            else if fraction_1 > 1.0 {
                fraction_1 = 1.0;
            }
            if fraction_2 < 0.0 {
                fraction_2 = 0.0;
            }
            else if fraction_2 > 1.0 {
                fraction_2 = 1.0;
            }

            middle_fraction = start_fraction + (end_fraction - start_fraction) * fraction_1;

            for i in 0..3 {
                middle[i] = start[i] + fraction_1 * (end[i] - start[i]);
            }

            self.check_node(node.children[side as usize].clone(), start_fraction, middle_fraction, start, middle);

            middle_fraction = start_fraction + (end_fraction - start_fraction) * fraction_2;

            for i in 0..3 {
                middle[i] = start[i] + fraction_2 * (end[i] - start[i]);
            }

            self.check_node(node.children[side as usize].clone(), middle_fraction, end_fraction, middle, end);
        }
    }

    fn check_brush(&mut self, brush: Brush) {

        let mut start_fraction = -1.0;
        let mut end_fraction = 1.0;
        let mut starts_out = false;
        let mut ends_out = false;

        for i in 0..brush.num_brush_sides {

            let brush_side = self.brush_sides[(brush.brush_side + i) as usize];
            let plane = self.planes[brush_side.plane as usize];

            let mut start_distance = 0.0;
            let mut end_distance = 0.0;

            if self.t_trace.t_type == RAY {
                start_distance = cgmath::dot(self.t_trace.start, cgmath::Vector3::new(plane.normal[0], plane.normal[1], plane.normal[2])) - plane.distance;
                end_distance = cgmath::dot(self.t_trace.end, cgmath::Vector3::new(plane.normal[0], plane.normal[1], plane.normal[2])) - plane.distance;
            }
            else if self.t_trace.t_type == SPHERE {
                start_distance = cgmath::dot(self.t_trace.start, cgmath::Vector3::new(plane.normal[0], plane.normal[1], plane.normal[2])) - (plane.distance + self.t_trace.radius);
                end_distance = cgmath::dot(self.t_trace.end, cgmath::Vector3::new(plane.normal[0], plane.normal[1], plane.normal[2])) - (plane.distance + self.t_trace.radius);
            }
            else if self.t_trace.t_type == BOX {

                let mut offset = cgmath::Vector3::new(0.0, 0.0, 0.0);
                for j in 0..3 {
                    if plane.normal[j] < 0.0 {
                        offset[j] = self.t_trace.maxs[j];
                    }
                    else {
                        offset[j] = self.t_trace.mins[j];
                    }

                    start_distance = (self.t_trace.start[0] + offset[0]) * plane.normal[0] +
                                    (self.t_trace.start[1] + offset[1]) * plane.normal[1] +
                                    (self.t_trace.start[2] + offset[2]) * plane.normal[2] -
                                    plane.distance;
                    
                    end_distance = (self.t_trace.end[0] + offset[0]) * plane.normal[0] +
                                    (self.t_trace.end[1] + offset[1]) * plane.normal[1] +
                                    (self.t_trace.end[2] + offset[2]) * plane.normal[2] -
                                    plane.distance;
                }
            }

            if start_distance > 0.0 {
                starts_out = true;
            }
            if end_distance > 0.0 {
                ends_out = true;
            }

            if start_distance > 0.0 && end_distance > 0.0 {
                return;
            }
            if start_distance <= 0.0 && end_distance <= 0.0 {
                continue;
            }

            if start_distance > end_distance {
                let fraction = (start_distance - EPSILON) / (start_distance - end_distance);
                if fraction > start_fraction {
                    start_fraction = fraction;
                }
            }
            else {
                let fraction = (start_distance + EPSILON) / (start_distance - end_distance);
                if fraction < end_fraction {
                    end_fraction = fraction;
                }
            }
        }

        if starts_out == false {
            self.t_trace.output_starts_out = false;
            if ends_out == false {
                self.t_trace.output_all_solid = true;
            }
            return;
        }

        if start_fraction < end_fraction {
            if start_fraction > -1.0 && start_fraction < self.t_trace.output_fraction {
                if start_fraction < 0.0 {
                    start_fraction = 0.0;
                }
                self.t_trace.output_fraction = start_fraction;
            }
        }
    }
    */