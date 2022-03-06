use glow::*;
use tobj;

/// Holds all information necessary to draw an initialized VAO.
pub struct VAO {
    pub vao: NativeVertexArray,
    vertex_buffer: NativeBuffer,
    normal_buffer: NativeBuffer,
    color_buffer: NativeBuffer,
    pub size: i32,
}

/// From the glow example
unsafe fn to_u8_slice<T>(buffer: &Vec<T>) -> &[u8] {
    core::slice::from_raw_parts(
        buffer.as_ptr() as *const u8,
        buffer.len() * core::mem::size_of::<T>(),
    )
}

/// Get the size of the given type in bytes
/// (from gloom-rs)
pub fn size_of<T>() -> i32 {
    std::mem::size_of::<T>() as i32
}

unsafe fn create_buffer(
    gl: &glow::Context,
    index: u32,
    arity: i32,
    coordinates: &Vec<f32>,
) -> NativeBuffer {
    // Generate and bind buffer
    let buffer = gl.create_buffer().expect("Unable to create buffer");
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));

    // Add coordinate buffer data
    gl.buffer_data_u8_slice(
        glow::ARRAY_BUFFER,
        to_u8_slice(coordinates),
        glow::STATIC_DRAW,
    );
    // Specify its form (xyz floats)
    gl.vertex_attrib_pointer_f32(
        index,
        arity,
        glow::FLOAT,
        false,
        arity * size_of::<f32>(),
        0,
    );
    // And enable this form
    gl.enable_vertex_attrib_array(index);

    buffer
}

impl VAO {
    /// Draws the VAO (obviously)
    pub unsafe fn draw(&self, gl: &glow::Context) {
        gl.bind_vertex_array(Some(self.vao));
        gl.draw_elements(glow::TRIANGLES, self.size, glow::UNSIGNED_INT, 0);
    }

    /// Create a VAO with the given coordinates and indices to coordinates.
    pub unsafe fn new(
        gl: &glow::Context,
        vertices: &Vec<f32>,
        normals: &Vec<f32>,
        color: &Vec<f32>,
        indices: &Vec<u32>,
    ) -> VAO {
        // Create a VAO
        let vao = gl.create_vertex_array().expect("Unable to create VAO");
        // Bind array
        gl.bind_vertex_array(Some(vao));

        // Generate and bind vertices and normals
        let vertex_buffer = create_buffer(&gl, 0, 3, vertices);
        let normal_buffer = create_buffer(&gl, 1, 3, normals);
        let color_buffer = create_buffer(&gl, 2, 4, color);

        let index_buffer = gl.create_buffer().expect("Unable to create index buffer");
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            to_u8_slice(indices),
            glow::STATIC_DRAW,
        );

        VAO {
            vao,
            vertex_buffer,
            normal_buffer,
            color_buffer,
            size: indices.len() as i32,
        }
    }

    pub unsafe fn destroy(&self, gl: &glow::Context) {
        gl.delete_vertex_array(self.vao);
        gl.delete_buffer(self.vertex_buffer);
        gl.delete_buffer(self.normal_buffer);
        gl.delete_buffer(self.color_buffer);
    }
}
