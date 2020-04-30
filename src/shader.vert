#version 450

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 overlay;
layout(location = 3) in int overlay_only;

layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec4 out_overlay;
layout(location = 2) out int out_overlay_only;


void main() {
    gl_Position = vec4(pos, 0.0, 1.0);

    out_overlay = overlay;
    out_uv = uv;
    out_overlay_only = overlay_only;
}
