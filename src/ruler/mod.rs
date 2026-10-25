pub mod bounds;

use image::RgbaImage;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulerMode {
    Bounds,
    Spacing,
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeMeasurement {
    pub cursor_x: u32,
    pub cursor_y: u32,
    pub left_x: u32,
    pub right_x: u32,
    pub top_y: u32,
    pub bottom_y: u32,
    pub width: u32,
    pub height: u32,
}

impl EdgeMeasurement {
    pub fn empty(cx: u32, cy: u32) -> Self {
        Self {
            cursor_x: cx,
            cursor_y: cy,
            left_x: cx,
            right_x: cx,
            top_y: cy,
            bottom_y: cy,
            width: 1,
            height: 1,
        }
    }
}

pub struct EdgeDetector {
    image: Arc<RgbaImage>,
    width: u32,
    height: u32,
}

impl EdgeDetector {
    pub fn new(image: Arc<RgbaImage>) -> Self {
        let (width, height) = image.dimensions();
        Self {
            image,
            width,
            height,
        }
    }

    #[allow(dead_code)]
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Detects edges in 4 cardinal directions starting from (cx, cy) with the specified tolerance (1..=100)
    pub fn detect_edges(&self, cx: u32, cy: u32, tolerance_pct: u8) -> EdgeMeasurement {
        if self.width == 0 || self.height == 0 {
            return EdgeMeasurement::empty(cx, cy);
        }

        let cx = cx.min(self.width - 1);
        let cy = cy.min(self.height - 1);

        let raw = self.image.as_raw();
        let stride = (self.width * 4) as usize;

        let base_idx = (cy as usize) * stride + (cx as usize) * 4;
        let base_r = raw[base_idx] as i32;
        let base_g = raw[base_idx + 1] as i32;
        let base_b = raw[base_idx + 2] as i32;

        // Tolerance percentage converted to squared Euclidean distance threshold
        // Maximum Euclidean distance in RGB space = sqrt(255^2 * 3) ~= 441.67
        let tol_factor = (tolerance_pct.clamp(1, 100) as f64) / 100.0;
        let max_dist = 441.6729559300637;
        let thresh_dist = tol_factor * max_dist;
        let thresh_sq = (thresh_dist * thresh_dist) as i32;

        // March LEFT
        let mut left_x = cx;
        while left_x > 0 {
            let next_x = left_x - 1;
            let idx = (cy as usize) * stride + (next_x as usize) * 4;
            let r = raw[idx] as i32;
            let g = raw[idx + 1] as i32;
            let b = raw[idx + 2] as i32;

            let dr = r - base_r;
            let dg = g - base_g;
            let db = b - base_b;
            let dist_sq = dr * dr + dg * dg + db * db;

            if dist_sq > thresh_sq {
                break;
            }
            left_x = next_x;
        }

        // March RIGHT
        let mut right_x = cx;
        while right_x + 1 < self.width {
            let next_x = right_x + 1;
            let idx = (cy as usize) * stride + (next_x as usize) * 4;
            let r = raw[idx] as i32;
            let g = raw[idx + 1] as i32;
            let b = raw[idx + 2] as i32;

            let dr = r - base_r;
            let dg = g - base_g;
            let db = b - base_b;
            let dist_sq = dr * dr + dg * dg + db * db;

            if dist_sq > thresh_sq {
                break;
            }
            right_x = next_x;
        }

        // March UP
        let mut top_y = cy;
        while top_y > 0 {
            let next_y = top_y - 1;
            let idx = (next_y as usize) * stride + (cx as usize) * 4;
            let r = raw[idx] as i32;
            let g = raw[idx + 1] as i32;
            let b = raw[idx + 2] as i32;

            let dr = r - base_r;
            let dg = g - base_g;
            let db = b - base_b;
            let dist_sq = dr * dr + dg * dg + db * db;

            if dist_sq > thresh_sq {
                break;
            }
            top_y = next_y;
        }

        // March DOWN
        let mut bottom_y = cy;
        while bottom_y + 1 < self.height {
            let next_y = bottom_y + 1;
            let idx = (next_y as usize) * stride + (cx as usize) * 4;
            let r = raw[idx] as i32;
            let g = raw[idx + 1] as i32;
            let b = raw[idx + 2] as i32;

            let dr = r - base_r;
            let dg = g - base_g;
            let db = b - base_b;
            let dist_sq = dr * dr + dg * dg + db * db;

            if dist_sq > thresh_sq {
                break;
            }
            bottom_y = next_y;
        }

        let width = (right_x - left_x) + 1;
        let height = (bottom_y - top_y) + 1;

        EdgeMeasurement {
            cursor_x: cx,
            cursor_y: cy,
            left_x,
            right_x,
            top_y,
            bottom_y,
            width,
            height,
        }
    }
}
