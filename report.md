---
title: "TDT4230 Final Project"
author: "Tore Bergebakken"
titlepage: true
---

# Concept

# Animated Camera

The camera was intended to revolve around the center of the scene, showing the content of the screens and reflections on the screens at the edges. This was achieved using a look-at matrix with vectors defined like so:

$$
math
$$

For inspecting visuals on the screens more closely, button presses were tied to animations that zoomed in and made the camera stay in front of a chosen screen. The transition was achieved using linear and spherical interpolation (GLM's `mix` and `slerp` functions) of position and orientation vectors respectively.

# Reflections

For real-time reflections, two different approaches were tested. Both involved rendering the scene from the perspective of the reflective objects.

## Planar Approach

<!-- just render one perspective with relatively wide FOV and map it using [show formula of that function] -->

$$
math
$$

## Cubemaps

Cubemaps are a collection of six textures representing the faces of a cube. In the case of reflections, the textures are based on what can be seen of the surrounding area from a reflective object. This was implemented using look-at matrices calculated with the GLM function

# Visualizations

Most of the screens were made to show some ray-marched scene.

## Ray Marching

<!-- involves sampling a distance estimator -->

## Signed Distance Functions

Signed distance functions, hereafter called SDFs, give exact values or estimates for the distance to a surface, and the distance _inside_ a surface, with the latter represented as negative values.

<!-- simple illustration and explanation of sphere sdf -->

$$
math
$$

SDFs can be combined using ...

<!-- mention using iq's smooth operators -->

## Bloom

Bloom was achieved by finding the number of steps taken to reach the back of the scene and mixing in a color based on a multiple of this step count. This created a ringing effect that I personally liked the look of.

<!-- pic of gyroid shader maybe -->

## Ambient Occlusion

## Shadows

Similarly to bloom and ambient occlusion, it is possible to use the (...)

<!-- show that shadow thing -->

# Conclusion

Don't wade too far off into the rabbit hole of SDF and expect reflections to take very little time.
