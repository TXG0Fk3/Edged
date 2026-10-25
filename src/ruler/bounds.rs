#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    #[allow(dead_code)]
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Constructs a normalized Rect from any two opposite corner points
    pub fn from_points(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        let x = x1.min(x2);
        let y = y1.min(y2);
        let width = (x1 - x2).abs() + 1.0;
        let height = (y1 - y2).abs() + 1.0;
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the center point (x, y) of the rectangle
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Returns true if the given point (px, py) lies within this rectangle
    #[allow(dead_code)]
    pub fn contains(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }
}
