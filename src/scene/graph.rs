use std::f32::consts::PI;

use glm;
use glow::*;

use super::{
    texture::{CubemapTexture, FrameBufferTexture},
    vao::VAO,
};

pub enum NodeType {
    Root,
    Geometry,
    Light,
    Screen,
}

/// Scene node
pub struct Node {
    index: usize,
    parent: Option<usize>,
    children: Vec<usize>,

    kind: NodeType,
    pub vao: Option<VAO>, // TODO problem when deleting VAO I guess :))))
    pub texture: Option<FrameBufferTexture>,
    pub normal_map: Option<FrameBufferTexture>,
    pub reflection_map: Option<FrameBufferTexture>,
    pub roughness_map: Option<FrameBufferTexture>,
    pub opacity_map: Option<FrameBufferTexture>,
    pub cubemap_texture: Option<CubemapTexture>,
    pub shader: Option<NativeShader>,
    pub emission_color: glm::Vec3,

    pub position: glm::Vec3,
    pub reference_point: glm::Vec3,
    pub rotation: glm::Vec3,
    pub total_rotation: glm::Vec3,
    pub scale: glm::Vec3,

    model_matrix: glm::Mat4,
    view_matrix: glm::Mat4,
}

/// Scene graph where the nodes are stored in a list for simplicity
pub struct SceneGraph {
    pub nodes: Vec<Node>,
    pub root: usize,
    // Light sources must have their positions sent to the shader
    pub light_sources: Vec<usize>,
    // Nodes to be treated as cameras during reflection rendering
    pub cameras: Vec<usize>,

    // Remember
    pub vaos: Vec<VAO>,

    // Scene graph needs access to shaders during rendering
    pub final_shader: Option<NativeProgram>,
    pub reflection_shader: Option<NativeProgram>,
    pub screen_shaders: Vec<(NativeProgram, FrameBufferTexture)>,
}

impl Node {
    pub fn new(kind: NodeType) -> Node {
        Node {
            index: 0,
            parent: None,
            children: vec![],
            kind,
            vao: None,
            texture: None,
            normal_map: None,
            roughness_map: None,
            opacity_map: None,
            reflection_map: None,
            cubemap_texture: None,
            shader: None,
            emission_color: glm::zero(),
            position: glm::zero(),
            reference_point: glm::zero(),
            rotation: glm::zero(),
            total_rotation: glm::zero(),
            scale: glm::vec3(1., 1., 1.),
            model_matrix: glm::identity(),
            view_matrix: glm::identity(),
        }
    }

    pub fn add_child(&mut self, index: usize) {
        self.children.push(index);
    }
}

impl SceneGraph {
    pub fn new() -> SceneGraph {
        SceneGraph {
            nodes: vec![Node::new(NodeType::Root)],
            root: 0,
            light_sources: vec![],
            cameras: vec![],
            vaos: vec![],
            final_shader: None,
            reflection_shader: None,
            screen_shaders: vec![],
        }
    }

    /// Add a child node and remember it especially well if it is a light source or screen
    pub fn add_child(&mut self, parent_index: usize, child: Node) -> usize {
        let child_index = self.nodes.len();
        match child.kind {
            NodeType::Light => self.light_sources.push(child_index),
            NodeType::Screen => self.cameras.push(child_index),
            _ => {}
        }
        self.nodes.push(child);
        self.nodes[parent_index].add_child(child_index);
        child_index
    }

    pub fn get_node(&mut self, node_index: usize) -> &mut Node {
        &mut self.nodes[node_index]
    }

