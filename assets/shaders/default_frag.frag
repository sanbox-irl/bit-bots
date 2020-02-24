#version 450
// UNIFORMS
layout(set = 0, binding = 0) uniform texture2D u_tex;
layout(set = 0, binding = 1) uniform sampler u_samp;

// IN
layout(location = 0) in vec2 in_frag_uv;
layout(location = 1) in vec4 in_frag_color;

// OUT
layout(location = 0) out vec4 out_color;

void main() {
    out_color = in_frag_color * texture(sampler2D(u_tex, u_samp), in_frag_uv);
}