use std::f32::consts::PI;
use std::time::Instant;

use crate::shader;

use super::graph::{Node, NodeType, SceneGraph};
use super::texture::{CubemapTexture, FrameBufferTexture, ImageTexture};
use super::vao::{load_obj, VAO};

const SIMPLE: bool = true;

pub fn create_scene(gl: &glow::Context) -> SceneGraph {
    // Create scene graph
    let mut scene_graph = SceneGraph::new();

    ///////// Room /////////

    // Just a floor piece, and everything fades to darkness around it
    let before_floor = Instant::now();
    let square_vao = unsafe { VAO::square(gl) };
    let room_size = 30.;
    let mut floor_node = Node::new(NodeType::Geometry);
    floor_node.vao = Some(square_vao);
    floor_node.scale = glm::vec3(room_size, room_size, room_size);
    floor_node.rotation.x = -PI / 2.;
    if !SIMPLE {
        floor_node.texture = unsafe {
            Some(ImageTexture::new(
                gl,
                "res/textures/weathered_brown_planks_diff_4k.jpg",
            ))
        };
        floor_node.normal_map = unsafe {
            Some(ImageTexture::new(
                gl,
                "res/textures/weathered_brown_planks_nor_gl_4k.jpg",
            ))
        };
        floor_node.roughness_map = unsafe {
            Some(ImageTexture::new(
                gl,
                "res/textures/weathered_brown_planks_rough_4k.jpg",
            ))
        };
    }
    scene_graph.add_child(0, floor_node);
    println!(
        "Loading models took {} seconds",
        Instant::now().duration_since(before_floor).as_secs_f32(),
    );

    ///////// Screens /////////

    let (models, materials) = load_obj("res/models/crt.obj");
    let crt_vao = unsafe { VAO::from_mesh(&gl, &models[0], &materials) };
    let screen_vao = unsafe { VAO::from_mesh(&gl, &models[1], &materials) };

    let mut crt_root_node = Node::new(NodeType::Root);
    crt_root_node.position.y += 2.;
    let crt_root = scene_graph.add_child(0, crt_root_node);

    // 8 screens in a circle at the bottom, some piling up from there
    let mut crts: Vec<usize> = vec![];
    let radius = 5.;
    let radius2 = 4.2;
    let leg = (radius * radius / 2. as f32).sqrt();
    let leg2 = (radius2 * radius2 / 2. as f32).sqrt();
    for &(position, rotation) in vec![
        (glm::vec3(0., 0., radius), glm::vec3(0., PI, 0.)),
        (glm::vec3(leg, 0., leg), glm::vec3(0., 5. * PI / 4., 0.)),
        (glm::vec3(radius, 0., 0.), glm::vec3(0., 3. * PI / 2., 0.)),
        (glm::vec3(leg, 0., -leg), glm::vec3(0., 7. * PI / 4., 0.)),
        (glm::vec3(0., 0., -radius), glm::vec3(0., 0., 0.)),
        (glm::vec3(-leg, 0., -leg), glm::vec3(0., PI / 4., 0.)),
        (glm::vec3(-radius, 0., 0.), glm::vec3(0., PI / 2., 0.)),
        (glm::vec3(-leg, 0., leg), glm::vec3(0., 3. * PI / 4., 0.)),
        (glm::vec3(0., 3., radius2), glm::vec3(PI / 6., PI, 0.)),
        (
            glm::vec3(leg2, 3., leg2),
            glm::vec3(PI / 6., 5. * PI / 4., 0.),
        ),
        (glm::vec3(-radius2, 3., 0.), glm::vec3(PI / 6., PI / 2., 0.)),
        (
            glm::vec3(leg2, 3., -leg2),
            glm::vec3(PI / 6., 7. * PI / 4., 0.),
        ),
        (glm::vec3(0., 3., -radius2), glm::vec3(PI / 6., 0., 0.)),
        (glm::vec3(-leg2, 3., -leg2), glm::vec3(PI / 6., PI / 4., 0.)),
        (glm::vec3(radius2, 3., 0.), glm::vec3(PI / 6., -PI / 2., 0.)),
        (
            glm::vec3(-leg2, 3., leg2),
            glm::vec3(PI / 6., 3. * PI / 4., 0.),
        ),
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
        screen_node.cubemap_texture = unsafe { Some(CubemapTexture::new(&gl, 400)) };
        crts.push(scene_graph.add_child(crt_index, screen_node));
    }

    let mut shaders: Vec<(glow::NativeProgram, usize)> = vec![];
    for (crt_index, shader_source) in vec![
        (0, "res/shaders/smooth.frag"),
        (1, "res/shaders/gyroid.frag"),
        (3, "res/shaders/smooth2.frag"),
        (5, "res/shaders/bloom.frag"),
        (6, "res/shaders/shadow.frag"),
        (7, "res/shaders/ripples.frag"),
        (8, "res/shaders/uv.frag"),
    ] {
        let shader = unsafe { shader::Shader::new(&gl, "res/shaders/screen.vert", shader_source) };
        shaders.push((shader.program, crts[crt_index]));
    }
    scene_graph.screen_shaders = shaders;

    ///////// Miscellaneous interior /////////

    let (goose_models, goose_materials) = load_obj("res/models/goose.obj");
    let goose_body_vao = unsafe { VAO::from_mesh(&gl, &goose_models[0], &goose_materials) };
    let goose_beak_vao = unsafe { VAO::from_mesh(&gl, &goose_models[1], &goose_materials) };
    let goose_eyes_vao = unsafe { VAO::from_mesh(&gl, &goose_models[2], &goose_materials) };

    let mut goose_node = Node::new(NodeType::Screen);
    let mut goose_beak_node = Node::new(NodeType::Geometry);
    let mut goose_eyes_node = Node::new(NodeType::Geometry);
    goose_node.vao = Some(goose_body_vao);
    goose_node.cubemap_texture = unsafe { Some(CubemapTexture::new(&gl, 400)) };
    goose_beak_node.vao = Some(goose_beak_vao);
    goose_eyes_node.vao = Some(goose_eyes_vao);
    goose_node.rotation.y = PI;

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

    if SIMPLE {
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
    }

    if !SIMPLE {
        let before = Instant::now();
        for (objname, name, position, rotation, scale) in vec![
            // +Z: Sofa
            (
                "sofa_03",
                "sofa_03",
                glm::vec3(0., 0., 12.),
                glm::vec3(0., PI, 0.),
                glm::vec3(4., 4., 4.),
            ),
            // +X: Cabinet and table
            (
                "vintage_cabinet_01",
                "vintage_cabinet_01_a",
                glm::vec3(17., 0., 0.),
                glm::vec3(0., -PI / 2., 0.),
                glm::vec3(4., 4., 4.),
            ),
            (
                "round_wooden_table_01",
                "round_wooden_table_01",
                glm::vec3(12., 0., 0.),
                glm::vec3(0., -PI / 2., 0.),
                glm::vec3(4., 4., 4.),
            ),
            (
                "modern_ceiling_lamp_01",
                "modern_ceiling_lamp_01",
                glm::vec3(12., 8., 0.),
                glm::vec3(0., -PI / 2., 0.),
                glm::vec3(4., 4., 4.),
            ),
            // -Z: Drawer with stuff on top
            (
                "vintage_wooden_drawer_01",
                "vintage_wooden_drawer_01",
                glm::vec3(0., 0., -12.),
                glm::vec3(0., 0., 0.),
                glm::vec3(8., 8., 8.),
            ),
            (
                "CashRegister_01",
                "CashRegister_01",
                glm::vec3(2., 4.25, -12.2),
                glm::vec3(0., 0., 0.),
                glm::vec3(4., 4., 4.),
            ),
        ] {
            let (models, materials) = load_obj(&format!("res/models/{}_2k.obj", objname));
            let texture =
                unsafe { ImageTexture::new(gl, &format!("res/textures/{}_diff_2k.jpg", name)) };
            let normal_map =
                unsafe { ImageTexture::new(gl, &format!("res/textures/{}_nor_gl_2k.jpg", name)) };
            let roughness_map =
                unsafe { ImageTexture::new(gl, &format!("res/textures/{}_rough_2k.jpg", name)) };
            //let opacity_map =
            //unsafe { ImageTexture::new(gl, &format!("res/textures/{}_opacity_2k.jpg", name)) };

            let mut root_node = Node::new(NodeType::Root);
            root_node.position = position;
            root_node.rotation = rotation;
            root_node.scale = scale;
            let root = scene_graph.add_child(0, root_node);

            for model in models {
                let mut node = Node::new(NodeType::Geometry);
                node.vao = unsafe { Some(VAO::from_mesh(&gl, &model, &materials)) };
                node.texture = Some(texture);
                node.normal_map = Some(normal_map);
                node.roughness_map = Some(roughness_map);
                //node.opacity_map = Some(opacity_map);
                scene_graph.add_child(root, node);
            }
        }
        println!(
            "Loading models took {} seconds",
            Instant::now().duration_since(before).as_secs_f32(),
        );
    }

    for (position, color) in vec![
        //(glm::vec3(10., 3., 0.), glm::vec3(0.4, 0.4, 0.4)),
        (glm::vec3(0., 6., 6.), glm::vec3(0.4, 0.4, 0.4)),
        (glm::vec3(-10., 4., 10.), glm::vec3(0.6, 0.4, 0.4)),
        //(glm::vec3(0., 2., -4.), glm::vec3(0.6, 0.6, 0.6)),
        (glm::vec3(12., 8., 0.), glm::vec3(0.6, 0.6, 0.6)), // at lamp
    ] {
        let mut light_node = Node::new(NodeType::Light);
        light_node.position = position.clone();
        light_node.emission_color = color;
        scene_graph.add_child(0, light_node);
    }

    scene_graph
}
