#version 430

in layout(location = 0) vec3 in_position;
in layout(location = 2) vec2 in_uv;

out layout(location = 0) vec3 position;
out layout(location = 2) vec2 uv;

void main() {
    position = in_position;
    uv = in_uv;
    gl_Position = vec4(in_position, 1.0);
}

