use glium;
use glium::Surface;
use super::glsl_functions;

#[derive(Copy, Clone)]
pub struct RotatedRectangle {
    pub position : [f32; 2],
    pub size : [f32; 2],
    pub angle : f32,
}
implement_vertex!(RotatedRectangle, position, size, angle);

#[derive(Copy, Clone)]
struct Vertex {
    vertex_pos: [f32; 2],
}
implement_vertex!(Vertex, vertex_pos);

pub struct RectangleBuffer {
    display: glium::Display,
    vertex_buffer: glium::VertexBuffer<Vertex>,
    index_buffer: glium::index::NoIndices,
    shader_program: glium::Program,
    shader_program_instancing: glium::Program,
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

    pub fn draw_rectangles(&self, frame : &mut glium::Frame, rects : &[RotatedRectangle], colour : [f32; 4]) {
        // Uses instancing to draw many rectangles at once.

        // Create a new vertex buffer containing per-instance attributes:
        let rect_buffer = glium::VertexBuffer::new(&self.display, rects).unwrap();

        let uniforms = uniform! {
            view: self.view_matrix,
            line_col: colour,
        };

        // Draw the instanced rectangles using the instancing shader program:
        frame.draw(
            (&self.vertex_buffer, rect_buffer.per_instance().unwrap()),
            &self.index_buffer,
            &self.shader_program_instancing,
            &uniforms,
            &Default::default()
        ).unwrap();
    }

    pub fn new(display: glium::Display) -> RectangleBuffer {
        let v = 0.5;
        let vertex1 = Vertex { vertex_pos: [-v, -v] };
        let vertex2 = Vertex { vertex_pos: [ v, -v] };
        let vertex3 = Vertex { vertex_pos: [-v, v] };
        let vertex4 = Vertex { vertex_pos: [ v, v] };
        let shape = vec![
            vertex1, vertex2, vertex3,
            vertex3, vertex4, vertex2,
        ];

        let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
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
            &display,
            glsl_functions::VERTEX_SHADER_POS_MV,
            fragment_shader_src,
            None
        ).unwrap();

        let vertex_shader_src = r#"
            #version 140
            in vec2 vertex_pos;

            in vec2 position;
            in vec2 size;
            in float angle;

            uniform mat4 view;
            void main() {
                float xs = size.x;
                float ys = size.y;

                float c = cos(angle);
                float s = sin(angle);
                float r11 = c;
                float r12 = -s;
                float r21 = s;
                float r22 = c;

                mat4 model = mat4(
                    vec4(xs*r11, xs*r12, 0.0, 0.0),
                    vec4(ys*r21, ys*r22, 0.0, 0.0),
                    vec4(0.0, 0.0, 1.0, 0.0),
                    vec4(position.x, position.y, 0.0, 1.0)
                );

                gl_Position = view * model * vec4(vertex_pos, 0.0, 1.0);
            }
        "#;

        let program_instancing = glium::Program::from_source(
            &display,
            vertex_shader_src,
            fragment_shader_src,
            None
        ).unwrap();

        RectangleBuffer {
            display: display,
            vertex_buffer: vertex_buffer,
            index_buffer: indices,
            shader_program: program,
            shader_program_instancing: program_instancing,
            view_matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32],
            ],
        }
    }
}
