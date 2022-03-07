#version 430
precision mediump float;
in vec3 position;
out vec4 color;
uniform float blue;

uniform layout(binding = 0) sampler2D sampler;

void main() {
    //color = vec4(position.xy, blue, 1.0);
    color = texture(sampler, vec2(0));
}
