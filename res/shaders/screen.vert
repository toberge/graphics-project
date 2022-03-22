#version 430

in layout(location = 0) vec3 in_position;
out layout(location = 0) vec3 position;

void main() {
    position = in_position;
    gl_Position = vec4(in_position, 1.0);
}
