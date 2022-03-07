#version 430

in layout(location = 0) vec3 position_in;
in layout(location = 2) vec2 textureCoordinates_in;
out vec3 position;
out vec2 textureCoordinates;

void main() {
    position = position_in;
    textureCoordinates = textureCoordinates_in;

    gl_Position = vec4(position_in.xy - 0.5, 0.0, 1.0);
}
