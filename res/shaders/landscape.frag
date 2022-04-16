#version 450

in layout(location = 0) vec3 position;
in layout(location = 2) vec2 uv;

uniform float time;
uniform vec2 screen_size;

out vec4 color;

// This shader is based on https://www.youtube.com/watch?v=BFld4EBO2RE

// Raymarching parameters
#define MAX_STEPS 300
#define NEAR_ENOUGH 0.01
#define NORMAL_EPSILON 0.01
#define TOO_FAR 100.0
#define UNDERSTEPPING 0.4

// Lighting parameters
#define DIFFUSE_FACTOR 1.2

// Colors
#define BACKGROUND_COLOR_BOTTOM vec3(0.35686,0.14902,0.20392)
#define BACKGROUND_COLOR_TOP .2*vec3(0.35686,0.14902,0.20392)
#define SHADOW_COLOR .2*vec3(0.35686,0.14902,0.20392)
#define FOG_COLOR .6*vec3(0.95686,0.14902,0.40392)
#define CLOUD_COLOR vec3(0.96863,0.96078,0.86667)

#define PI 3.14159265359

#define ROT(a)           \
    transpose(mat2(      \
         cos(a), sin(a), \
        -sin(a), cos(a)  \
    ))

const mat2 rot4 = ROT(10./42.);

// Value noise thing from iq's video

float coefficient(vec2 ij) {
    vec2 uv = 69.*fract(ij/PI/3.);
    return 2.*fract(uv.x*uv.y*(uv.x+uv.y))-1.;
}

float noise(vec2 point) {
    float x = point.x;
    float z = point.y;
    // Get indices in floor grid and coefficients based on it
    vec2 ij = floor(point.xy);
    float a = coefficient(ij);
    float b = coefficient(vec2(ij.x+1., ij.y));
    float c = coefficient(vec2(ij.x, ij.y+1.));
    float d = coefficient(ij+1.);

    return a +
        (b-a)*smoothstep(0., 1., x-ij.x) +
        (c-a)*smoothstep(0., 1., z-ij.y) +
        (a-b-c+d)*smoothstep(0., 1., x-ij.x)*smoothstep(0., 1., z-ij.y);
}

float terrain(vec2 point) {
    float d = noise(point);
    mat2 rot = rot4;
    float scale = 2.;
    // Fourier-like summation
    for (int i = 0; i < 6; i++) {
        d += noise(scale*rot*point)/scale;
        scale *= 2.;
        rot *= rot;
    }
    return d;
}

float distance_from_everything(vec3 point) {
    point.z += time;
    float d = point.y;
    d += terrain(point.xz/3.)/1.;
    return d;
}

float ray_march(vec3 ray_origin, vec3 ray_direction) {
    // How far we've traveled
    float d = 0.0;
    for (int i = 0; i < MAX_STEPS; i++) {
        // Where we stand
        vec3 point = ray_origin + ray_direction*d;
        // How far anything is from us
        float current_distance = distance_from_everything(point);
        // March on
        d += current_distance * UNDERSTEPPING;
        // Check status - have we reached a surface?
        if (current_distance < NEAR_ENOUGH || d > TOO_FAR)
            break;
    }
    return d;
}

// Approach taken from https://iquilezles.org/articles/rmshadows
float ray_shadow(vec3 ray_origin, vec3 ray_direction) {
    // Start some distance along the rain to avoid counting shadow because we're close to the surface
    float d = NEAR_ENOUGH;
    float shade = 1.0;
    float shadow_factor = 32.;
    // Sign of the direction we're travelling,
    // if this changes we know that we passed the light source.
    for (int i = 0; i < MAX_STEPS/2; i++) {
        // Where we stand
        vec3 point = ray_origin + ray_direction*d;
        // How far anything is from us
        float current_distance = distance_from_everything(point);
        // March on
        d += current_distance*UNDERSTEPPING*1.5;
        // Update to more significant shadow value
        shade = min(shade, shadow_factor * current_distance / d);
        // Only check if we've gone too far or gone beyond the light source
        if (d > TOO_FAR)
            break;
    }
    return max(shade, 0.);
}

// See https://iquilezles.org/articles/normalsSDF
vec3 estimate_normal(vec3 point) {
    vec2 e = vec2(NORMAL_EPSILON, 0); // x smol, y none
    // Find normal as tangent of distance function
    return normalize(vec3(
        distance_from_everything(point + e.xyy) - distance_from_everything(point - e.xyy),
        distance_from_everything(point + e.yxy) - distance_from_everything(point - e.yxy),
        distance_from_everything(point + e.yyx) - distance_from_everything(point - e.yyx)
    ));
}


vec3 lighting(vec3 point, vec3 camera, vec3 ray_direction, float dist) {
    // Use a light source infinitely far away instead of a point light
    vec3 l = normalize(vec3(-.7, .1, -.2));
    vec3 c = normalize(camera-point);
    vec3 light;
    if (length(point - camera) > TOO_FAR*0.99) {
        // Draw sky
        light = mix(BACKGROUND_COLOR_BOTTOM, BACKGROUND_COLOR_TOP, ray_direction.y);
        // Add clouds
        // Project raydir.xy (screen pos) onto sky plane
        vec2 cloud_pos = vec2(0);
        cloud_pos.x = 9. + 24.*mix(ray_direction.x, ray_direction.x/10., sqrt(max(0., ray_direction.y+.1)));
        cloud_pos.y = 10./(ray_direction.y*.5+.5) + time*.2;
        light += CLOUD_COLOR*.5*smoothstep(-.2, .8, terrain(cloud_pos));
    } else {
        vec3 n = estimate_normal(point);
        vec3 r = reflect(-l, n);

        // Material color
        vec3 surface_color = (.2*n+.8).rbg; // color values in the range [0.6, 1.0]
        // Mix in some snow at the mountaintops
        surface_color = mix(surface_color, vec3(1), smoothstep(0.5, 1.5, point.y));

        // Lighting color

        // Standard phong lighting
        float diffuse = max(dot(n, l), 0.);
        light = DIFFUSE_FACTOR * diffuse * surface_color;

        // Find shadow factor by marching from where you stand
        float shade = ray_shadow(point, l);
        // Mix in shadow
        light = mix(SHADOW_COLOR, light, shade);
        // Add some detail to the shadows
        // (simulating light bouncing around to areas in shadow)
        light += (1.+n.y)/2.*BACKGROUND_COLOR_BOTTOM/8.;
        light += max(0., dot(n, -l))*surface_color/4.;

        // Mix in fog
        light = mix(FOG_COLOR, light, exp(-0.03*dist));
    }
    // Enhance contrast & saturation I guess
    light = smoothstep(0., 1., light);
    return light;
}

// From assignment template code in TDT4230
// (really just the same random function as in the book of shaders and other places)
float rand(vec2 co) { return fract(sin(dot(co.xy, vec2(12.9898,78.233))) * 43758.5453); }
float dither(vec2 uv) { return (rand(uv)*2. - 1.) / 512.; }

void main() {
    // Transform into range [-1, 1]
    vec2 xy = (uv - .5) * 2.;

    // Ray origin
    vec3 camera = vec3(2., 2.5, -4.);
    // Ray direction
    vec3 ray_direction = vec3(xy, 1.);

    float d = ray_march(camera, ray_direction);
    
    vec3 point = camera + ray_direction * d;
    
    vec3 base_color = lighting(point, camera, ray_direction, d) + dither(uv);

    color = vec4(base_color, 1.);
}
