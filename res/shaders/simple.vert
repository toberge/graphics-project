#version 130
in vec3 in_position;
out vec3 position;
void main() {
    position = in_position;
    gl_Position = vec4(in_position.xy - 0.5, 0.0, 1.0);
}
