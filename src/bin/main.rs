#[macro_use]
extern crate glium;
// extern crate image;

extern crate optical_music_recognition;
use optical_music_recognition::ffmpeg_camera::ffmpeg_camera;

// use std::io::Cursor;
use glium::DisplayBuild;
use glium::Surface;

use std::borrow::Cow;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

fn main() {
    // let mut camera =
    //     ffmpeg_camera::FfmpegCamera::get_default()
    //         .expect("Failed to open camera.");
    let mut camera =
        ffmpeg_camera::FfmpegCamera::get_camera("default", "29.970000", (1280, 720))
            .expect("Failed to open camera.");

    let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();


    // let image = image::load(Cursor::new(&include_bytes!("../../curved-3.jpg")[..]),
    //                         image::JPEG).unwrap().to_rgba();
    // let image_dimensions = image.dimensions();
    // let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);
    // let texture = glium::texture::Texture2d::new(&display, image).unwrap();


    let v = 0.95;
    let vertex1 = Vertex { position: [-v, -v], tex_coords: [1.0, 1.0] };
    let vertex2 = Vertex { position: [ v, -v], tex_coords: [0.0, 1.0] };
    let vertex3 = Vertex { position: [ -v,  v], tex_coords: [1.0, 0.0] };
    let vertex4 = Vertex { position: [ v, v], tex_coords: [0.0, 0.0] };
    let shape = vec![
        vertex1, vertex2, vertex3,
        vertex3, vertex4, vertex2,
    ];

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let vertex_shader_src = r#"
        #version 140
        in vec2 position;
        in vec2 tex_coords;
        out vec2 v_tex_coords;
        uniform mat4 matrix;
        void main() {
            v_tex_coords = tex_coords;
            gl_Position = matrix * vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140
        in vec2 v_tex_coords;
        out vec4 color;
        uniform sampler2D tex;
        void main() {
            vec4 uyvy = texture(tex, v_tex_coords);

            // From https://en.wikipedia.org/wiki/YCbCr#JPEG_conversion
            // (it's close enough)
            float y = uyvy.y;
            float cb = uyvy.x;
            float cr = uyvy.z;

            float r = y + 1.402*(cr - 0.5);
            float g = y - 0.34414*(cb-0.5) - 0.71414*(cr-0.5);
            float b = y + 1.772*(cb-0.5);

            color = vec4(r, g, b, 1);

            // color = texture(tex, v_tex_coords);
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    //  let mut t = -0.5;
     let t = 0.0;

    loop {
        let webcam_frame = camera.get_image().unwrap();
        let cow: Cow<[_]> = Cow::Owned(webcam_frame.data);
        // let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);
        let img_w = webcam_frame.width as u32 / 2;
        let img_h = webcam_frame.height as u32;
        let raw_image = glium::texture::RawImage2d {
            data: cow,
            width: img_w,
            height: img_h,
            format: glium::texture::ClientFormat::U8U8U8U8
        };
        let texture = glium::texture::Texture2d::new(&display, raw_image).unwrap();

        // image.save_pgm("image.pgm").unwrap();
        // image.save_jpeg("image.jpg").unwrap();

        // // we update `t`
        // t += 0.0002;
        // if t > 0.5 {
        //     t = -0.5;
        // }

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [ t , 0.0, 0.0, 1.0f32],
            ],
            tex: &texture,
        };

        target.draw(&vertex_buffer, &indices, &program, &uniforms,
                    &Default::default()).unwrap();
        target.finish().unwrap();

        // listing the events produced by the window and waiting to be received
        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => return,   // the window has been closed by the user
                _ => ()
            }
        }
    }
}
