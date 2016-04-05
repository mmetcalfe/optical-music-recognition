use glium;
use glium_text;
use std;
use nalgebra as na;

pub struct TextHelper {
    text_system: glium_text::TextSystem,
    font_texture: glium_text::FontTexture,
}

impl TextHelper {
    pub fn new(display : &glium::Display) -> TextHelper {
        // The `TextSystem` contains the shaders and elements used for text display.
        let system = glium_text::TextSystem::new(display);

        // Creating a `FontTexture`, which is a regular `Texture` which contains the font.
        let font_name = "/Users/mitchell/Library/Fonts/FiraMono-Bold.otf";
        let font_file = std::fs::File::open(&std::path::Path::new(font_name)).unwrap();
        let font = glium_text::FontTexture::new(display, font_file, 48).unwrap();

        TextHelper {
            text_system: system,
            font_texture: font,
        }
    }

    pub fn draw_string(&self, target : &mut glium::Frame, string: &str, pos: na::Vec2<f32>, scale: f32, colour: (f32, f32, f32, f32)) {
        let text_display = glium_text::TextDisplay::new(
            &self.text_system,
            &self.font_texture,
            string
        );

        // TODO: Multiply this by the drawing context's view matrix.
        let matrix = [[scale*0.5, 0.0, 0.0, 0.0],
                      [0.0, scale, 0.0, 0.0],
                      [0.0, 0.0, 1.0, 0.0],
                      [pos[0], pos[1], 0.0, 1.0]];

        glium_text::draw(
            &text_display,
            &self.text_system,
            target,
            matrix,
            colour
        );
    }

}
