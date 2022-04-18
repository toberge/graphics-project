#version 430
precision mediump float;

in layout(location = 0) vec3 position;
in layout(location = 2) vec2 uv;

uniform vec3 camera_position;

uniform int mode;

#define STANDARD_MODE 0

uniform layout(binding = 0) sampler2D color_sampler;
uniform layout(binding = 1) sampler2D depth_sampler;
uniform layout(binding = 2) sampler2D crt_sampler;
uniform layout(binding = 3) sampler2D crt_depth_sampler;

out vec4 color;

void main() {
    vec3 pre_color = texture(color_sampler, uv).rgb;
    float depth = texture(depth_sampler, uv).r;
    vec3 crt = texture(crt_sampler, uv).rgb;
    float crt_depth = texture(crt_depth_sampler, uv).r;
    // Perform manual depth test to see if crt should contribute to color
    if (crt_depth <= depth && mode == STANDARD_MODE) {
        pre_color = pre_color + 0.5 * crt;
    }
    color = vec4(pre_color, 1.);
}
