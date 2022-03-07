extern crate sdl2;
mod scene;
mod shader;
use glow::*;
use scene::setup::create_scene;
use scene::texture;
use scene::vao::VAO;

fn main() {
    // Create a context from a sdl2 window
    let (gl, window, mut events_loop, _context) = unsafe { create_sdl2_context() };
    // Create a shader program from source
    let shader =
        unsafe { shader::Shader::new(&gl, "res/shaders/simple.vert", "res/shaders/simple.frag") };

    let single_color_shader = unsafe {
        shader::Shader::new(
            &gl,
            "res/shaders/single_color.vert",
            "res/shaders/single_color.frag",
        )
    };

    let mut scene_graph = create_scene(&gl);
    scene_graph.final_shader = Some(shader.program);

    // temp test of texture stuff
    let square = unsafe { VAO::square(&gl) };
    let texture = unsafe { texture::Texture::framebuffer_texture(&gl, 100, 100) };
    scene_graph.nodes[1].texture = Some(texture.texture);
    scene_graph.nodes[2].texture = Some(texture.texture);

    unsafe {
        shader.activate(&gl);

        // Upload some uniforms
        gl.uniform_1_f32(shader.get_uniform_location(&gl, "blue").as_ref(), 0.8);

        gl.clear_color(0.1, 0.2, 0.3, 1.0);
    }

    let first_frame_time = std::time::Instant::now();
    let mut last_frame_time = first_frame_time;

    'render: loop {
        // Time delta code from gloom-rs
        let now = std::time::Instant::now();
        let time = now.duration_since(first_frame_time).as_secs_f32();
        let delta_time = now.duration_since(last_frame_time).as_secs_f32();
        last_frame_time = now;

        for event in events_loop.poll_iter() {
            match event {
                sdl2::event::Event::KeyDown { keycode, .. } => {
                    println!("{}", keycode.expect("Could not get keycode :("))
                }
                sdl2::event::Event::MouseMotion { xrel, yrel, .. } => {
                    println!("{}, {}", xrel as f32 * delta_time, yrel as f32 * delta_time)
                }
                sdl2::event::Event::Quit { .. } => break 'render,
                _ => {}
            }
        }

        unsafe {
            // Render texture
            gl.bind_framebuffer(glow::FRAMEBUFFER, texture.framebuffer);
            gl.clear(glow::COLOR_BUFFER_BIT);
            single_color_shader.activate(&gl);
            square.draw(&gl);
            // Reset framebuffer and render scene
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.clear(glow::COLOR_BUFFER_BIT);
            shader.activate(&gl);
            scene_graph.render(&gl, 0);
            // eh
            window.gl_swap_window();
        }
    }

    unsafe {
        // Clean up
        scene_graph.teardown(&gl);

        // (extra stuff)
        single_color_shader.destroy(&gl);
        square.destroy(&gl);
    }
}

/// From glow tutorial
unsafe fn create_sdl2_context() -> (
    glow::Context,
    sdl2::video::Window,
    sdl2::EventPump,
    sdl2::video::GLContext,
) {
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 0);
    let window = video
        .window("Secret CRT stash", 1024, 769)
        .opengl()
        .resizable()
        .build()
        .unwrap();
    let gl_context = window.gl_create_context().unwrap();
    let gl = glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _);
    let event_loop = sdl.event_pump().unwrap();

    (gl, window, event_loop, gl_context)
}
