use ffmpeg_camera::image_uyvy;
use ffmpeg_camera::image_ycbcr;
use ffmpeg_camera::image::Image;
use glium;
use glium::Surface;
use std::borrow::Cow;
use ffmpeg_camera::ffmpeg_utils;
use super::glsl_functions;

extern crate core;
use self::core::ops::Deref;

pub fn texture_to_image(texture : &glium::texture::Texture2d) -> image_ycbcr::Image {
    // // Slow options (i.e. ~500ms):
    // // Somewhat informative: http://gamedev.stackexchange.com/a/44798
    // let raw_image : glium::texture::RawImage2d<u8> = dst_texture.read();
    // let raw_image : glium::texture::RawImage2d<u8> = buffer.read_as_texture_2d().unwrap(); // Slow
    // let data = raw_image.data.deref().to_vec();

    // So much faster (i.e. many frames per second):
    let pixel_buffer = texture.read_to_pixel_buffer();
    let local_buffer : Vec<(u8, u8, u8, u8)> = pixel_buffer.read().unwrap();

    // Note: This step is unstable, as it relies on undefined behaviour. Rust does not define
    // the memory layout of tuples, but we assume here that the obvious layout is used (i.e.
    // the fields are packed together and are layed out in declaration order).
    let data = ffmpeg_utils::vec_to_bytes(local_buffer);

    image_ycbcr::Image::from_raw_parts(
        texture.get_width() as usize,
        texture.get_height().unwrap() as usize,
        data
    )
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

pub struct ImagePane<'a> {
    display : &'a glium::Display,
    vertex_buffer : glium::VertexBuffer<Vertex>,
    index_buffer : glium::index::NoIndices,
    uyuv_ycbcr_conversion_program : glium::Program,
    ycbcr_drawing_program : glium::Program,
}

impl<'a> ImagePane<'a> {
    pub fn draw_texture(&self, target: &mut glium::Frame, program: &glium::Program, texture: glium::texture::Texture2d) {
        let uniforms = uniform! {
            tex: &texture,
        };
        target.draw(
            &self.vertex_buffer,
            &self.index_buffer,
            program,
            &uniforms,
            &Default::default()
        ).unwrap();
    }

    pub fn uyvy_image_to_texture(&self, image : &image_uyvy::Image) -> glium::texture::Texture2d {
        let cow: Cow<[_]> = Cow::Borrowed(&image.data);
        // let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);
        let img_w = image.width as u32;
        let img_h = image.height as u32;
        let raw_image = glium::texture::RawImage2d {
            data: cow,
            width: img_w,
            height: img_h,
            format: glium::texture::ClientFormat::U8U8
        };
        let texture = glium::texture::Texture2d::new(self.display, raw_image).unwrap();

        texture
    }

    pub fn ycbcr_image_to_texture(&self, image : &image_ycbcr::Image) -> glium::texture::Texture2d {
        let cow: Cow<[_]> = Cow::Borrowed(&image.data);
        // let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);
        let img_w = image.width as u32;
        let img_h = image.height as u32;
        let raw_image = glium::texture::RawImage2d {
            data: cow,
            width: img_w,
            height: img_h,
            format: glium::texture::ClientFormat::U8U8U8U8
        };
        let texture = glium::texture::Texture2d::new(self.display, raw_image).unwrap();

        texture
    }

    pub fn convert_uyvy_ycbcr(&self, uyvy_image : &image_uyvy::Image)
        -> Result<image_ycbcr::Image, glium::framebuffer::ValidationError> {
        let src_texture = self.uyvy_image_to_texture(uyvy_image);

        let dst_texture = glium::texture::Texture2d::empty_with_format(
            self.display,
            glium::texture::UncompressedFloatFormat::U8U8U8U8,
            glium::texture::MipmapsOption::NoMipmap,
            uyvy_image.width as u32,
            uyvy_image.height as u32
        ).unwrap();
        let mut framebuffer = try!(glium::framebuffer::SimpleFrameBuffer::new(self.display, &dst_texture));

        let uniforms = uniform! {
            tex: &src_texture,
        };
        framebuffer.draw(
            &self.vertex_buffer,
            &self.index_buffer,
            &self.uyuv_ycbcr_conversion_program,
            &uniforms,
            &Default::default()
        ).unwrap();

        Ok(texture_to_image(&dst_texture))
    }

    pub fn draw_image_uyvy(&self, target : &mut glium::Frame, image : &image_uyvy::Image) {
        let texture = self.uyvy_image_to_texture(image);

        self.draw_texture(target, &self.uyuv_ycbcr_conversion_program, texture)
    }

    pub fn draw_image_ycbcr(&self, target : &mut glium::Frame, image : &image_ycbcr::Image) {
        let texture = self.ycbcr_image_to_texture(image);

        self.draw_texture(target, &self.ycbcr_drawing_program, texture)
    }

    pub fn make_uyuv_ycbcr_conversion_program(display : &glium::Display) -> glium::Program {
        let fragment_shader_src = String::new()
            + r#"
            #version 140
            in vec2 v_tex_coords;
            out vec4 color;
            uniform sampler2D tex;
            "#
            + glsl_functions::CONVERT_UYVY422_YUV24
            + glsl_functions::CONVERT_YCBCRA_RGBA
            + r#"
            void main() {
                // ivec2 pix_1 = ivec2(v_tex_coords.x*1280.0, v_tex_coords.y*720.0);
                ivec2 pix_1 = ivec2(gl_FragCoord.xy);
                vec4 ycbcra = convert_uyvy422_yuv24(tex, pix_1);
                color = ycbcra;
            }
        "#;

        glium::Program::from_source(
            display,
            glsl_functions::VERTEX_SHADER_POS_TEX,
            &fragment_shader_src,
            None
        ).unwrap()
    }

    pub fn make_ycbcr_drawing_program(display : &glium::Display) -> glium::Program {
        let fragment_shader_src = String::new()
            + r#"
            #version 140
            in vec2 v_tex_coords;
            out vec4 color;
            uniform sampler2D tex;
            "#
            + glsl_functions::CONVERT_YCBCRA_RGBA
            + r#"
            void main() {
                vec4 ycbcra = texture(tex, v_tex_coords);
                vec4 rgba = convert_ycbcra_rgba(ycbcra);
                color = rgba;
            }
        "#;

        glium::Program::from_source(
            display,
            glsl_functions::VERTEX_SHADER_POS_TEX,
            &fragment_shader_src,
            None
        ).unwrap()
    }

    pub fn new(display : &glium::Display) -> ImagePane {
        // let v = 0.95;
        let v = 1.0;
        let vx = -v;
        let vy = v;
        let vertex1 = Vertex { position: [-vx, -vy], tex_coords: [1.0, 1.0] };
        let vertex2 = Vertex { position: [ vx, -vy], tex_coords: [0.0, 1.0] };
        let vertex3 = Vertex { position: [-vx, vy], tex_coords: [1.0, 0.0] };
        let vertex4 = Vertex { position: [ vx, vy], tex_coords: [0.0, 0.0] };
        let shape = vec![
            vertex1, vertex2, vertex3,
            vertex3, vertex4, vertex2,
        ];

        let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        ImagePane {
            display : display,
            vertex_buffer: vertex_buffer,
            index_buffer: indices,
            uyuv_ycbcr_conversion_program: Self::make_uyuv_ycbcr_conversion_program(display),
            ycbcr_drawing_program: Self::make_ycbcr_drawing_program(display),
        }
    }
}
