use glium;
use glium::Surface;
use super::glsl_functions;

pub struct RotatedRectangle {
    pub position : [f32; 2],
    pub size : [f32; 2],
    pub angle : f32,
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);

pub struct RectangleBuffer {
    vertex_buffer: glium::VertexBuffer<Vertex>,
    index_buffer: glium::index::NoIndices,
    shader_program: glium::Program,
    view_matrix: [[f32; 4]; 4],
}

impl RectangleBuffer {
    pub fn set_view_matrix(&mut self, matrix: &[[f32; 4]; 4]) {
        self.view_matrix = matrix.clone()
    }

    pub fn draw_rectangle(&self, target : &mut glium::Frame, rect : &RotatedRectangle, colour : [f32; 4]) {
        let x = rect.position[0];
        let y = rect.position[1];
        let angle = rect.angle;
        let xs = rect.size[0];
        let ys = rect.size[1];

        let c = angle.cos();
        let s = angle.sin();
        let r11 = c;
        let r12 = -s;
        let r21 = s;
        let r22 = c;
        // let xr = r11*x + r12*y;
        // let yr = r21*x + r22*y;

        let uniforms = uniform! {
            model: [
                [xs*r11, xs*r12, 0.0, 0.0],
                [ys*r21, ys*r22, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [x, y, 0.0, 1.0f32],
            ],
            view: self.view_matrix,
            line_col: colour,
        };

        target.draw(
            &self.vertex_buffer,
            &self.index_buffer,
            &self.shader_program,
            &uniforms,
            &Default::default()
        ).unwrap();
    }

    pub fn new(display : &glium::Display) -> RectangleBuffer {
        let v = 0.5;
        let vertex1 = Vertex { position: [-v, -v] };
        let vertex2 = Vertex { position: [ v, -v] };
        let vertex3 = Vertex { position: [-v, v] };
        let vertex4 = Vertex { position: [ v, v] };
        let shape = vec![
            vertex1, vertex2, vertex3,
            vertex3, vertex4, vertex2,
        ];

        let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        let fragment_shader_src = r#"
            #version 140
            out vec4 color;
            uniform vec4 line_col;
            void main() {
                color = line_col;
            }
        "#;

        let program = glium::Program::from_source(
            display,
            glsl_functions::VERTEX_SHADER_POS_MV,
            fragment_shader_src,
            None
        ).unwrap();

        RectangleBuffer {
            vertex_buffer: vertex_buffer,
            index_buffer: indices,
            shader_program: program,
            view_matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32],
            ],
        }
    }
}
