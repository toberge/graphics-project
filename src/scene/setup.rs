use super::graph::{Node, NodeType, SceneGraph};
use super::vao::{load_obj, VAO};
use glm;

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
            32.,
        )
    };
    let (models, materials) = load_obj("res/models/crt.obj");
    let crt_vao = unsafe { VAO::from_mesh(&gl, &models[0], &materials) };

    let (goose_models, goose_materials) = load_obj("res/models/goose.obj");
    let goose_body_vao = unsafe { VAO::from_mesh(&gl, &goose_models[0], &goose_materials) };
    let goose_beak_vao = unsafe { VAO::from_mesh(&gl, &goose_models[1], &goose_materials) };
    let goose_eyes_vao = unsafe { VAO::from_mesh(&gl, &goose_models[2], &goose_materials) };

    // Create scene graph
    let mut scene_graph = SceneGraph::new();
    let mut test_node = Node::new(NodeType::Geometry);
    test_node.vao = Some(test_vao);
    test_node.position = glm::vec3(0., 0., -3.);
    let mut crt_node = Node::new(NodeType::Geometry);
    crt_node.vao = Some(crt_vao);
    let mut goose_node = Node::new(NodeType::Geometry);
    let mut goose_beak_node = Node::new(NodeType::Geometry);
    let mut goose_eyes_node = Node::new(NodeType::Geometry);
    goose_node.vao = Some(goose_body_vao);
    goose_beak_node.vao = Some(goose_beak_vao);
    goose_eyes_node.vao = Some(goose_eyes_vao);

    scene_graph.add_child(0, test_node);
    scene_graph.add_child(0, crt_node);
    scene_graph.add_child(0, goose_node);
    scene_graph.add_child(3, goose_beak_node);
    scene_graph.add_child(3, goose_eyes_node);

    scene_graph
}
