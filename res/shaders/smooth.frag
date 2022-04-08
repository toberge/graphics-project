#version 450

// Smoothed bulbs

// Raymarching parameters
#define MAX_STEPS 50
#define NEAR_ENOUGH 0.1
#define TOO_FAR 10.0

// Lighting parameters
#define DIFFUSE_FACTOR 0.6
#define SPECULAR_FACTOR .9
#define SHININESS 16.

#define BACKGROUND_COLOR vec3(0., 0., 0.)
#define SMOOTH_FACTOR 0.3

in layout(location = 0) vec3 position;
in layout(location = 2) vec2 uv;

uniform float time;
uniform vec2 screen_size;

out vec4 color;

float sphere(vec3 point, vec3 center, float radius) {
    return length(point - center) - radius;
}

// Big thanks to Inigo Iquilez
float smooth_min(float a, float b, float k) {
    float h = max(k - abs(a-b), 0.) / k;
    return min(a, b) - h*h*h*k*(1./6.);
}

float distance_from_everything(vec3 point) {
    float d = sphere(point, vec3(sin(time), 0, 0), .8);
    d = smooth_min(d, sphere(point, vec3(cos(time), .8, -.5), .6), SMOOTH_FACTOR);
    d = smooth_min(d, sphere(point, vec3(sin(1.1*time+.5), -.8, -.3), .6), SMOOTH_FACTOR);
    d = smooth_min(d, sphere(point, vec3(1.2, 0, 0), 1.), SMOOTH_FACTOR);
    d = smooth_min(d, sphere(point, vec3(-1.2, 0, 0), 1.), SMOOTH_FACTOR);
    d = smooth_min(d, sphere(point, vec3(0, 1, 0), 1.), SMOOTH_FACTOR);
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
        d += current_distance;
        // Check status - have we reached a surface?
        if (current_distance < NEAR_ENOUGH || d > TOO_FAR)
            break;
    }
    return d;
}

// See https://www.iquilezles.org/www/articles/normalsSDF/normalsSDF.htm
vec3 estimate_normal(vec3 point) {
    vec2 e = vec2(NEAR_ENOUGH, 0); // x smol, y none
    // Find normal as tangent of distance function
    return normalize(vec3(
        distance_from_everything(point + e.xyy) - distance_from_everything(point - e.xyy),
        distance_from_everything(point + e.yxy) - distance_from_everything(point - e.yxy),
        distance_from_everything(point + e.yyx) - distance_from_everything(point - e.yyx)
    ));
}

float phong_light(vec3 point, vec3 light_position, vec3 n, vec3 camera) {
    vec3 l = normalize(light_position-point);
    vec3 c = normalize(camera-point);
    // Standard diffuse term
    float diffuse = max(dot(n, l), 0.);
    float specular = pow(max(dot(reflect(-l, n), -c), 0.), SHININESS);

    return DIFFUSE_FACTOR * diffuse + SPECULAR_FACTOR * specular;
}

vec3 lighting(vec3 point, vec3 camera, vec3 camDir, float dist) {
    // Avoid casting specular highlight from the void
    if (length(point - camera) > TOO_FAR*0.99)
        return BACKGROUND_COLOR;

    vec3 changing_color = vec3(1., cos(point.y +.9), sin(point.x));
    return phong_light(point, vec3(1, 1, -2), estimate_normal(point), camera) * changing_color;
}

void main() {
    // Transform into range [-1, 1]
    vec2 xy = (uv - .5) * 2.;

    // Ray origin
    vec3 camera = vec3(0, 0, -3);
    // Ray direction
    vec3 ray_direction = vec3(xy, 1.);

    float d = ray_march(camera, ray_direction);
    
    vec3 point = camera + ray_direction * d;
    
    vec3 base_color = lighting(point, camera, ray_direction, d);

    color = vec4(base_color, 1.);
}
