use super::Line;

#[derive(Copy, Clone)]
pub struct RotatedRectangle {
    pub position : [f32; 2],
    pub size : [f32; 2],
    pub angle : f32,
}

impl RotatedRectangle {
    pub fn from_line(line: &Line, lw: f32) -> RotatedRectangle {
        let x1 = line.a[0];
        let y1 = line.a[1];
        let x2 = line.b[0];
        let y2 = line.b[1];

        let xd = x2 - x1;
        let yd = y2 - y1;
        let len = (xd*xd + yd*yd).sqrt();

        let xa = (x1 + x2) / 2.0;
        let ya = (y1 + y2) / 2.0;

        let angle = -f32::atan2(yd, xd);

        let rect = RotatedRectangle {
            position: [xa, ya],
            size: [len, lw],
            angle: angle,
        };

        rect
    }
}
