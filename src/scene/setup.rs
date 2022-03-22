use std::f32::consts::PI;

use crate::shader;

use super::graph::{Node, NodeType, SceneGraph};
use super::texture::Texture;
use super::vao::{load_obj, VAO};

pub fn create_scene(gl: &glow::Context) -> SceneGraph {
    // Create scene graph
    let mut scene_graph = SceneGraph::new();

    ///////// Room /////////

    let square_vao = unsafe { VAO::square(gl) };
    let room_size = 20.;
    for &(x, z, rot) in vec![
        (0., 1., PI),
        (0., -1., 0.),
        (1., 0., -PI / 2.),
        (-1., 0., PI / 2.),
    ]
    .iter()
    {
        let mut wall_node = Node::new(NodeType::Geometry);
        wall_node.vao = Some(square_vao);
        wall_node.scale = glm::vec3(room_size, room_size, room_size);
        wall_node.rotation.y = rot;
        wall_node.position.x = x * room_size;
        wall_node.position.y = room_size;
        wall_node.position.z = z * room_size;
        scene_graph.add_child(0, wall_node);
    }
    let mut floor_node = Node::new(NodeType::Geometry);
    floor_node.vao = Some(square_vao);
    floor_node.scale = glm::vec3(room_size, room_size, room_size);
    floor_node.rotation.x = -PI / 2.;
    scene_graph.add_child(0, floor_node);

    ///////// Screens /////////

    let (models, materials) = load_obj("res/models/crt.obj");
    let crt_vao = unsafe { VAO::from_mesh(&gl, &models[0], &materials) };
    let screen_vao = unsafe { VAO::from_mesh(&gl, &models[1], &materials) };

    let mut crt_root_node = Node::new(NodeType::Root);
    crt_root_node.position.y += 2.;
    let crt_root = scene_graph.add_child(0, crt_root_node);

    let mut crts: Vec<usize> = vec![];
    for &(position, rotation) in vec![
        (glm::vec3(0., 0., 5.), glm::vec3(0., 0., 0.)),
        (glm::vec3(4., 0., 4.), glm::vec3(0., PI / 4., 0.)),
        (glm::vec3(5., 0., 0.), glm::vec3(0., PI / 2., 0.)),
        (glm::vec3(4., 0., -4.), glm::vec3(0., 3. * PI / 4., 0.)),
        (glm::vec3(0., 0., -5.), glm::vec3(0., PI, 0.)),
        (glm::vec3(-4., 0., -4.), glm::vec3(0., -3. * PI / 4., 0.)),
        (glm::vec3(-5., 0., 0.), glm::vec3(0., -PI / 2., 0.)),
        (glm::vec3(-4., 0., 4.), glm::vec3(0., -PI / 4., 0.)),
        (glm::vec3(0., 4., 0.), glm::vec3(0., 0., 0.)),
    ]
    .iter()
    {
        let mut crt_node = Node::new(NodeType::Geometry);
        crt_node.vao = Some(crt_vao);
        crt_node.scale = glm::vec3(1.5, 1.5, 1.5);
        crt_node.rotation = rotation;
        crt_node.position = position;
        let crt_index = scene_graph.add_child(crt_root, crt_node);
        let mut screen_node = Node::new(NodeType::Screen);
        screen_node.vao = Some(screen_vao);
        screen_node.reflection_texture =
            unsafe { Some(Texture::framebuffer_texture(&gl, 200, 200)) };
        crts.push(scene_graph.add_child(crt_index, screen_node));
    }

    let mut shaders: Vec<(glow::NativeProgram, Texture)> = vec![];
    for (crt_index, shader_source) in vec![
        (2, "res/shaders/single_color.frag"),
        (0, "res/shaders/smooth.frag"),
        (3, "res/shaders/smooth2.frag"),
    ] {
        let shader = unsafe { shader::Shader::new(&gl, "res/shaders/screen.vert", shader_source) };
        let texture = unsafe { Texture::framebuffer_texture(&gl, 200, 200) };
        (*scene_graph.get_node(crts[crt_index])).texture = Some(texture);
        shaders.push((shader.program, texture));
    }
    scene_graph.screen_shaders = shaders;

    ///////// Miscellaneous interior /////////

    let (goose_models, goose_materials) = load_obj("res/models/goose.obj");
    let goose_body_vao = unsafe { VAO::from_mesh(&gl, &goose_models[0], &goose_materials) };
    let goose_beak_vao = unsafe { VAO::from_mesh(&gl, &goose_models[1], &goose_materials) };
    let goose_eyes_vao = unsafe { VAO::from_mesh(&gl, &goose_models[2], &goose_materials) };

    let mut goose_node = Node::new(NodeType::Geometry);
    let mut goose_beak_node = Node::new(NodeType::Geometry);
    let mut goose_eyes_node = Node::new(NodeType::Geometry);
    goose_node.vao = Some(goose_body_vao);
    goose_beak_node.vao = Some(goose_beak_vao);
    goose_eyes_node.vao = Some(goose_eyes_vao);

    let goose_root = scene_graph.add_child(0, goose_node);
    scene_graph.add_child(goose_root, goose_beak_node);
    scene_graph.add_child(goose_root, goose_eyes_node);

    let (cube_models, cube_materials) = load_obj("res/models/cube.obj");
    let cube_vaos: Vec<VAO> = unsafe {
        cube_models
            .iter()
            .map(|model| VAO::from_mesh(&gl, &model, &cube_materials))
            .collect()
    };

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
        chair_node.position.x = x * 4.;
        chair_node.position.y = 2.;
        chair_node.position.z = y * 4.;
        scene_graph.add_child(0, chair_node);
    }

    let (sofa_models, sofa_materials) = load_obj("res/models/sofa_03_1k.obj");
    let sofa_vao = unsafe { VAO::from_mesh(&gl, &sofa_models[0], &sofa_materials) };
    let mut sofa_node = Node::new(NodeType::Geometry);
    sofa_node.vao = Some(sofa_vao);
    sofa_node.position.z += 12.;
    sofa_node.rotation.y = PI;
    sofa_node.scale = glm::vec3(4., 4., 4.);
    scene_graph.add_child(0, sofa_node);

    for (position, color) in vec![
        (glm::vec3(10., 3., 0.), glm::vec3(0.4, 0.4, 0.4)),
        (glm::vec3(0., 2., 10.), glm::vec3(0.4, 0.4, 0.4)),
        (glm::vec3(-10., 4., 10.), glm::vec3(0.6, 0.4, 0.4)),
        (glm::vec3(0., 2., -4.), glm::vec3(0.6, 0.6, 0.6)),
    ] {
        let mut light_node = Node::new(NodeType::Light);
        light_node.position = position.clone();
        light_node.emission_color = color;
        scene_graph.add_child(0, light_node);
    }

    scene_graph
}
