#version 450

layout(location = 0) in vec2 v_tex_coords;
layout(location = 1) in vec3 v_normal;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D t_diffuse;
layout(set = 0, binding = 1) uniform sampler s_diffuse;

void main() {
    vec3 light_pos = vec3(0.0, 0.0, 20.0);
    vec3 light_dir = normalize(light_pos - vec3(0.0, 0.0, 0.0));
    float diff = max(dot(light_dir, v_normal), 0.1);
    f_color = texture(sampler2D(t_diffuse, s_diffuse), v_tex_coords) * vec4(diff, diff, diff, 1.0);
}