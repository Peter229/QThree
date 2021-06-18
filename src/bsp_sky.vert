#version 450

layout(location = 0) in vec3 a_position;
layout(location = 1) in vec2 a_tex_coords;
layout(location = 2) in vec3 a_tex_coords_lightmap;
layout(location = 3) in vec3 a_normal;
layout(location = 4) in vec4 a_colour;

layout(location = 0) out vec4 v_colour;
layout(location = 1) out vec2 v_tex_coords;

layout(set = 1, binding = 0)
uniform Uniforms {
    mat4 proj;
    mat4 view;
    mat4 model;
    float time;
};

void main() {
    v_colour = a_colour;
    vec2 v_tex_coord = a_tex_coords;
    v_tex_coord = v_tex_coord + vec2(0.015 * time, 0.016 * time);
    v_tex_coords = v_tex_coord * vec2(1.0 / 3.0, 1.0 / 3.0);
    gl_Position = proj * view * model * vec4(a_position, 1.0);
}