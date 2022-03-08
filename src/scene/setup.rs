use std::f32::consts::PI;

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
    let (screen_models, screen_materials) = load_obj("res/models/crt_screen.obj");
    let screen_vao = unsafe { VAO::from_mesh(&gl, &screen_models[0], &screen_materials) };
    let square_vao = unsafe { VAO::square(&gl) };

    let (goose_models, goose_materials) = load_obj("res/models/goose.obj");
    let goose_body_vao = unsafe { VAO::from_mesh(&gl, &goose_models[0], &goose_materials) };
    let goose_beak_vao = unsafe { VAO::from_mesh(&gl, &goose_models[1], &goose_materials) };
    let goose_eyes_vao = unsafe { VAO::from_mesh(&gl, &goose_models[2], &goose_materials) };

    let (chair_models, chair_materials) = load_obj("res/models/chair.obj");
    let chair_vaos = unsafe {
        chair_models
            .iter()
            .map(|model| VAO::from_mesh(&gl, &model, &chair_materials))
    };

    // Create scene graph
    let mut scene_graph = SceneGraph::new();
    let mut test_node = Node::new(NodeType::Screen);
    test_node.vao = Some(test_vao);
    test_node.position = glm::vec3(0., 0., 20.);

    let mut crt_node = Node::new(NodeType::Geometry);
    crt_node.vao = Some(crt_vao);
    crt_node.position.z = 1.;
    let mut screen_node = Node::new(NodeType::Screen);
    screen_node.vao = Some(screen_vao);

    let mut crt_node2 = Node::new(NodeType::Geometry);
    crt_node2.vao = Some(crt_vao);
    crt_node2.rotation.y = PI / 2.;
    crt_node2.position.x = 1.;
    let mut screen_node2 = Node::new(NodeType::Screen);
    screen_node2.vao = Some(screen_vao);

    let mut square_node = Node::new(NodeType::Screen);
    square_node.vao = Some(square_vao);
    square_node.position.z -= 4.;
    square_node.position.y -= 4.;

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
    scene_graph.add_child(2, screen_node);
    scene_graph.add_child(0, square_node);
    scene_graph.add_child(0, crt_node2);
    scene_graph.add_child(8, screen_node2);

    for (x, y) in vec![
        (0., 5.),
        (5., 0.),
        (5., 5.),
        (5., -5.),
        (-5., -5.),
        (-5., 5.),
        (-5., 0.),
        (0., -5.),
    ] {
        let mut chair_node = Node::new(NodeType::Geometry);
        chair_node.position.x = x;
        chair_node.position.y = -0.5;
        chair_node.position.z = y;
        let chair_index = scene_graph.add_child(0, chair_node);
        // TODO avoid clone()
        chair_vaos.clone().for_each(|vao| {
            let mut part_node = Node::new(NodeType::Geometry);
            part_node.vao = Some(vao);
            scene_graph.add_child(chair_index, part_node);
        });
    }

    scene_graph
}
