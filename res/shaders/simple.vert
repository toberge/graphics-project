#version 430
precision highp float;

in layout(location = 0) vec3 position_in;
in layout(location = 1) vec3 normal_in;
in layout(location = 2) vec2 textureCoordinates_in;
in layout(location = 3) vec4 color_in;

uniform mat4 view_transform;
uniform mat4 model_transform;
uniform mat3 normal_transform;

out layout(location = 0) vec3 position;
out layout(location = 1) vec3 normal;
out layout(location = 2) vec2 textureCoordinates;
out layout(location = 3) vec4 color;

void main() {
    textureCoordinates = textureCoordinates_in;
    color = color_in;

    normal = normalize(normal_transform * normal_in);
    position = vec3(model_transform * vec4(position_in, 1.));
    gl_Position = view_transform * vec4(position_in, 1.0);
}
