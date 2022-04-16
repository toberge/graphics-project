#version 450

in layout(location = 0) vec3 position;
in layout(location = 2) vec2 uv;

uniform float time;
uniform vec2 screen_size;

out vec4 color;

const float delta = 0.002;

float sphere(vec2 p, float r) {
    return length(p) - r;
}

float scene(vec2 p) {
    float r = .03 + .05*(sin(4.*time)*sin(7.*time)*sin(13.*time)+1.);
    float d = sphere(p, r);
    d = min(d, sphere(p-vec2(.5+.1*abs(sin(time)), 0.), r));
    d = min(d, sphere(p-vec2(-.5-.1*abs(sin(time)), 0.), r));
    return d;
}

vec3 plot(vec2 p) {
    return smoothstep(delta, -delta, scene(p)) * vec3(.8+.1*sin(vec3(5, 0, 7)*time));
}

void main() {
    // Transform into range [-1, 1]
    vec2 xy = (uv - .5) * 2.;
    vec3 col = plot(xy);
    color = vec4(col,1.0);
}
