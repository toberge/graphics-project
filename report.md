---
title: "TDT4230 Final Project"
author: "Tore Bergebakken"
titlepage: true
bibliography: "sources.bib"
citation-style: "ieee"
link-citations: true
---

# Concept

The project's goal was a scene with a heap of CRT monitors showing various visuals rendered in real-time, as well as reflecting the surrounding scene in a somewhat realistic manner.

# Implementation

The project was written in Rust with the `glow-rs` wrapper for OpenGL. The scene graph was implemented with node references as indexes into a list of nodes, inspired by an article. [TODO LINK IT PLZ]

## Animated Camera

The camera was intended to revolve around the center of the scene, showing the content of the screens and reflections on the screens at the edges. This was achieved using a look-at matrix with vectors defined like so:

$$
\begin{aligned}
  v_{\text{ground}} &= [\cos r, 0, \sin r]\\
  v_{\text{eye}} &= [\cos r, h, \sin r]\\
  v_{\text{center}} &= \text{lowermost point of the crt structure}\\
  v_{\text{up}} &= (v_{\text{center}} - v_{\text{eye}}) \times ((v_{\text{center}} - v_{\text{eye}}) \times (v_{\text{ground}} - v_{\text{eye}}))
,\end{aligned}
$$
where $r$ is the radius and $h$ is the height of the orbit.

For inspecting visuals on the screens more closely, button presses were tied to animations that zoomed in and made the camera stay in front of a chosen screen. The transition was achieved using linear and spherical interpolation (GLM's `mix` and `slerp` functions) of position and orientation vectors respectively.

## Reflections

For real-time reflections, two different approaches were tested. Both involved rendering the scene from the perspective of the reflective objects.

### Planar Approach

<!-- just render one perspective with relatively wide FOV and map it using [show formula of that function] -->

The first approach was to render the scene from the perspective of the reflective object to _one_ two-dimensional texture. 

blah blah here's ther formula TODO

$$
\begin{aligned}
  r_{uv} = ( 0.25\cdot (t_{uv} - 0.5) + 0.75\cdot 0.5\cdot  (TBN^{-1} \cdot  r_{x,y} ) )\cdot 0.5 + 0.5
,\end{aligned}
$$
where $t_{uv}$ is the texture coordinates, $TBN$ is a matrix that transforms from tangent to world space, $r$ is the reflection vector and $r_{uv}$ is the reflection texture coordinates.

### Cubemaps

Cubemaps are a collection of six textures representing the faces of a cube. In the case of reflections, the textures are based on what can be seen of the surrounding area from a reflective object. This was implemented using look-at matrices calculated with the GLM function

## Visualizations

Most of the screens were made to show some ray-marched scene.

### Ray Marching

A personal note: I stumbled down the rabbit hole of this rendering technique a month or so before the project started via some intriguing videos [@cp_raymarching; @aoc_raymarching], and decided I would have to delve into this in the project in some way.

<!-- involves sampling a distance estimator -->

### Signed Distance Functions

Signed distance functions, hereafter called SDFs, give exact values or estimates for the distance to a surface, and the distance _inside_ a surface, with the latter represented as negative values.

<!-- simple illustration and explanation of sphere sdf -->

$$
math
$$

SDFs can be combined using simple operations like `min` and `max` (corresponding to boolean union and intersection, respectively) to create more complex shapes. Additionally, one can increase the complexity of a surface by adding some other function to its SDF. <!-- demoed on that ripple shader -->

Some of the shaders in this project use Inigo Iquelez' smooth combination operators [@iq_smooth], and some SDFs might be lifted from the same person's library [@iq_sdfs].

<!-- mention using iq's smooth operators -->

### Bloom

Bloom was achieved by finding the number of steps taken to reach the back of the scene and mixing in a color based on a multiple of this step count. This created a ringing effect that I personally liked the look of.

<!-- pic of gyroid shader maybe -->

### Ambient Occlusion

### Shadows

Similarly to bloom and ambient occlusion, it is possible to use the (...)

The shaders with shadows in this project use the simpler soft shadow approch proposed by Inigo Quilez [@iq_shadows].

<!-- show that shadow thing -->

# Conclusion

Don't wade too far off into the rabbit hole of SDF and expect reflections to take very little time.

# References

