#version 450

layout(location = 0) in vec3 a_position;
layout(location = 1) in vec3 a_colour;

layout(location = 0) out vec3 v_colour;

layout(set = 0, binding = 0)
uniform Uniforms {
    mat4 proj;
    mat4 view;
    mat4 model;
    float time;
};

void main() {
    v_colour = a_colour;
    gl_Position = proj * view * model * vec4(a_position, 1.0);
}