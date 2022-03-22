use std::f32::consts::PI;

use crate::shader;

use super::graph::{Node, NodeType, SceneGraph};
use super::texture::Texture;
use super::vao::{load_obj, VAO};

pub fn create_scene(gl: &glow::Context) -> SceneGraph {
    // Create VAOs
    let (models, materials) = load_obj("res/models/crt.obj");
    let crt_vao = unsafe { VAO::from_mesh(&gl, &models[0], &materials) };
    let screen_vao = unsafe { VAO::from_mesh(&gl, &models[1], &materials) };

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
    let (cube_models, cube_materials) = load_obj("res/models/cube.obj");
    let cube_vaos: Vec<VAO> = unsafe {
        cube_models
            .iter()
            .map(|model| VAO::from_mesh(&gl, &model, &cube_materials))
            .collect()
    };

    // Create scene graph
    let mut scene_graph = SceneGraph::new();

    // TODO the order of these matters for what is rendered :)))
    let crt_texture4 = unsafe { Texture::framebuffer_texture(&gl, 400, 400) };
    let crt_texture3 = unsafe { Texture::cubemap_texture(&gl, 400, 400) };
    let crt_texture2 = unsafe { Texture::cubemap_texture(&gl, 400, 400) };
    let crt_texture = unsafe { Texture::cubemap_texture(&gl, 400, 400) };
    let mut crt_node = Node::new(NodeType::Geometry);
    crt_node.vao = Some(crt_vao);
    crt_node.scale = glm::vec3(4.5, 4.5, 4.5);
    crt_node.position.z = 2.5;
    let mut screen_node = Node::new(NodeType::Screen);
    screen_node.vao = Some(screen_vao);
    screen_node.reflection_texture = Some(crt_texture);

    let mut crt_node2 = Node::new(NodeType::Geometry);
    crt_node2.vao = Some(crt_vao);
    crt_node2.rotation.y = PI / 2.;
    crt_node2.position.x = 2.5;
    crt_node2.scale = glm::vec3(2., 2., 2.);
    let mut screen_node2 = Node::new(NodeType::Screen);
    screen_node2.vao = Some(screen_vao);
    screen_node2.reflection_texture = Some(crt_texture2);

    let mut crt_node3 = Node::new(NodeType::Geometry);
    crt_node3.vao = Some(crt_vao);
    crt_node3.rotation.y = PI;
    crt_node3.position.z = -2.5;
    crt_node3.scale = glm::vec3(2., 2., 2.);
    let mut screen_node3 = Node::new(NodeType::Screen);
    screen_node3.vao = Some(screen_vao);
    screen_node3.reflection_texture = Some(crt_texture3);

    let mut crt_node4 = Node::new(NodeType::Geometry);
    crt_node4.vao = Some(crt_vao);
    crt_node4.rotation.y = -PI / 2.;
    crt_node4.position.x = -2.5;
    crt_node4.scale = glm::vec3(2., 2., 2.);
    let mut screen_node4 = Node::new(NodeType::Geometry);
    screen_node4.vao = Some(screen_vao);
    screen_node4.texture = Some(crt_texture4);

    let mut goose_node = Node::new(NodeType::Geometry);
    let mut goose_beak_node = Node::new(NodeType::Geometry);
    let mut goose_eyes_node = Node::new(NodeType::Geometry);
    goose_node.vao = Some(goose_body_vao);
    goose_beak_node.vao = Some(goose_beak_vao);
    goose_eyes_node.vao = Some(goose_eyes_vao);

    let single_color_shader = unsafe {
        shader::Shader::new(
            &gl,
            "res/shaders/single_color.vert",
            "res/shaders/single_color.frag",
        )
    };
    scene_graph.screen_shaders = vec![(single_color_shader.program, crt_texture4)];

    scene_graph.add_child(0, crt_node);
    scene_graph.add_child(0, goose_node);
    scene_graph.add_child(2, goose_beak_node);
    scene_graph.add_child(2, goose_eyes_node);
    scene_graph.add_child(1, screen_node);
    scene_graph.add_child(0, crt_node2);
    scene_graph.add_child(6, screen_node2);
    scene_graph.add_child(0, crt_node3);
    scene_graph.add_child(8, screen_node3);
    scene_graph.add_child(0, crt_node4);
    scene_graph.add_child(10, screen_node4);

    for (i, (x, y)) in vec![
        (0., 5.),
        (5., 0.),
        (5., 5.),
        (5., -5.),
        (-5., -5.),
        (-5., 5.),
        (-5., 0.),
        (0., -5.),
    ]
    .iter()
    .enumerate()
    {
        let mut chair_node = Node::new(NodeType::Geometry);
        chair_node.vao = Some(cube_vaos[i]);
        chair_node.scale = glm::vec3(4., 4., 4.);
        chair_node.position.x = x * 8.;
        chair_node.position.y = -0.5;
        chair_node.position.z = y * 4.;
        let chair_index = scene_graph.add_child(0, chair_node);
        // TODO avoid clone()
        //chair_vaos.clone().for_each(|vao| {
        //    let mut part_node = Node::new(NodeType::Geometry);
        //    part_node.vao = Some(vao);
        //    scene_graph.add_child(chair_index, part_node);
        //});
    }

    scene_graph
}
