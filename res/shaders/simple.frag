#version 130
precision mediump float;
in vec3 position;
out vec4 color;
uniform float blue;
void main() {
    color = vec4(position.xy, blue, 1.0);
}
