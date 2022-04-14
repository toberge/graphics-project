#version 430
precision mediump float;

#define FRESNEL_BIAS 0.02
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
in layout(location = 7) mat3 cubemap_transform;

uniform int use_texture;
uniform int use_reflection;
uniform int use_cubemaps;
uniform int use_normals;
uniform int use_roughness;
uniform int use_opacity;

uniform int mode;

#define STANDARD_MODE 0
#define REFLECTION_MODE 1
#define NORMALS_MODE 2
#define REFLECTION_VECTORS_MODE 3

uniform float shininess;
uniform vec3 camera_position;

uniform uint num_light_sources;
uniform LightSource light_sources[MAX_LIGHT_SOURCES];

uniform layout(binding = 0) sampler2D texture_sampler;
uniform layout(binding = 1) sampler2D reflection_sampler;
uniform layout(binding = 2) sampler2D normal_sampler;
uniform layout(binding = 3) sampler2D roughness_sampler;
uniform layout(binding = 4) sampler2D opacity_sampler;
uniform layout(binding = 5) samplerCube cubemap_sampler;

out vec4 color;

void main() {
    vec3 cam_dir = normalize(camera_position - position);
    // Since image files are in opposite order of OpenGL's uvs,
    // use flipped uvs for textures loaded from image files.
    // Avoiding flips halves load time.
    vec2 flipped_uv = vec2(uv.x, 2. - uv.y);

    vec3 diffuse_reflection;
    if (use_texture == 1) {
        if (use_reflection == 1) {
            diffuse_reflection = texture(texture_sampler, uv).rgb;
        } else {
            // not camera, use flipped coords
            diffuse_reflection = texture(texture_sampler, flipped_uv).rgb;
        }
    } else {
        diffuse_reflection = color_in.rgb;
    }

    vec3 normal = normalize(normal_in);
    if (use_normals == 1) {
        normal = normalize(TBN * (2*vec3(texture(normal_sampler, flipped_uv)) - 1));
    }

    float shininess = 32;
    if (use_roughness == 1) {
        float roughness = texture(roughness_sampler, flipped_uv).r;
        shininess = 5. / (roughness * roughness);
    }

    float opacity = 1.;
    if (use_opacity == 1) {
        opacity = texture(opacity_sampler, flipped_uv).r;
    }

    vec3 reflection = vec3(0);
    if (use_reflection == 1) {
        if (use_cubemaps == 1) {
            reflection = texture(cubemap_sampler, reflect(-cam_dir, normal)).rgb;
        } else {
            // Transform the reflection vector into tangent space
            vec3 r = inverse(TBN) * reflect(-cam_dir, normal);
            // Then combine the uv coords with the reflection vector's xy values
            // (ignoring depth since distance to point should contribute to the reflection anyway)
            vec2 reflection_uv = mix(uv.xy - .5, r.xy*.5, .75);
            reflection_uv = reflection_uv*.5 + .5;
            // TODO need to flip u direction on some diagonal monitors
            reflection = texture(reflection_sampler, reflection_uv).rgb;
        }
    }

    vec3 lighting = vec3(0);

    for (int i = 0; i < num_light_sources; i++) {
        vec3 light = light_sources[i].position;
        vec3 light_color = light_sources[i].color;

        // Attenuation (reduces reach of lightsource)
        float la = 0.01;
        float lb = 0.02;
        float lc = 0.005;
        float d = length(light - position);
        float L = 1 / (la + d*lb + d*d*lc);
        L = 1;

        // Phong model
        // Parameters – note that specular reflection is independent of surface color!
        vec3 specular_reflection = vec3(1.0, 1.0, 1.0);
        // Vectors
        vec3 light_dir = normalize(light - position);
        vec3 reflection_dir = reflect(-light_dir, normal);

        // Calculation
        float diffuse_factor = L * max(0, dot(light_dir, normal));
        float specular_factor = L * pow(max(0, dot(reflection_dir, cam_dir)), shininess);
        lighting += diffuse_factor * diffuse_reflection * light_color + specular_factor * specular_reflection * light_color;

    }

    if (use_reflection == 1 && use_texture == 1) {
        // This is noe of the CRT screens, boost the light from its contents
        lighting += diffuse_reflection * EMMISSIVE_FACTOR;
    }

    if (mode == NORMALS_MODE) {
        color = vec4(normal*.5+.5, 1.);
    } else if (mode == REFLECTION_VECTORS_MODE) {
        color = vec4(reflect(-cam_dir, normal)*.5+.5, 1.);
    } else if (use_reflection == 1) {
        float fresnel_factor = max(0, min(1,FRESNEL_BIAS + FRESNEL_SCALE * pow(1. + dot(-cam_dir, normal), FRESNEL_POWER)));
        if (mode == REFLECTION_MODE) {
            color = vec4(reflection, opacity);
        } else {
            color = vec4(mix(lighting, reflection, fresnel_factor), opacity);
        }
    } else {
        color = vec4(lighting, opacity);
    }
}
