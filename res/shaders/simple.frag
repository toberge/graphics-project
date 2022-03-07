#version 430
precision mediump float;

in layout(location = 0) vec3 position;
in layout(location = 1) vec3 normal_in;
in layout(location = 2) vec2 textureCoordinates;
in layout(location = 3) vec4 color_in;

uniform int use_texture;

uniform float shininess;
uniform vec3 camera_position;

uniform layout(binding = 0) sampler2D sampler;

out vec4 color;

void main() {
    vec3 normal = normalize(normal_in);
    vec3 diffuse_reflection;
    if (use_texture == 1) {
        diffuse_reflection = texture(sampler, textureCoordinates).rgb;
    } else {
        diffuse_reflection = color_in.rgb;
        //diffuse_reflection = vec3(1., 1., 1.);
    }

    vec3 lighting = vec3(0);

    for (int i = 0; i < 1; i++) {
        //vec3 light = lightSource.position;
        vec3 light = vec3(1., 2., 0.);
        float S = 1.;

        // Attenuation (reduces reach of lightsource)
        float la = 0.001;
        float lb = 0.003;
        float lc = 0.002;
        float d = length(light - position);
        float L = 1 / (la + d*lb + d*d*lc);
        L *= 5;
        L = 1; // TODO remove

        // Phong model
        // Parameters – note that specular reflection is independent of surface color!
        vec3 specular_reflection = vec3(1.0, 1.0, 1.0);
        // Vectors
        vec3 lightDir = normalize(light - position);
        vec3 camDir = normalize(camera_position - position);
        vec3 reflection = reflect(-lightDir, normal);

        // Calculation
        float diffuse_factor = S * L * max(0, dot(lightDir, normal));
        float specular_factor = S * L * pow(max(0, dot(reflection, camDir)), shininess);
        //lighting += diffuse_factor * lightSource.color * diffuse_reflection + specular_factor * lightSource.color * specular_reflection;
        lighting += diffuse_factor * diffuse_reflection + specular_factor * specular_reflection;

    }

    //color = vec4((normal + 1.) / 2., 1.);
    //color = vec4((camera_position + 1.) / 2., 1.);
    color = vec4(lighting, 1.);
    //color = texture(sampler, textureCoordinates);
}
