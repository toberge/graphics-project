extern crate nalgebra_glm as glm;
mod scene;
mod shader;
use glow::*;
use glutin::event::{
    DeviceEvent,
    ElementState::{Pressed, Released},
    Event, KeyboardInput,
    VirtualKeyCode::{self, *},
    WindowEvent,
};
use glutin::event_loop::ControlFlow;
use scene::setup::create_scene;
use scene::{
    camera::{Camera, FirstPersonCamera, RevolvingCamera},
    texture,
    vao::VAO,
};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const LOOK_SPEED: f32 = 0.005;
const MOVE_SPEED: f32 = 20.0;
const MOUSE_LOOK: bool = true;
const FREE_LOOK: bool = false;

struct State {
    just_reflections: bool,
    just_reflection_vectors: bool,
    just_normals: bool,
}

impl State {
    fn new() -> State {
        State {
            just_reflections: false,
            just_reflection_vectors: false,
            just_normals: false,
        }
    }

    fn encode(&self) -> i32 {
        if self.just_reflections {
            1
        } else if self.just_normals {
            2
        } else if self.just_reflection_vectors {
            3
        } else {
            0
        }
    }
}

// Debug callback to panic upon enountering any OpenGL error
// from gloom-rs :)))))
pub fn debug_callback(source: u32, e_type: u32, id: u32, severity: u32, error_message: &str) {
    if e_type != glow::DEBUG_TYPE_ERROR {
        return;
    }
    if severity == glow::DEBUG_SEVERITY_HIGH
        || severity == glow::DEBUG_SEVERITY_MEDIUM
        || severity == glow::DEBUG_SEVERITY_LOW
    {
        let severity_string = match severity {
            glow::DEBUG_SEVERITY_HIGH => "high",
            glow::DEBUG_SEVERITY_MEDIUM => "medium",
            glow::DEBUG_SEVERITY_LOW => "low",
            _ => "unknown",
        };
        panic!(
            "{}: Error of severity {} raised from {}: {}\n",
            id, severity_string, source, error_message
        );
    }
}

