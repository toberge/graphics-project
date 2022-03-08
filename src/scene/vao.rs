use glow::*;
use tobj;

#[derive(Clone, Copy)]
/// Holds all information necessary to draw an initialized VAO.
pub struct VAO {
    pub vao: NativeVertexArray,
    vertex_buffer: NativeBuffer,
    normal_buffer: NativeBuffer,
    uv_buffer: NativeBuffer,
    color_buffer: NativeBuffer,
    pub size: i32,
    pub shininess: f32,
}

pub fn load_obj(file: &str) -> (Vec<tobj::Model>, Vec<tobj::Material>) {
    // From tobj example + earlier assignment
    let obj = tobj::load_obj(
        file,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    );
    assert!(obj.is_ok());

    let (models, materials) = obj.expect("Failed to load OBJ file");

    let materials = materials.expect("Failed to load material");

    return (models, materials);
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
        uvs: &Vec<f32>,
        color: &Vec<f32>,
        indices: &Vec<u32>,
        shininess: f32,
    ) -> VAO {
        // Create a VAO
        let vao = gl.create_vertex_array().expect("Unable to create VAO");
        // Bind array
        gl.bind_vertex_array(Some(vao));

        // Generate and bind vertices and normals
        let vertex_buffer = create_buffer(&gl, 0, 3, vertices);
        let normal_buffer = create_buffer(&gl, 1, 3, normals);
        let uv_buffer = create_buffer(&gl, 2, 2, uvs);

        let color_buffer = create_buffer(&gl, 3, 4, color);

        let index_buffer = gl.create_buffer().expect("Unable to create index buffer");
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            to_u8_slice(indices),
            glow::STATIC_DRAW,
        );

        VAO {
            vao,
            size: indices.len() as i32,
            vertex_buffer,
            normal_buffer,
            uv_buffer,
            color_buffer,
            shininess,
        }
    }

    pub unsafe fn from_mesh(
        gl: &glow::Context,
        model: &tobj::Model,
        materials: &Vec<tobj::Material>,
    ) -> VAO {
        let id = model
            .mesh
            .material_id
            .expect("No material in texture; abort!");
        // Repeat single-color material
        let mut colors: Vec<f32> = materials[id].diffuse.to_vec();
        println!("{:?}", colors);
        colors.push(1.0);
        colors = colors.repeat(model.mesh.indices.len());
        VAO::new(
            gl,
            &model.mesh.positions,
            &model.mesh.normals,
            &model.mesh.texcoords,
            &colors,
            &model.mesh.indices,
            materials[id].shininess,
        )
    }

    /// Creates a square to render arbitrary shaders on
    pub unsafe fn square(gl: &glow::Context) -> VAO {
        VAO::new(
            gl,
            &vec![-1., -1., 0., 1., -1., 0., 1., 1., 0., -1., 1., 0.],
            &vec![0., 0., -1.].repeat(4),
            &vec![0., 0., 1., 0., 1., 1., 0., 1.],
            &vec![1., 1., 1., 1.].repeat(4), // Color is irrelevant here
            &vec![0, 1, 2, 0, 2, 3],
            32.,
        )
    }

    pub unsafe fn destroy(&self, gl: &glow::Context) {
        gl.delete_vertex_array(self.vao);
        gl.delete_buffer(self.vertex_buffer);
        gl.delete_buffer(self.normal_buffer);
        gl.delete_buffer(self.uv_buffer);
        gl.delete_buffer(self.color_buffer);
    }
}
