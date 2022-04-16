#version 450

in layout(location = 0) vec3 position;
in layout(location = 2) vec2 uv;

uniform float time;
uniform vec2 screen_size;

out vec4 color;

// This shader is based on https://www.youtube.com/watch?v=BFld4EBO2RE
// and altered for a typical whateveritiswave feel

// Raymarching parameters
#define MAX_STEPS 300
#define NEAR_ENOUGH 0.01
#define NORMAL_EPSILON 0.01
#define TOO_FAR 50.0
#define UNDERSTEPPING 0.5

// Lighting parameters
#define DIFFUSE_FACTOR 1.

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
    for (int i = 0; i < 2; i++) {
        d += noise(scale*rot*point)/scale;
        scale *= 2.;
        rot *= rot;
    }
    return d;
}

float distance_from_everything(vec3 point) {
    point.z += 4.*time;
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
    vec3 l = normalize(vec3(cos(4.*time), .1, sin(4.*time)));
    vec3 c = normalize(camera-point);
    vec3 light;
    if (length(point - camera) > TOO_FAR*0.99) {
        // Draw a gradient backgrop
        light = mix(BACKGROUND_COLOR_BOTTOM, BACKGROUND_COLOR_TOP, ray_direction.y);
        // Add some lines
        float d = min(fract(ray_direction.x*10.), 1.-fract(ray_direction.x*10.));
        light += smoothstep(.1, -.1, d)*vec3(1);
        // And a sun-like thing (though ignore its light contribution)
        light += smoothstep(.1, -.1, length(ray_direction.xy - vec2(0, -.1))-.4)*vec3(.35, .3, 0);
    } else {
        vec3 n = estimate_normal(point);
        vec3 r = reflect(-l, n);

        // Material color
        vec3 surface_color = (.2*n+.8).rbg; // color values in the range [0.6, 1.0]

        // Lighting color
        float diffuse = max(dot(n, l), 0.);
        light = DIFFUSE_FACTOR * diffuse * surface_color;
        // Mix in some detail to the shadows
        light += (1.+n.y)/2.*BACKGROUND_COLOR_BOTTOM/8.;
        light += max(0., dot(n, -l))*surface_color/4.;

        // Add a grid for futurism
        point.z += 4.*time;
        vec2 f = fract(point.xz*3.);
        float d = min(f.x, 1.-f.x);
        d = min(d, f.y);
        d = min(d, 1.-f.y);
        // Make the grid fade away in the distance
        light += smoothstep(.14, -.14, d)*vec3(1.4)*exp(-0.02*dist);

        // Mix in fog
        light = mix(FOG_COLOR, light, exp(-0.04*dist));
    }
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
    vec3 camera = vec3(4., 2.5, -4.);
    // Ray direction (normalizing it bends the lines in the sky)
    vec3 ray_direction = normalize(vec3(xy, 1.));

    float d = ray_march(camera, ray_direction);
    
    vec3 point = camera + ray_direction * d;
    
    vec3 base_color = lighting(point, camera, ray_direction, d) + dither(uv);

    color = vec4(base_color, 1.);
}
