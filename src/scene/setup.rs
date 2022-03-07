use super::graph::{Node, NodeType, SceneGraph};
use super::vao::{load_obj, VAO};

pub fn create_scene(gl: &glow::Context) -> SceneGraph {
    // Create VAOs
    let test_vao = unsafe {
        VAO::new(
            &gl,
            &vec![0.7, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            &vec![0.0, 0.0, 1.0, 0.0].repeat(3),
            &vec![0., 1., 0., 0., 1., 0.],
            &vec![0.0, 1.0, 0.0, 1.0].repeat(3),
            &vec![0, 1, 2],
        )
    };
    let (models, materials) = load_obj("res/models/crt.obj");
    let crt_vao = unsafe { VAO::from_mesh(&gl, &models[0], &materials) };

    // Create scene graph
    let mut scene_graph = SceneGraph::new();
    let mut test_node = Node::new(NodeType::Geometry);
    test_node.vao = Some(test_vao);
    let mut crt_node = Node::new(NodeType::Geometry);
    crt_node.vao = Some(crt_vao);
    scene_graph.add_child(0, test_node);
    scene_graph.add_child(0, crt_node);

    scene_graph
}
