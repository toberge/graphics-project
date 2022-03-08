#version 430

in layout(location = 0) vec3 position;

uniform float time;

out vec4 color;

void main() {
    vec2 uv = vec2(gl_FragCoord.x / 102.4, gl_FragCoord.y / 76.9);
    color = vec4(uv, 0., 1.);
}