    pub fn update(&mut self, gl: &glow::Context) {
        self.update_transformations(self.root, &glm::identity(), &glm::zero());

        unsafe {
            gl.use_program(self.final_shader);
            gl.uniform_1_u32(
                gl.get_uniform_location(self.final_shader.unwrap(), "num_light_sources")
                    .as_ref(),
                self.light_sources.len() as u32,
            );
            for (i, &light_index) in self.light_sources.clone().iter().enumerate() {
                let light = &self.nodes[light_index];
                gl.uniform_3_f32_slice(
                    gl.get_uniform_location(
                        self.final_shader.unwrap(),
                        &format!("light_sources[{}].position", i),
                    )
                    .as_ref(),
                    &light.position.as_slice(),
                );
                gl.uniform_3_f32_slice(
                    gl.get_uniform_location(
                        self.final_shader.unwrap(),
                        &format!("light_sources[{}].color", i),
                    )
                    .as_ref(),
                    &light.emission_color.as_slice(),
                );
            }
        }
    }

    /// Update transformation matrices for the whole tree
    pub fn update_transformations(
        &mut self,
        node_index: usize,
        transformation_so_far: &glm::Mat4,
        rotation_so_far: &glm::Vec3,
    ) {
        let mut node = &mut (self.nodes[node_index]);
        // Construct transformation matrix
        let mut mat: glm::Mat4 = glm::identity();
        // Scale and rotate in terms of the reference point
        mat = glm::translation(&-node.reference_point) * mat;
        mat = glm::scaling(&node.scale) * mat;
        // TODO consider quaternion here maaaaybe?
        mat = glm::rotation(node.rotation.z, &glm::vec3(0.0, 0.0, 1.0)) * mat;
        mat = glm::rotation(node.rotation.y, &glm::vec3(0.0, 1.0, 0.0)) * mat;
        mat = glm::rotation(node.rotation.x, &glm::vec3(1.0, 0.0, 0.0)) * mat;
        mat = glm::translation(&node.reference_point) * mat;
        // Translate to position
        mat = glm::translation(&node.position) * mat;
        mat = transformation_so_far * mat;

        // Then update the node's matrix
        node.model_matrix = mat;
        // And update the node's total rotation
        let rotation = rotation_so_far + node.rotation;
        node.total_rotation = rotation;

        // Recurse
        for child in node.children.to_vec() {
            self.update_transformations(child, &mat, &rotation);
        }
    }

