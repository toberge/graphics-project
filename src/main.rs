extern crate sdl2;
mod scene;
use glow::*;
use scene::vao::VAO;

fn main() {
    // Create a context from a sdl2 window
    let (gl, window, mut events_loop, _context) = unsafe { create_sdl2_context() };
    // Create a shader program from source
    let program = unsafe { create_program(&gl, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE) };
    // Create a vertex buffer and vertex array object

    let test = unsafe {
        VAO::new(
            &gl,
            &vec![0.7, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            &vec![0.0, 0.0, 1.0, 0.0].repeat(3),
            &vec![0.0, 1.0, 0.0, 1.0].repeat(3),
            &vec![0, 1, 2],
        )
    };

    unsafe {
        gl.use_program(Some(program));

        // Upload some uniforms
        set_uniform(&gl, program, "blue", 0.8);

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
            gl.clear(glow::COLOR_BUFFER_BIT);
            test.draw(&gl);
            window.gl_swap_window();
        }
    }

    unsafe {
        // Clean up
        gl.delete_program(program);
        test.destroy(&gl);
    }
}

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
        .window("Hello triangle!", 1024, 769)
        .opengl()
        .resizable()
        .build()
        .unwrap();
    let gl_context = window.gl_create_context().unwrap();
    let gl = glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _);
    let event_loop = sdl.event_pump().unwrap();

    (gl, window, event_loop, gl_context)
}

unsafe fn create_program(
    gl: &glow::Context,
    vertex_shader_source: &str,
    fragment_shader_source: &str,
) -> NativeProgram {
    let program = gl.create_program().expect("Cannot create program");

    let shader_sources = [
        (glow::VERTEX_SHADER, vertex_shader_source),
        (glow::FRAGMENT_SHADER, fragment_shader_source),
    ];

    let mut shaders = Vec::with_capacity(shader_sources.len());

    for (shader_type, shader_source) in shader_sources.iter() {
        let shader = gl
            .create_shader(*shader_type)
            .expect("Cannot create shader");
        gl.shader_source(shader, shader_source);
        gl.compile_shader(shader);
        if !gl.get_shader_compile_status(shader) {
            panic!("{}", gl.get_shader_info_log(shader));
        }
        gl.attach_shader(program, shader);
        shaders.push(shader);
    }

    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        panic!("{}", gl.get_program_info_log(program));
    }

    for shader in shaders {
        gl.detach_shader(program, shader);
        gl.delete_shader(shader);
    }

    program
}

unsafe fn set_uniform(gl: &glow::Context, program: NativeProgram, name: &str, value: f32) {
    let uniform_location = gl.get_uniform_location(program, name);
    // See also `uniform_n_i32`, `uniform_n_u32`, `uniform_matrix_4_f32_slice` etc.
    gl.uniform_1_f32(uniform_location.as_ref(), value)
}

const VERTEX_SHADER_SOURCE: &str = r#"#version 130
  in vec3 in_position;
  out vec3 position;
  void main() {
    position = in_position;
    gl_Position = vec4(in_position.xy - 0.5, 0.0, 1.0);
  }"#;
const FRAGMENT_SHADER_SOURCE: &str = r#"#version 130
  precision mediump float;
  in vec3 position;
  out vec4 color;
  uniform float blue;
  void main() {
    color = vec4(position.xy, blue, 1.0);
  }"#;
