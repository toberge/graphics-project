#version 430

in layout(location = 0) vec3 position;

uniform float time;

out vec4 color;

void main() {
    // Transform into range [0, 1]
    // TODO Add uniform with texture size...
    vec2 uv = vec2(gl_FragCoord.x / 200., gl_FragCoord.y / 200.);
    // Then into range [-1, 1]
    vec2 xy = (uv - .5) * 2.;

    color = vec4(uv, 0., 1.);
}