fn main() {
    ///// This is from gloom-rs as well /////

    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(false)
        .with_inner_size(glutin::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    //.with_multisampling(4);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Use mouse controls with invisible mouse confined to the screen.
    if MOUSE_LOOK && FREE_LOOK {
        windowed_context
            .window()
            .set_cursor_grab(true)
            .expect("failed to grab cursor");
        windowed_context.window().set_cursor_visible(false);
    }

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);
    // Do the same for *just* pressed and not held keys
    let arc_just_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    let just_pressed_keys = Arc::clone(&arc_just_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers. This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let (context, gl) = unsafe {
            let c = windowed_context.make_current().unwrap();
            let gl = glow::Context::from_loader_function(|s| c.get_proc_address(s) as *const _);
            (c, gl)
        };

        // Set OpenGL options
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);
            gl.enable(glow::CULL_FACE);
            //gl.enable(glow::MULTISAMPLE);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.enable(glow::DEBUG_OUTPUT_SYNCHRONOUS);
            gl.debug_message_callback(debug_callback);
        }

        // Create a shader program from source
        let shader =
            unsafe { shader::Shader::new(&gl, "res/shaders/world.vert", "res/shaders/world.frag") };
        let post_shader =
            unsafe { shader::Shader::new(&gl, "res/shaders/post.vert", "res/shaders/post.frag") };
        let post_buffer = unsafe {
            texture::PostProcessingTexture::new(&gl, WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
        };
        let crt_buffer = unsafe {
            texture::PostProcessingTexture::new(&gl, WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
        };

        let canvas = unsafe { VAO::square(&gl) };

        let mut scene_graph = create_scene(&gl);
        scene_graph.final_shader = Some(shader.program);

        scene_graph.update_transformations(scene_graph.root, &glm::identity(), &glm::zero());

        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;

        // Camera object that revolves around the center of the scene
        // and can zoom in on one of the 8 lower CRTs
        let mut rotcam = RevolvingCamera::new(
            glm::vec3(0., 2., 0.),
            15.,
            7.,
            (0..16)
                .into_iter()
                .map(|i| {
                    let node = scene_graph.get_node(scene_graph.cameras[i]);
                    return (node.world_position(), node.look_at_eye(2.));
                })
                .collect::<Vec<(glm::Vec3, glm::Vec3)>>(),
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        );
        // Camera object that holds current position, yaw and pitch
        let mut fpcam = FirstPersonCamera::new(WINDOW_WIDTH, WINDOW_HEIGHT);
        fpcam.z += 10.;
        fpcam.y += 3.;

        let mut state = State::new();

        // Render reflections once since there's nothing dynamici in the scene
        // other than the contents of the screens
        scene_graph.update(&gl);
        unsafe {
            scene_graph.render_reflections(&gl);
        }

        loop {
            // Time delta code from gloom-rs
            let now = std::time::Instant::now();
            let time = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;

            // Handle keyboard input
            // Keypresses trigger state changes
            if let Ok(mut keys) = just_pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        VirtualKeyCode::R => {
                            state.just_reflections = !state.just_reflections;
                        }
                        VirtualKeyCode::N => {
                            state.just_normals = !state.just_normals;
                        }
                        VirtualKeyCode::M => {
                            state.just_reflection_vectors = !state.just_reflection_vectors;
                        }
                        _ => {}
                    }
                }
                // All presses handled, clear your memory
                keys.clear();
            }
            // Continuously pressed keys trigger camera movements
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    if FREE_LOOK {
                        fpcam.handle_keys(key, time, delta_time * MOVE_SPEED);
                    } else {
                        rotcam.handle_keys(key, time, delta_time * MOVE_SPEED);
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if MOUSE_LOOK {
                if let Ok(mut delta) = mouse_delta.lock() {
                    if FREE_LOOK {
                        fpcam.handle_mouse((delta.0 * LOOK_SPEED, delta.1 * LOOK_SPEED));
                    } else {
                        rotcam.handle_mouse((delta.0 * LOOK_SPEED, delta.1 * LOOK_SPEED));
                    }
                    *delta = (0.0, 0.0);
                }
            }

            unsafe {
                // Update transformations
                scene_graph.update(&gl);
                let view_transform = if FREE_LOOK {
                    fpcam.create_transformation(time, delta_time)
                } else {
                    rotcam.create_transformation(time, delta_time)
                };
                let camera_position = if FREE_LOOK {
                    fpcam.get_position(time)
                } else {
                    rotcam.get_position(time)
                };

                // Render content
                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(crt_buffer.framebuffer));
                gl.viewport(0, 0, crt_buffer.width, crt_buffer.height);
                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                scene_graph.render_screens(&gl, time, &view_transform);

                // Reset framebuffer and render scene
                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(post_buffer.framebuffer));
                gl.viewport(0, 0, WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32);
                gl.clear_color(0., 0., 0., 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                shader.activate(&gl);
                gl.uniform_1_i32(
                    gl.get_uniform_location(shader.program, "mode").as_ref(),
                    state.encode(),
                );
                scene_graph.render(
                    &gl,
                    scene_graph.root,
                    &view_transform,
                    &camera_position,
                    true,
                );
                // Post-processing
                gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                gl.use_program(Some(post_shader.program));
                gl.uniform_1_i32(
                    gl.get_uniform_location(post_shader.program, "mode")
                        .as_ref(),
                    state.encode(),
                );
                gl.uniform_3_f32_slice(
                    gl.get_uniform_location(post_shader.program, "camera_position")
                        .as_ref(),
                    camera_position.as_ref(),
                );
                gl.active_texture(glow::TEXTURE0);
                gl.bind_texture(glow::TEXTURE_2D, Some(post_buffer.color_buffer_texture));
                gl.active_texture(glow::TEXTURE1);
                gl.bind_texture(glow::TEXTURE_2D, Some(post_buffer.depth_buffer_texture));
                gl.active_texture(glow::TEXTURE2);
                gl.bind_texture(glow::TEXTURE_2D, Some(crt_buffer.color_buffer_texture));
                gl.active_texture(glow::TEXTURE3);
                gl.bind_texture(glow::TEXTURE_2D, Some(crt_buffer.depth_buffer_texture));
                canvas.draw(&gl);
                // Swap which color buffer is displayed
                context.swap_buffers().unwrap();
            }
        }

        //unsafe {
        //    // Clean up
        //    scene_graph.teardown(&gl);

        //    // (extra stuff)
        //    single_color_shader.destroy(&gl);
        //    square.destroy(&gl);
        //}
    });

    ///// The rest is from gloom-rs as well /////

    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events get handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: key_state,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                // Keep track of *just* pressed keys
                if let Ok(mut keys) = arc_just_pressed_keys.lock() {
                    match key_state {
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                        _ => {}
                    }
                }
                // As well as pressed *and held* keys
                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        }
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle escape separately
                match keycode {
                    Escape => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            _ => {}
        }
    });
}
