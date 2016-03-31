use ffmpeg_camera::image_uyvy;
use ffmpeg_camera::image_ycbcr;
use ffmpeg_camera::image::Image;
use glium;
use glium::Surface;
use std::borrow::Cow;
use ffmpeg_camera::ffmpeg_utils;

extern crate core;
use self::core::ops::Deref;

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
    shader_program : glium::Program,
}

impl<'a> ImagePane<'a> {
    pub fn draw_texture(&self, target : &mut glium::Frame, texture : glium::texture::Texture2d) {
        let uniforms = uniform! {
            tex: &texture,
        };

        target.draw(
            &self.vertex_buffer,
            &self.index_buffer,
            &self.shader_program,
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
            &self.shader_program,
            &uniforms,
            &Default::default()
        ).unwrap();

        // // Slow options (i.e. ~500ms):
        // // Somewhat informative: http://gamedev.stackexchange.com/a/44798
        // let raw_image : glium::texture::RawImage2d<u8> = dst_texture.read();
        // let raw_image : glium::texture::RawImage2d<u8> = buffer.read_as_texture_2d().unwrap(); // Slow
        // let data = raw_image.data.deref().to_vec();

        // So much faster (i.e. many frames per second):
        let pixel_buffer = dst_texture.read_to_pixel_buffer();
        let local_buffer : Vec<(u8, u8, u8, u8)> = pixel_buffer.read().unwrap();

        // Note: This step is unstable, as it relies on undefined behaviour. Rust does not define
        // the memory layout of tuples, but we assume here that the obvious layout is used (i.e.
        // the fields are packed together and are layed out in declaration order).
        let data = ffmpeg_utils::vec_to_bytes(local_buffer);

        Ok(image_ycbcr::Image::from_raw_parts(
            uyvy_image.width as usize,
            uyvy_image.height as usize,
            data
        ))
    }

    pub fn draw_image_uyvy(&self, target : &mut glium::Frame, image : &image_uyvy::Image) {
        let texture = self.uyvy_image_to_texture(image);

        self.draw_texture(target, texture)
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

        let vertex_shader_src = r#"
            #version 140
            in vec2 position;
            in vec2 tex_coords;
            out vec2 v_tex_coords;
            // uniform mat4 matrix;
            void main() {
                v_tex_coords = tex_coords;
                // gl_Position = matrix * vec4(position, 0.0, 1.0);
                gl_Position = vec4(position, 0.0, 1.0);
            }
        "#;

        // Based on: http://stackoverflow.com/q/25440114/3622526
        let fragment_shader_src = r#"
            #version 140
            in vec2 v_tex_coords;
            out vec4 color;
            uniform sampler2D tex;

            vec4 convert_uyvy422_yuv24(sampler2D uyvy422_tex, ivec2 pix_1) {
                bool is_odd = mod(pix_1.x, 2) != 0;
                int offset = is_odd ? -1 : 1;
                ivec2 pix_2 = ivec2(pix_1.x + offset, pix_1.y);

                vec4 col_1 = texelFetch(uyvy422_tex, pix_1, 0);
                vec4 col_2 = texelFetch(uyvy422_tex, pix_2, 0);

                float y, cb, cr;
                if (is_odd) {
                    vec2 uy = col_1.xy;
                    vec2 vy = col_2.xy;
                    y = uy.y;
                    cb = vy.x;
                    cr = uy.x;
                } else {
                    vec2 uy = col_2.xy;
                    vec2 vy = col_1.xy;
                    y = vy.y;
                    cb = vy.x;
                    cr = uy.x;
                }

                return vec4(y, cb, cr, 1.0);
            }

            vec4 convert_ycbcra_rgba(vec4 ycbcra) {
                float y = ycbcra.x;
                float cb = ycbcra.y;
                float cr = ycbcra.z;
                float a = ycbcra.w;

                // From https://en.wikipedia.org/wiki/YCbCr#JPEG_conversion
                float r = y + 1.402*(cr-0.5);
                float g = y - 0.34414*(cb-0.5) - 0.71414*(cr-0.5);
                float b = y + 1.772*(cb-0.5);

                return vec4(r, g, b, a);
            }

            void main() {
                // ivec2 pix_1 = ivec2(v_tex_coords.x*1280.0, v_tex_coords.y*720.0);
                ivec2 pix_1 = ivec2(gl_FragCoord.xy);

                vec4 ycbcra = convert_uyvy422_yuv24(tex, pix_1);
                vec4 rgba = convert_ycbcra_rgba(ycbcra);

                color = rgba;
                // color = texture(tex, v_tex_coords);
            }
        "#;

        let program = glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap();

        ImagePane {
            display : display,
            vertex_buffer: vertex_buffer,
            index_buffer: indices,
            shader_program: program
        }
    }
}
