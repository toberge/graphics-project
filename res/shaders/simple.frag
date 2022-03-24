#version 430
precision mediump float;

#define FRESNEL_BIAS 0.01
#define FRESNEL_POWER 2.
#define FRESNEL_SCALE 0.50

#define EMMISSIVE_FACTOR 0.3

#define MAX_LIGHT_SOURCES 32

struct LightSource {
    vec3 position;
    vec3 color;
};

in layout(location = 0) vec3 position;
in layout(location = 1) vec3 normal_in;
in layout(location = 2) vec2 uv;
in layout(location = 3) vec4 color_in;
in layout(location = 4) mat3 TBN;

uniform int use_texture;
uniform int use_reflection;
uniform int use_normals;
uniform int use_roughness;
uniform int use_opacity;

uniform float shininess;
uniform vec3 camera_position;

uniform uint num_light_sources;
uniform LightSource light_sources[MAX_LIGHT_SOURCES];

uniform layout(binding = 0) sampler2D texture_sampler;
uniform layout(binding = 1) sampler2D reflection_sampler;
uniform layout(binding = 2) sampler2D normal_sampler;
uniform layout(binding = 3) sampler2D roughness_sampler;
uniform layout(binding = 4) sampler2D opacity_sampler;

out vec4 color;

void main() {
    vec3 cam_dir = normalize(camera_position - position);

    vec3 diffuse_reflection;
    if (use_texture == 1) {
        diffuse_reflection = texture(texture_sampler, uv).rgb;
    } else {
        diffuse_reflection = color_in.rgb;
    }

    vec3 normal = normalize(normal_in);
    if (use_normals == 1) {
        normal = normalize(TBN * (2*vec3(texture(normal_sampler, uv)) - 1));
    }

    float shininess = 32;
    if (use_roughness == 1) {
        float roughness = texture(roughness_sampler, uv).r;
        shininess = 5. / (roughness * roughness);
    }

    float opacity = 1.;
    if (use_opacity == 1) {
        opacity = texture(opacity_sampler, uv).r;
    }

    vec3 reflection = vec3(0);
    if (use_reflection == 1) {
        //diffuse_reflection = texture(reflection_sampler, reflect(-cam_dir, normal)).rgb;
        reflection = texture(reflection_sampler, uv).rgb;
    }

    vec3 lighting = vec3(0);

    for (int i = 0; i < num_light_sources; i++) {
        vec3 light = light_sources[i].position;
        vec3 light_color = light_sources[i].color;
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
        vec3 light_dir = normalize(light - position);
        vec3 reflection = reflect(-light_dir, normal);

        // Calculation
        float diffuse_factor = S * L * max(0, dot(light_dir, normal));
        float specular_factor = S * L * pow(max(0, dot(reflection, cam_dir)), shininess);
        lighting += diffuse_factor * diffuse_reflection * light_color + specular_factor * specular_reflection * light_color;

    }

    if (use_reflection == 1 && use_texture == 1) {
        // This is noe of the CRT screens, boost the light from its contents
        lighting += diffuse_reflection * EMMISSIVE_FACTOR;
    }

    if (use_reflection == 1) {
        float fresnel_factor = max(0, min(1,FRESNEL_BIAS + FRESNEL_SCALE * pow(1. + dot(-cam_dir, normal), FRESNEL_POWER)));
        color = vec4(mix(lighting, reflection, fresnel_factor), opacity);
    } else {
        color = vec4(lighting, opacity);
    }
}
