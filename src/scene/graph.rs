use glm;
use glow::*;

use super::vao::VAO;

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
    pub texture: Option<NativeTexture>,

    pub position: glm::Vec3,
    pub reference_point: glm::Vec3,
    pub rotation: glm::Vec3,
    pub scale: glm::Vec3,

    model_matrix: glm::Mat4,
    view_matrix: glm::Mat4,
}

/// Scene graph where the nodes are stored in a list for simplicity
pub struct SceneGraph {
    pub nodes: Vec<Node>,
    pub root: usize,
    // Nodes to be treated as cameras during reflection rendering
    pub cameras: Vec<usize>,

    // Remember
    pub vaos: Vec<VAO>,

    // Scene graph needs access to shaders during rendering
    pub final_shader: Option<NativeProgram>,
    pub reflection_shader: Option<NativeProgram>,
    pub screen_shaders: Vec<NativeProgram>,
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
            position: glm::zero(),
            reference_point: glm::zero(),
            rotation: glm::zero(),
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
            cameras: vec![],
            vaos: vec![],
            final_shader: None,
            reflection_shader: None,
            screen_shaders: vec![],
        }
    }

    pub fn add_child(&mut self, parent_index: usize, child: Node) {
        self.nodes.push(child);
        let child_index = self.nodes.len() - 1;
        self.nodes[parent_index].add_child(child_index);
    }

    pub fn update_transformations(&mut self, node_index: usize, transformation_so_far: &glm::Mat4) {
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

        // Recurse
        for child in node.children.to_vec() {
            self.update_transformations(child, &mat);
        }
    }

    // TODO render_reflections and render_screens?

    pub unsafe fn render(
        &self,
        gl: &glow::Context,
        node_index: usize,
        view_transform: &glm::Mat4,
        camera_position: &glm::Vec3,
    ) {
        let node = &self.nodes[node_index];
        if let Some(vao) = &node.vao {
            // Test uniform loc:
            //match gl.get_uniform_location(self.final_shader.unwrap(), "model_transform") {
            //    Some(_) => println!("yay"),
            //    None => println!("nay"),
            //}

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
                gl.bind_texture(glow::TEXTURE0, Some(texture));
                gl.active_texture(glow::TEXTURE0);
            } else {
                gl.uniform_1_i32(
                    gl.get_uniform_location(self.final_shader.unwrap(), "use_texture")
                        .as_ref(),
                    0,
                );
            }

            // Then draw the VAO
            vao.draw(gl);
        }

        // Recurse
        for child in node.children.to_vec() {
            self.render(gl, child, view_transform, camera_position);
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
            .for_each(|&shader| gl.delete_program(shader));
    }
}
