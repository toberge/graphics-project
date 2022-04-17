#version 450

in layout(location = 0) vec3 position;
in layout(location = 2) vec2 uv;

uniform float time;
uniform vec2 screen_size;

out vec4 color;

// Raymarching parameters
#define MAX_STEPS 200
#define NEAR_ENOUGH 0.001
#define TOO_FAR 15.0

// Lighting parameters
#define DIFFUSE_FACTOR  1.2
#define SPECULAR_FACTOR .3
#define SHININESS 4.

#define BACKGROUND_COLOR .2*vec3(0.35686,0.14902,0.20392)
#define SPECULAR_COLOR .4*vec3(0.95686,0.14902,0.40392)

// Toggle animated light
#define ANIMATE 1
// Toggle green shadow emphasis
//#define EMPHASIZE_SHADOW 1


// From assignment template code in TDT4230
float rand(vec2 co) { return fract(sin(dot(co.xy, vec2(12.9898,78.233))) * 43758.5453); }
float dither(vec2 uv) { return (rand(uv)*2. - 1.) / 512.; }


float sphere(vec3 point, vec3 center, float radius) {
    return length(point - center) - radius;
}

// SDF from Inigo Iquilez
float capsule( vec3 p, vec3 a, vec3 b, float r )
{
  vec3 pa = p - a, ba = b - a;
  float h = clamp( dot(pa,ba)/dot(ba,ba), 0.0, 1.0 );
  return length( pa - ba*h ) - r;
}

// Smooth operators by Inigo Iquilez
float smooth_min(float a, float b, float k) {
    float h = clamp( 0.5 + 0.5*(a-b)/k, 0.0, 1.0 );
    return mix( a, b, h ) - k*h*(1.0-h);
}

float smooth_diff( float d1, float d2, float k ) {
    float h = clamp( 0.5 - 0.5*(d2+d1)/k, 0.0, 1.0 );
    return mix( d2, -d1, h ) + k*h*(1.0-h);
}

float distance_from_everything(vec3 point) {
    float d = point.y;
    d = smooth_min(d, capsule(point, vec3(0), vec3(0, 2, 0), 1.), 1.2);
    vec3 eye_pos = vec3(.3, 1.8, -1.);
    d = smooth_diff(sphere(point, eye_pos, .15), d, .05);
    d = smooth_diff(sphere(point, vec3(-eye_pos.x, eye_pos.yz), .15), d, .05);
    eye_pos.y -= .4;
    eye_pos.x -= .1;
    d = smooth_diff(capsule(point, eye_pos, vec3(-eye_pos.x, eye_pos.yz), .15), d, .05);
    return d;
}

vec3 sun_position() {
    float orbit_radius = 5.;
#ifdef ANIMATE
    return vec3(orbit_radius*cos(1.2*time), 2., orbit_radius*sin(1.2*time));
#else
    return vec3(-.5*orbit_radius, 1., -orbit_radius);
#endif
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
        d += current_distance;
        // Check status - have we reached a surface?
        if (current_distance < NEAR_ENOUGH || d > TOO_FAR)
            break;
    }
    return d;
}

vec3 sample_sun(vec3 ray_origin, vec3 ray_direction) {
    float d = 0.0;
    float intensity = 0.;
    float inside = 0.;
    vec3 light = sun_position();
    for (int i = 0; i < MAX_STEPS; i++) {
        // Where we stand
        vec3 point = ray_origin + ray_direction*d;
        // How far anything is from us
        float current_distance = distance_from_everything(point);
        // March on in fixed steps
        d += 1.;
        // Sample distance to light
        intensity += pow(smoothstep(2., 0., length(point - light)), 3.);
        // Sample negative values of the SDF
        if (current_distance < NEAR_ENOUGH)
            inside += abs(current_distance);
        if (d > TOO_FAR)
            break;
    }
    // Diminish intensity based on how obstructed the view of the sun was
    return vec3(intensity) * exp(-10.*inside);
}

// Approach taken from https://iquilezles.org/articles/rmshadows
float ray_shadow(vec3 ray_origin, vec3 ray_direction, vec3 light_position) {
    // Start some distance along the rain to avoid counting shadow because we're close to the surface
    float d = NEAR_ENOUGH;
    float shade = 1.0;
    float shadow_factor = 32.;
    // Sign of the direction we're travelling,
    // if this changes we know that we passed the light source.
    vec3 side = sign(light_position - ray_origin);
    for (int i = 0; i < MAX_STEPS; i++) {
        // Where we stand
        vec3 point = ray_origin + ray_direction*d;
        // How far anything is from us
        float current_distance = distance_from_everything(point);
        // March on
        d += current_distance;
        // Update to more significant shadow value
        shade = min(shade, shadow_factor * current_distance / d);
        // Only check if we've gone too far or gone beyond the light source
        if (d > TOO_FAR || sign(light_position - point) != side)
            break;
    }
    return max(shade, 0.);
}

// See https://iquilezles.org/articles/normalsSDF
vec3 estimate_normal(vec3 point) {
    vec2 e = vec2(NEAR_ENOUGH, 0); // x smol, y none
    // Find normal as tangent of distance function
    return normalize(vec3(
        distance_from_everything(point + e.xyy) - distance_from_everything(point - e.xyy),
        distance_from_everything(point + e.yxy) - distance_from_everything(point - e.yxy),
        distance_from_everything(point + e.yyx) - distance_from_everything(point - e.yyx)
    ));
}


vec3 lighting(vec3 point, vec3 camera, vec3 ray_direction, float dist) {
    // Avoid casting specular highlight from the void
    if (length(point - camera) > TOO_FAR*0.99)
        return BACKGROUND_COLOR;

    vec3 light_position = sun_position();

    vec3 n = estimate_normal(point);
    vec3 l = normalize(light_position-point);
    vec3 c = normalize(camera-point);
    vec3 r = reflect(-l, n);

    // Standard phong lighting
    float diffuse = max(dot(n, l), 0.);
    float specular = pow(max(dot(r, -c), 0.), SHININESS);
    vec3 surface_color = (.2*n+.8).rbg; // color values in the range [0.6, 1.0]
    vec3 phong = DIFFUSE_FACTOR * diffuse * surface_color + SPECULAR_FACTOR * specular * SPECULAR_COLOR;

    // Find soft shadow factor by marching from where you stand
    float shade = ray_shadow(point, l, light_position);

#ifdef EMPHASIZE_SHADOW
    return vec3(l.x, 0., l.z) * SPECULAR_COLOR + (1.-shade) * vec3(0, 1, 0);
#else
    return mix(BACKGROUND_COLOR, phong, shade);
#endif
}


void main() {
    // Transform into range [-1, 1]
    vec2 xy = (uv - .5) * 2.;

    // Ray origin
    vec3 camera = vec3(0., 1., -3.);
    // Ray direction
    vec3 ray_direction = vec3(xy, 1.);

    float d = ray_march(camera, ray_direction);
    
    vec3 point = camera + ray_direction * d;
    
    vec3 base_color = lighting(point, camera, ray_direction, d);
    base_color += sample_sun(camera, ray_direction);
    base_color = smoothstep(0., 1., base_color);

    color = vec4(base_color + dither(uv), 1.);
}
