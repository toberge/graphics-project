use glow::*;

#[derive(Clone, Copy)]
pub struct Texture {
    pub framebuffer: Option<NativeFramebuffer>,
    pub texture: NativeTexture,
}

impl Texture {
    pub unsafe fn framebuffer_texture(
        gl: &glow::Context,
        width: i32,
        height: i32,
        attachment: u32,
    ) -> Texture {
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
        gl.framebuffer_texture(glow::FRAMEBUFFER, attachment, Some(texture), 0);
        gl.draw_buffer(attachment);

        if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
            panic!("Framebuffer creation failed!");
        }

        Texture {
            framebuffer: Some(framebuffer),
            texture,
        }
    }
}
