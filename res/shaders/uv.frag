#version 430

in layout(location = 0) vec3 position;
in layout(location = 2) vec2 uv;

uniform float time;
uniform vec2 screen_size;

out vec4 color;

void main() {
    color = vec4(uv, 0., 1.);
}
