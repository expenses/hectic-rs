#version 450

layout(location = 0) out vec4 outColor;
layout(location = 0) in vec2 out_uv;

layout(set = 0, binding = 0) uniform texture2D tex;
layout(set = 0, binding = 1) uniform sampler samp;

void main() {
    outColor = texture(sampler2D(tex, samp), out_uv);
}
