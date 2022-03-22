#version 430
precision mediump float;

#define FRESNEL_BIAS 0.01
#define FRESNEL_POWER 2.
#define FRESNEL_SCALE 0.05
// #define FRESNEL_SCALE 0.05 seems better

#define MAX_LIGHT_SOURCES 32

struct LightSource {
    vec3 position;
    vec3 color;
};

in layout(location = 0) vec3 position;
in layout(location = 1) vec3 normal_in;
in layout(location = 2) vec2 uv;
in layout(location = 3) vec4 color_in;

uniform int use_texture;
uniform int use_reflection;

uniform float shininess;
uniform vec3 camera_position;

uniform uint num_light_sources;
uniform LightSource light_sources[MAX_LIGHT_SOURCES];

uniform layout(binding = 0) sampler2D sampler;
uniform layout(binding = 1) sampler2D reflection_sampler;

out vec4 color;

void main() {
    vec3 normal = normalize(normal_in);
    vec3 diffuse_reflection;

    vec3 camDir = normalize(camera_position - position);
    if (use_texture == 1) {
        diffuse_reflection = texture(sampler, uv).rgb;
    } else if (use_reflection == 1) {
        //diffuse_reflection = texture(reflection_sampler, reflect(-camDir, normal)).rgb;
        diffuse_reflection = texture(reflection_sampler, uv).rgb;
    } else {
        diffuse_reflection = color_in.rgb;
    }

    vec3 lighting = vec3(0);

    for (int i = 0; i < num_light_sources; i++) {
        vec3 light = light_sources[i].position;
        vec3 lightColor = light_sources[i].color;
        float S = 1;

        // Attenuation (reduces reach of lightsource)
        float la = 0.01;
        float lb = 0.03;
        float lc = 0.02;
        float d = length(light - position);
        float L = 1 / (la + d*lb + d*d*lc);
        L = 1;

        // Phong model
        // Parameters – note that specular reflection is independent of surface color!
        vec3 specular_reflection = vec3(1.0, 1.0, 1.0);
        // Vectors
        vec3 lightDir = normalize(light - position);
        vec3 reflection = reflect(-lightDir, normal);

        // Calculation
        float diffuse_factor = S * L * max(0, dot(lightDir, normal));
        float specular_factor = S * L * pow(max(0, dot(reflection, camDir)), shininess);
        //lighting += diffuse_factor * lightSource.color * diffuse_reflection + specular_factor * lightSource.color * specular_reflection;
        lighting += diffuse_factor * diffuse_reflection * lightColor + specular_factor * specular_reflection * lightColor;

    }

    if (use_reflection == 1) {

        float fresnel_factor = FRESNEL_BIAS + FRESNEL_SCALE * pow(1. + dot(camDir, normal), FRESNEL_POWER);
        color = vec4(mix(lighting, diffuse_reflection, fresnel_factor), 1.);
    } else {
        color = vec4(lighting, 1.);
    }
}
