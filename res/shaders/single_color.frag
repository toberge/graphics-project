#version 430

in vec3 position;
out vec4 color;

void main() {
    color = vec4((position.xy + 1.) / 2., 0., 1.);
}
