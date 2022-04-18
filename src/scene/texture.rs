use glow::*;
use image::io::Reader as ImageReader;

#[derive(Clone, Copy)]
pub struct FrameBufferTexture {
    pub framebuffer: Option<NativeFramebuffer>,
    pub texture: NativeTexture,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Copy)]
pub struct ImageTexture {
    pub texture: NativeTexture,
}

#[derive(Clone, Copy)]
pub struct PBRTextures {
    pub texture: NativeTexture,
    pub normals: NativeTexture,
    pub roughness: NativeTexture,
}

#[derive(Clone, Copy)]
pub struct CubemapTexture {
    pub framebuffers: [NativeFramebuffer; 6],
    pub texture: NativeTexture,
    pub size: i32,
}

#[derive(Clone, Copy)]
pub struct PostProcessingTexture {
    pub framebuffer: NativeFramebuffer,
    pub color_buffer_texture: NativeTexture,
    pub depth_buffer_texture: NativeTexture,
    pub width: i32,
    pub height: i32,
}

impl FrameBufferTexture {
    pub unsafe fn new(gl: &glow::Context, width: i32, height: i32) -> FrameBufferTexture {
        let framebuffer = gl
            .create_framebuffer()
            .expect("Could not create framebuffer");
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));

        // Create texture
        let texture = gl.create_texture().expect("Could not create texture");
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as i32,
            width,
            height,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            None,
        );
        // Specify mipmap interpolation
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::LINEAR as i32,
        );

        // Attach texture to framebuffer
        gl.framebuffer_texture(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, Some(texture), 0);
        gl.draw_buffer(glow::COLOR_ATTACHMENT0);

        // Depth texture
        // (no need to do it as a texture here, but this does *work* at least)
        let depth_buffer_texture = gl.create_texture().expect("Could not create texture");
        gl.bind_texture(glow::TEXTURE_2D, Some(depth_buffer_texture));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::DEPTH_COMPONENT24 as i32,
            width,
            height,
            0,
            glow::DEPTH_COMPONENT,
            glow::FLOAT,
            None,
        );
        //// Specify mipmap interpolation
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::LINEAR as i32,
        );
        // Specify depth-specific options
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_COMPARE_FUNC,
            glow::LEQUAL as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_COMPARE_MODE,
            glow::NONE as i32,
        );

        // Attach texture to framebuffer
        gl.framebuffer_texture(
            glow::FRAMEBUFFER,
            glow::DEPTH_ATTACHMENT,
            Some(depth_buffer_texture),
            0,
        );

        if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
            panic!("Framebuffer creation failed!");
        }

        gl.bind_framebuffer(glow::FRAMEBUFFER, None);

        FrameBufferTexture {
            framebuffer: Some(framebuffer),
            texture,
            width,
            height,
        }
    }
}

impl ImageTexture {
    pub unsafe fn new(gl: &glow::Context, filepath: &str) -> FrameBufferTexture {
        // Load image
        let image = ImageReader::open(filepath)
            .expect(&format!("Could not open {}", filepath))
            .decode()
            .expect(&format!("Error processing image at {}", filepath));

        // Create texture
        let texture = gl.create_texture().expect("Could not create texture");
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as i32,
            image.width() as i32,
            image.height() as i32,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            Some(image.to_rgba8().to_vec().as_slice()),
        );
        // Specify mipmap interpolation
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::LINEAR as i32,
        );

        FrameBufferTexture {
            framebuffer: None,
            texture,
            width: 0, // irrelevant
            height: 0,
        }
    }
}