    pub unsafe fn render_screens(&self, gl: &glow::Context, time: f32) {
        let canvas = VAO::square(gl);
        for (shader, texture) in self.screen_shaders.clone() {
            gl.bind_framebuffer(glow::FRAMEBUFFER, texture.framebuffer);
            gl.viewport(0, 0, texture.width, texture.height);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            gl.use_program(Some(shader));
            gl.uniform_1_f32(gl.get_uniform_location(shader, "time").as_ref(), time);
            gl.uniform_2_f32_slice(
                gl.get_uniform_location(shader, "screen_size").as_ref(),
                glm::vec2(texture.width as f32, texture.height as f32).as_slice(),
            );
            canvas.draw(&gl);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }

    pub unsafe fn render_reflections(&self, gl: &glow::Context) {
        for node_index in self.cameras.clone() {
            let texture = self.nodes[node_index].reflection_map.expect(&format!(
                "Node {} was not assigned reflection texture",
                node_index
            ));
            gl.bind_framebuffer(glow::FRAMEBUFFER, texture.framebuffer);
            gl.viewport(0, 0, texture.width, texture.height);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            gl.use_program(self.final_shader);
            self.render_in_terms_of(&gl, node_index);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }

    /// Render scene tree from the persepective of one particular node
    pub unsafe fn render_in_terms_of(&self, gl: &glow::Context, node_index: usize) {
        let node = &self.nodes[node_index];

        let perspective: glm::Mat4 = glm::perspective(1., PI / 4., 1.0, 100.0);
        let camera_position: glm::Vec3 =
            glm::vec4_to_vec3(&(node.model_matrix * glm::vec4(0., 0., 0., 1.)));
        let mut rotation: glm::Mat4 = glm::mat3_to_mat4(&glm::mat4_to_mat3(&glm::inverse(
            &glm::transpose(&node.model_matrix),
        )))
        .normalize();
        rotation = glm::scale(&rotation, &glm::vec3(1., 1., -1.));
        // flip winding order :)))))
        // the ultimate hack
        gl.front_face(glow::CW);
        let camera_transform = perspective * rotation * glm::translation(&-camera_position);

        self.render(gl, self.root, &camera_transform, &camera_position, false);
        gl.front_face(glow::CCW);
    }

    pub unsafe fn render_cubemap_reflections(&self, gl: &glow::Context) {
        for node_index in self.cameras.clone() {
            if let Some(texture) = self.nodes[node_index].cubemap_texture {
                gl.use_program(self.final_shader);
                for (i, &(center, up)) in [
                    (glm::vec3(1., 0., 0.), glm::vec3(0., 1., 0.)),  // +X
                    (glm::vec3(-1., 0., 0.), glm::vec3(0., 1., 0.)), // -X
                    (glm::vec3(0., 1., 0.), glm::vec3(1., 0., 0.)),  // +Y
                    (glm::vec3(0., -1., 0.), glm::vec3(1., 0., 0.)), // -Y
                    (glm::vec3(0., 0., 1.), glm::vec3(0., 1., 0.)),  // +Z
                    (glm::vec3(0., 0., -1.), glm::vec3(0., 1., 0.)), // -Z
                ]
                .iter()
                .enumerate()
                {
                    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(texture.framebuffers[i]));
                    gl.viewport(0, 0, texture.size, texture.size);
                    gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                    self.render_in_terms_of_with_lookat(&gl, node_index, &center, &up);
                }
                gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            }
        }
    }

    /// Render scene tree from the persepective of one particular node
    pub unsafe fn render_in_terms_of_with_lookat(
        &self,
        gl: &glow::Context,
        node_index: usize,
        center: &glm::Vec3,
        up: &glm::Vec3,
    ) {
        let node = &self.nodes[node_index];

        let perspective: glm::Mat4 = glm::perspective(1., PI / 2., 4.0, 1000.0);
        let camera_position: glm::Vec3 =
            glm::vec4_to_vec3(&(node.model_matrix * glm::vec4(0., 0., 0., 1.)));
        let mat = glm::mat4_to_mat3(&glm::transpose(&glm::inverse(&node.model_matrix))).normalize();

        let camera_transform = perspective
            * glm::look_at(
                &camera_position,
                &(camera_position + glm::normalize(&(mat * center))),
                &glm::normalize(&(mat * up)),
            );

        self.render(gl, self.root, &camera_transform, &glm::zero(), false);
    }

    pub unsafe fn render(
        &self,
        gl: &glow::Context,
        node_index: usize,
        view_transform: &glm::Mat4,
        camera_position: &glm::Vec3,
        with_reflection: bool,
    ) {
        let node = &self.nodes[node_index];
        if let Some(vao) = &node.vao {
            // Set uniforms (a lot of them)
            gl.uniform_matrix_4_f32_slice(
                gl.get_uniform_location(self.final_shader.unwrap(), "model_transform")
                    .as_ref(),
                false,
                node.model_matrix.as_slice(),
            );
            gl.uniform_matrix_4_f32_slice(
                gl.get_uniform_location(self.final_shader.unwrap(), "view_transform")
                    .as_ref(),
                false,
                (view_transform * node.model_matrix).as_slice(),
            );
            gl.uniform_matrix_3_f32_slice(
                gl.get_uniform_location(self.final_shader.unwrap(), "normal_transform")
                    .as_ref(),
                false,
                // Normal restoration matrix from earlier
                &glm::mat4_to_mat3(&glm::transpose(&glm::inverse(&node.model_matrix))).as_slice(),
            );
            gl.uniform_3_f32_slice(
                gl.get_uniform_location(self.final_shader.unwrap(), "camera_position")
                    .as_ref(),
                &camera_position.as_slice(),
            );
            gl.uniform_1_f32(
                gl.get_uniform_location(self.final_shader.unwrap(), "shininess")
                    .as_ref(),
                vao.shininess,
            );

            // Bind texture if one exists, and indicate whether the model has a texture or not
            if let Some(texture) = node.texture {
                gl.uniform_1_i32(
                    gl.get_uniform_location(self.final_shader.unwrap(), "use_texture")
                        .as_ref(),
                    1,
                );
                gl.active_texture(glow::TEXTURE0);
                gl.bind_texture(glow::TEXTURE_2D, Some(texture.texture));
            } else {
                gl.uniform_1_i32(
                    gl.get_uniform_location(self.final_shader.unwrap(), "use_texture")
                        .as_ref(),
                    0,
                );
            }

            // Normal map
            if let Some(texture) = node.normal_map {
                gl.uniform_1_i32(
                    gl.get_uniform_location(self.final_shader.unwrap(), "use_normals")
                        .as_ref(),
                    1,
                );
                gl.active_texture(glow::TEXTURE2);
                gl.bind_texture(glow::TEXTURE_2D, Some(texture.texture));
            } else {
                gl.uniform_1_i32(
                    gl.get_uniform_location(self.final_shader.unwrap(), "use_normals")
                        .as_ref(),
                    0,
                );
            }

            // Roughness map
            if let Some(texture) = node.roughness_map {
                gl.uniform_1_i32(
                    gl.get_uniform_location(self.final_shader.unwrap(), "use_roughness")
                        .as_ref(),
                    1,
                );
                gl.active_texture(glow::TEXTURE3);
                gl.bind_texture(glow::TEXTURE_2D, Some(texture.texture));
            } else {
                gl.uniform_1_i32(
                    gl.get_uniform_location(self.final_shader.unwrap(), "use_roughness")
                        .as_ref(),
                    0,
                );
            }

            // Opacity map
            if let Some(texture) = node.opacity_map {
                gl.uniform_1_i32(
                    gl.get_uniform_location(self.final_shader.unwrap(), "use_opacity")
                        .as_ref(),
                    1,
                );
                gl.active_texture(glow::TEXTURE4);
                gl.bind_texture(glow::TEXTURE_2D, Some(texture.texture));
            } else {
                gl.uniform_1_i32(
                    gl.get_uniform_location(self.final_shader.unwrap(), "use_opacity")
                        .as_ref(),
                    0,
                );
            }

            // Reflection texture
            if with_reflection {
                if let Some(reflection) = node.reflection_map {
                    gl.uniform_1_i32(
                        gl.get_uniform_location(self.final_shader.unwrap(), "use_reflection")
                            .as_ref(),
                        1,
                    );
                    gl.active_texture(glow::TEXTURE1);
                    gl.bind_texture(glow::TEXTURE_2D, Some(reflection.texture));
                } else {
                    gl.uniform_1_i32(
                        gl.get_uniform_location(self.final_shader.unwrap(), "use_reflection")
                            .as_ref(),
                        0,
                    );
                }
            } else {
                gl.uniform_1_i32(
                    gl.get_uniform_location(self.final_shader.unwrap(), "use_reflection")
                        .as_ref(),
                    0,
                );
            }

            // Then draw the VAO
            vao.draw(gl);
        }

        // Recurse
        for child in node.children.to_vec() {
            self.render(gl, child, view_transform, camera_position, with_reflection);
        }
    }

    pub unsafe fn teardown(&mut self, gl: &glow::Context) {
        // Clean up VAOs
        self.vaos.iter().for_each(|vao| vao.destroy(gl));
        // Clean up shaders
        self.final_shader
            .map_or_else(|| return, |shader| gl.delete_program(shader));
        self.reflection_shader
            .map_or_else(|| return, |shader| gl.delete_program(shader));
        self.screen_shaders
            .iter()
            .for_each(|&(shader, _)| gl.delete_program(shader));
    }
}
