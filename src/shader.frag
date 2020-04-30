#version 450

layout(location = 0) out vec4 outColor;
layout(location = 0) in vec2 out_uv;
layout(location = 1) in vec4 out_overlay;
layout(location = 2) in flat int out_overlay_only;

layout(set = 0, binding = 0) uniform texture2D tex;
layout(set = 0, binding = 1) uniform sampler samp;

void main() {
    if (out_overlay_only == 1) {
        outColor = out_overlay;
        return;
    }

    vec4 texture_colour = texture(sampler2D(tex, samp), out_uv);

    if (texture_colour.a == 0.0) {
        discard;
    }

    outColor = vec4(mix(texture_colour.rgb, out_overlay.rgb, out_overlay.a), texture_colour.a);
}