impl CubemapTexture {
    pub unsafe fn new(gl: &glow::Context, size: i32) -> CubemapTexture {
        let mut framebuffers = [None, None, None, None, None, None];
        // Create texture
        let texture = gl.create_texture().expect("Could not create texture");
        gl.bind_texture(glow::TEXTURE_CUBE_MAP, Some(texture));

        // Specify mipmap interpolation
        gl.tex_parameter_i32(
            glow::TEXTURE_CUBE_MAP,
            glow::TEXTURE_MAG_FILTER,
            glow::LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_CUBE_MAP,
            glow::TEXTURE_MIN_FILTER,
            glow::LINEAR as i32,
        );
        // Clamp to edges between cubemap sides to avoid ugly borders
        gl.tex_parameter_i32(
            glow::TEXTURE_CUBE_MAP,
            glow::TEXTURE_WRAP_R,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_CUBE_MAP,
            glow::TEXTURE_WRAP_S,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_CUBE_MAP,
            glow::TEXTURE_WRAP_T,
            glow::CLAMP_TO_EDGE as i32,
        );

        for i in 0..6 {
            framebuffers[i] = Some(
                gl.create_framebuffer()
                    .expect("Could not create framebuffer"),
            );
            gl.bind_framebuffer(glow::FRAMEBUFFER, framebuffers[i]);

            gl.tex_image_2d(
                glow::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                0,
                glow::RGBA as i32,
                size,
                size,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                None,
            );

            // Attach texture to framebuffer
            //gl.framebuffer_texture(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, Some(texture), 0);
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                Some(texture),
                0,
            );
            gl.draw_buffer(glow::COLOR_ATTACHMENT0);

            // Depth texture
            // (no need to do it as a texture here, but this does *work* at least)
            let depth_buffer_texture = gl.create_texture().expect("Could not create texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(depth_buffer_texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::DEPTH_COMPONENT24 as i32,
                size,
                size,
                0,
                glow::DEPTH_COMPONENT,
                glow::FLOAT,
                None,
            );
            //// Specify mipmap interpolation
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            // Specify depth-specific options
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_COMPARE_FUNC,
                glow::LEQUAL as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_COMPARE_MODE,
                glow::NONE as i32,
            );

            // Attach texture to framebuffer
            gl.framebuffer_texture(
                glow::FRAMEBUFFER,
                glow::DEPTH_ATTACHMENT,
                Some(depth_buffer_texture),
                0,
            );

            //if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
            //    panic!(
            //        "Framebuffer creation failed! {}",
            //        gl.check_framebuffer_status(glow::FRAMEBUFFER)
            //    );
            //}
        }

        gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        gl.bind_texture(glow::TEXTURE_CUBE_MAP, None);

        CubemapTexture {
            // I don't rust :))))
            framebuffers: [
                framebuffers[0].unwrap(),
                framebuffers[1].unwrap(),
                framebuffers[2].unwrap(),
                framebuffers[3].unwrap(),
                framebuffers[4].unwrap(),
                framebuffers[5].unwrap(),
            ],
            texture,
            size,
        }
    }
}

impl PostProcessingTexture {
    pub unsafe fn new(gl: &glow::Context, width: i32, height: i32) -> PostProcessingTexture {
        let framebuffer = gl
            .create_framebuffer()
            .expect("Could not create framebuffer");
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));

        // Color texture
        let color_buffer_texture = gl.create_texture().expect("Could not create texture");
        gl.bind_texture(glow::TEXTURE_2D, Some(color_buffer_texture));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as i32,
            width,
            height,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            None,
        );
        // Specify mipmap interpolation
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::LINEAR as i32,
        );

        // Attach texture to framebuffer
        gl.framebuffer_texture(
            glow::FRAMEBUFFER,
            glow::COLOR_ATTACHMENT0,
            Some(color_buffer_texture),
            0,
        );
        // Set draw buffer to the attachment with the color texture
        gl.draw_buffer(glow::COLOR_ATTACHMENT0);

        // Depth texture
        let depth_buffer_texture = gl.create_texture().expect("Could not create texture");
        gl.bind_texture(glow::TEXTURE_2D, Some(depth_buffer_texture));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::DEPTH_COMPONENT24 as i32,
            width,
            height,
            0,
            glow::DEPTH_COMPONENT,
            glow::FLOAT,
            None,
        );
        //// Specify mipmap interpolation
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::LINEAR as i32,
        );
        // Specify depth-specific options
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_COMPARE_FUNC,
            glow::LEQUAL as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_COMPARE_MODE,
            glow::NONE as i32,
        );

        // Attach texture to framebuffer
        gl.framebuffer_texture(
            glow::FRAMEBUFFER,
            glow::DEPTH_ATTACHMENT,
            Some(depth_buffer_texture),
            0,
        );

        if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
            panic!("Framebuffer creation failed!");
        }

        gl.bind_framebuffer(glow::FRAMEBUFFER, None);

        PostProcessingTexture {
            framebuffer,
            color_buffer_texture,
            depth_buffer_texture,
            width,
            height,
        }
    }
}
