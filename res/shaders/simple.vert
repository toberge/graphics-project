#version 430

in layout(location = 0) vec3 position_in;
in layout(location = 2) vec2 textureCoordinates_in;

uniform mat4 view_transform;
uniform mat4 model_transform;

out vec3 position;
out vec2 textureCoordinates;

void main() {
    textureCoordinates = textureCoordinates_in;

    position = vec3(model_transform * vec4(position_in, 1.));
    gl_Position = view_transform * vec4(position_in, 1.0);
}
