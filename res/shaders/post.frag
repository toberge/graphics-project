#version 430
precision mediump float;

in layout(location = 0) vec3 position;
in layout(location = 2) vec2 uv;

uniform vec3 camera_position;

uniform layout(binding = 0) sampler2D color_sampler;
uniform layout(binding = 1) sampler2D depth_sampler;
uniform layout(binding = 2) sampler2D crt_sampler;
uniform layout(binding = 3) sampler2D crt_depth_sampler;

out vec4 color;

// TODO actual noise or sth
float fog(vec3 p) {
    return abs((.5 + abs(p.x) + abs(p.y)) - p.z)/40.;
}

float raymarch_fog(float depth) {
    float step = .1;
    float intensity = 0.;
    vec3 p = vec3(position.xy, 0.);
    float d = 0.;
    while (d < depth) {
        intensity += fog(p);
        d += step;
        p.z = d;
    }
    return intensity;
}

void main() {
    vec3 pre_color = texture(color_sampler, uv).rgb;
    float depth = texture(depth_sampler, uv).r;
    vec3 crt = texture(crt_sampler, uv).rgb;
    float crt_depth = texture(crt_depth_sampler, uv).r;
    // Perform manual depth test to see if crt should contribute to color
    if (crt_depth <= depth) {
        pre_color = pre_color + 0.4 * crt;
    }
    // Extra effects
    vec3 fog_color = vec3(.7);
    float intensity = raymarch_fog(depth);
    intensity = 0.; // temporary make it disappear TODO undo when you want to
    color = vec4(mix(pre_color, fog_color, intensity), 1.);
}
