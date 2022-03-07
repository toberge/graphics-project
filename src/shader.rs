use std::path::Path;

use glow::*;

pub struct Shader {
    pub program: NativeProgram,
}

unsafe fn read_and_compile_shader(
    gl: &glow::Context,
    shader_path: &str,
    shader_type: u32,
) -> NativeShader {
    let path = Path::new(shader_path);
    let source = std::fs::read_to_string(path).expect(&format!("No shader at {}", path.display()));
    let shader = gl
        .create_shader(shader_type)
        .expect("Could not create shader");
    gl.shader_source(shader, &source);
    gl.compile_shader(shader);
    if !gl.get_shader_compile_status(shader) {
        panic!(
            "Shader compilation failed for {}!\n{}",
            shader_path,
            gl.get_shader_info_log(shader)
        );
    }
    shader
}

impl Shader {
    pub unsafe fn get_uniform_location(
        &self,
        gl: &glow::Context,
        location: &str,
    ) -> Option<NativeUniformLocation> {
        gl.get_uniform_location(self.program, location)
    }

    pub unsafe fn activate(&self, gl: &glow::Context) {
        gl.use_program(Some(self.program));
    }

    pub unsafe fn new(
        gl: &glow::Context,
        vertex_shader_path: &str,
        fragment_shader_path: &str,
    ) -> Shader {
        let program = gl.create_program().expect("Cannot create program");
        let vertex_shader = read_and_compile_shader(gl, vertex_shader_path, glow::VERTEX_SHADER);
        let fragment_shader =
            read_and_compile_shader(gl, fragment_shader_path, glow::FRAGMENT_SHADER);
        // Add shaders to program
        gl.attach_shader(program, vertex_shader);
        gl.attach_shader(program, fragment_shader);
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!(
                "Shader linking error!\n{}",
                gl.get_program_info_log(program)
            );
        }
        // Clean up
        gl.detach_shader(program, vertex_shader);
        gl.detach_shader(program, fragment_shader);
        gl.delete_shader(vertex_shader);
        gl.delete_shader(fragment_shader);
        Shader { program }
    }

    pub unsafe fn destroy(&self, gl: &glow::Context) {
        gl.delete_program(self.program);
    }
}
