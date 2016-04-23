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

    pub fn get_transform(&self) -> [[f32; 4]; 4] {
        let x = self.position[0];
        let y = self.position[1];
        let angle = self.angle;
        let xs = self.size[0] / 2.0;
        let ys = self.size[1] / 2.0;

        let c = angle.cos();
        let s = angle.sin();
        let r11 = c;
        let r12 = -s;
        let r21 = s;
        let r22 = c;
        // let xr = r11*x + r12*y;
        // let yr = r21*x + r22*y;

        [
            [xs*r11, xs*r12, 0.0, 0.0],
            [ys*r21, ys*r22, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [x, y, 0.0, 1.0f32],
        ]
    }
}
