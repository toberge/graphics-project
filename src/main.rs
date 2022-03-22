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
use scene::camera::Camera;
use scene::setup::create_scene;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 769;
const LOOK_SPEED: f32 = 0.005;
const MOVE_SPEED: f32 = 20.0;
const MOUSE_LOOK: bool = true;

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
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Use mouse controls with invisible mouse confined to the screen.
    if MOUSE_LOOK {
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
            gl.enable(glow::MULTISAMPLE);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.enable(glow::DEBUG_OUTPUT_SYNCHRONOUS);
            gl.debug_message_callback(debug_callback);
        }

        // Create a shader program from source
        let shader = unsafe {
            shader::Shader::new(&gl, "res/shaders/simple.vert", "res/shaders/simple.frag")
        };

        let mut scene_graph = create_scene(&gl);
        scene_graph.final_shader = Some(shader.program);

        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;

        // Camera object that holds current position, yaw and pitch
        let mut cam = Camera::new(WINDOW_WIDTH, WINDOW_HEIGHT);

        cam.z += 10.;
        cam.y += 3.;

        let mut pitch = 0.;
        let mut yaw = 0.;

        loop {
            // Time delta code from gloom-rs
            let now = std::time::Instant::now();
            let time = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    cam.handle_keys(key, delta_time * MOVE_SPEED);
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if MOUSE_LOOK {
                if let Ok(mut delta) = mouse_delta.lock() {
                    cam.yaw += delta.0 * LOOK_SPEED;
                    cam.pitch += delta.1 * LOOK_SPEED;
                    yaw += delta.0 * LOOK_SPEED;
                    pitch += delta.1 * LOOK_SPEED;
                    *delta = (0.0, 0.0);
                }
            }

            unsafe {
                // Update transformations
                scene_graph.update(&gl);
                // Render content
                scene_graph.render_screens(&gl, time);
                // Render reflections
                scene_graph.render_reflections(&gl);
                // Reset framebuffer and render scene
                gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                gl.clear_color(0.1, 0.2, 0.3, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                shader.activate(&gl);
                scene_graph.render(
                    &gl,
                    0,
                    &cam.create_transformation(),
                    &glm::vec3(cam.x, cam.y, cam.z),
                    true,
                );
                // eh
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
