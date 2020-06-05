#version 450

layout(location = 0) in vec2 v_point;

layout(location = 1) in vec2 i_position;
layout(location = 2) in vec2 i_dimensions;
layout(location = 3) in float i_rotation;
layout(location = 4) in vec2 i_uv_top_left;
layout(location = 5) in vec2 i_uv_dimensions;
layout(location = 6) in vec4 i_overlay;
layout(location = 7) in int i_overlay_only;

layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec4 out_overlay;
layout(location = 2) out int out_overlay_only;

layout(set = 0, binding = 2) uniform Uniforms {
    vec2 window_size;
    vec2 virtual_size;
};

void main() {
    float scale_factor = min(window_size.x / virtual_size.x, window_size.y / virtual_size.y);
    vec2 centering_offset = window_size - (virtual_size * scale_factor);

    vec2 pos = i_position * 2.0 * scale_factor;
    pos -= window_size;
    pos += centering_offset;

    vec2 position = pos * vec2(1.0, -1.0);
    vec2 dimensions = i_dimensions * vec2(1.0, -1.0) * scale_factor;

    mat2 rotation = mat2(
        cos(i_rotation), -sin(i_rotation),
        sin(i_rotation),  cos(i_rotation)
    );

    vec2 screen_space_pos = position + (rotation * (dimensions * v_point));

    gl_Position = vec4(screen_space_pos / window_size, 0.0, 1.0);

    vec2 uv_offset = i_uv_top_left + i_uv_dimensions * 0.5;

    out_overlay = i_overlay;
    out_uv = uv_offset + i_uv_dimensions * v_point * 0.5;
    out_overlay_only = i_overlay_only;
}
