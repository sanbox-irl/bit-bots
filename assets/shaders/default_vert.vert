#version 450

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_vert_uv;

layout(location = 0) out vec2 out_frag_uv;
layout(location = 1) out vec4 out_frag_color;

layout(push_constant) uniform PushConstants {
    vec2 camera_position;
    vec2 native_resolution;
    vec2 entity_position;
    vec2 image_size;
    vec2 norm_image_coordinate;
    vec2 norm_image_size;
    vec4 color;
}
pc;

void main() {
    // MODEL POSITION
    vec2 model_position = vec2(2.0 / pc.native_resolution.x, -2.0 / pc.native_resolution.y);

    model_position *= in_position.xy * pc.image_size + pc.entity_position - pc.camera_position;
    gl_Position = vec4(model_position, in_position.z, 1.0);

    // OUT
    out_frag_uv = pc.norm_image_coordinate + in_vert_uv * pc.norm_image_size;
    out_frag_color = pc.color;
}