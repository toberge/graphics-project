#version 430

in layout(location = 0) vec3 position;

uniform float time;
uniform vec2 screen_size;

out vec4 color;

void main() {
    // Transform into range [0, 1]
    vec2 uv = gl_FragCoord.xy / screen_size;
    // Then into range [-1, 1]
    vec2 xy = (uv - .5) * 2.;

    color = vec4(uv, 0., 1.);
}
