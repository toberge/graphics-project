extern crate nalgebra_glm as glm;
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
    texture: Option<NativeTexture>,

    position: glm::Vec3,
    reference_point: glm::Vec3,
    rotation: glm::Vec3,
    scale: glm::Vec3,

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

    // TODO render_reflections and render_textures?

    pub unsafe fn render(&self, gl: &glow::Context, node_index: usize) {
        let node = &self.nodes[node_index];
        if let Some(vao) = &node.vao {
            // TODO uniforms
            // TODO texture :))))
            vao.draw(gl);
        }

        // Recurse
        for child in node.children.to_vec() {
            self.render(gl, child);
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