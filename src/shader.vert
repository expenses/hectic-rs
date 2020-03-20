#version 450

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 out_uv;

void main() {
    gl_Position = vec4(pos, 0.0, 1.0);
    out_uv = uv;
}
