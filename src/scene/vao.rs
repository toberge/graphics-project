use glow::*;
use tobj;

#[derive(Clone, Copy)]
/// Holds all information necessary to draw an initialized VAO.
pub struct VAO {
    pub vao: NativeVertexArray,
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
        create_buffer(&gl, 0, 3, vertices);
        create_buffer(&gl, 1, 3, normals);
        create_buffer(&gl, 2, 2, uvs);
        create_buffer(&gl, 3, 4, color);

        // Compute tangent and bitangent vectors
        let mut tangents: Vec<glm::Vec3> = vec![];
        let mut bitangents: Vec<glm::Vec3> = vec![];
        for i in (0..indices.len()).step_by(3) {
            // Same procedure as in the linked tutorial (for assignment 2)
            let vertex0 = glm::vec3(
                vertices[(indices[i] * 3) as usize],
                vertices[(indices[i] * 3 + 1) as usize],
                vertices[(indices[i] * 3 + 2) as usize],
            );
            let vertex1 = glm::vec3(
                vertices[(indices[i + 1] * 3) as usize],
                vertices[(indices[i + 1] * 3 + 1) as usize],
                vertices[(indices[i + 1] * 3 + 2) as usize],
            );
            let vertex2 = glm::vec3(
                vertices[(indices[i + 2] * 3) as usize],
                vertices[(indices[i + 2] * 3 + 1) as usize],
                vertices[(indices[i + 2] * 3 + 2) as usize],
            );
            let uv0 = glm::vec2(
                uvs[(indices[i] * 2) as usize],
                uvs[(indices[i] * 2 + 1) as usize],
            );
            let uv1 = glm::vec2(
                uvs[(indices[i + 1] * 2) as usize],
                uvs[(indices[i + 1] * 2 + 1) as usize],
            );
            let uv2 = glm::vec2(
                uvs[(indices[i + 2] * 2) as usize],
                uvs[(indices[i + 2] * 2 + 1) as usize],
            );
            let delta_pos_1 = vertex1 - vertex0;
            let delta_pos_2 = vertex2 - vertex0;
            let delta_uv_1 = uv1 - uv0;
            let delta_uv_2 = uv2 - uv0;
            let r = 1.0 / (delta_uv_1.x * delta_uv_2.y - delta_uv_1.y * delta_uv_2.x);
            let tangent = (delta_pos_1 * delta_uv_2.y - delta_pos_2 * delta_uv_1.y) * r;
            let bitangent = (delta_pos_2 * delta_uv_1.x - delta_pos_1 * delta_uv_2.x) * r;
            tangents.push(tangent);
            tangents.push(tangent);
            tangents.push(tangent);
            bitangents.push(bitangent);
            bitangents.push(bitangent);
            bitangents.push(bitangent);
        }
        // Assign them as attributes
        create_buffer(
            &gl,
            4,
            3,
            &tangents.iter().flat_map(|&t| [t.x, t.y, t.z]).collect(),
        );
        create_buffer(
            &gl,
            5,
            3,
            &bitangents.iter().flat_map(|&t| [t.x, t.y, t.z]).collect(),
        );

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
            &vec![1., 0., 1., 1.].repeat(4), // Color is irrelevant here
            &vec![0, 1, 2, 0, 2, 3],
            32.,
        )
    }
}
