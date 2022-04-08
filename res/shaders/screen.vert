#version 430

in layout(location = 0) vec3 position_in;
in layout(location = 2) vec2 uv_in;
out layout(location = 2) vec2 uv;

uniform mat4 view_transform;

void main() {
    uv = uv_in;
    gl_Position = view_transform * vec4(position_in, 1.0);
}
