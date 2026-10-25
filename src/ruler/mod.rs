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

impl RulerMode {
    /// Cycles to the next mode in order: Spacing -> Bounds -> Horizontal -> Vertical -> Spacing
    pub fn next(&self) -> Self {
        match self {
            RulerMode::Spacing => RulerMode::Bounds,
            RulerMode::Bounds => RulerMode::Horizontal,
            RulerMode::Horizontal => RulerMode::Vertical,
            RulerMode::Vertical => RulerMode::Spacing,
        }
    }

    /// Cycles to the previous mode
    pub fn prev(&self) -> Self {
        match self {
            RulerMode::Spacing => RulerMode::Vertical,
            RulerMode::Vertical => RulerMode::Horizontal,
            RulerMode::Horizontal => RulerMode::Bounds,
            RulerMode::Bounds => RulerMode::Spacing,
        }
    }

    /// User-friendly display name
    pub fn name(&self) -> &'static str {
        match self {
            RulerMode::Bounds => "Bounds (Rectangle)",
            RulerMode::Spacing => "Spacing (Cross)",
            RulerMode::Horizontal => "Horizontal",
            RulerMode::Vertical => "Vertical",
        }
    }
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

        // Clamp cursor coordinates to image boundaries to prevent out-of-bounds indexing
        let cx = cx.min(self.width - 1);
        let cy = cy.min(self.height - 1);

        let raw = self.image.as_raw();
        let stride = (self.width * 4) as usize;

        let base_idx = (cy as usize) * stride + (cx as usize) * 4;
        let base_r = raw[base_idx] as i32;
        let base_g = raw[base_idx + 1] as i32;
        let base_b = raw[base_idx + 2] as i32;

        // WHY squared distance threshold:
        // Maximum Euclidean distance in RGB color space is sqrt(255^2 * 3) ~= 441.6729559300637.
        // Squaring both sides: threshold^2 = (tolerance_pct / 100)^2 * (255^2 * 3) = (tolerance_pct / 100)^2 * 195075.
        // Operating entirely in squared Euclidean space eliminates costly `sqrt()` calls on every marched pixel.
        let tol_fraction = (tolerance_pct.clamp(1, 100) as f64) / 100.0;
        let thresh_sq = (tol_fraction * tol_fraction * 195_075.0) as i32;

        // Inlined helper to compute squared Euclidean RGB distance from base pixel
        #[inline(always)]
        fn color_diff_sq(raw: &[u8], idx: usize, base_r: i32, base_g: i32, base_b: i32) -> i32 {
            let dr = raw[idx] as i32 - base_r;
            let dg = raw[idx + 1] as i32 - base_g;
            let db = raw[idx + 2] as i32 - base_b;
            dr * dr + dg * dg + db * db
        }

        // WHY row_offset / col_offset hoisting:
        // Calculating `(cy * stride)` or `(cx * 4)` once outside each loop saves millions of redundant
        // integer multiplications per second across rapid cursor movements.
        let row_offset = (cy as usize) * stride;
        let col_offset = (cx as usize) * 4;

        // March LEFT along row `cy`
        let mut left_x = cx;
        while left_x > 0 {
            let next_x = left_x - 1;
            let idx = row_offset + (next_x as usize) * 4;
            if color_diff_sq(raw, idx, base_r, base_g, base_b) > thresh_sq {
                break;
            }
            left_x = next_x;
        }

        // March RIGHT along row `cy`
        let mut right_x = cx;
        while right_x + 1 < self.width {
            let next_x = right_x + 1;
            let idx = row_offset + (next_x as usize) * 4;
            if color_diff_sq(raw, idx, base_r, base_g, base_b) > thresh_sq {
                break;
            }
            right_x = next_x;
        }

        // March UP along column `cx`
        let mut top_y = cy;
        while top_y > 0 {
            let next_y = top_y - 1;
            let idx = (next_y as usize) * stride + col_offset;
            if color_diff_sq(raw, idx, base_r, base_g, base_b) > thresh_sq {
                break;
            }
            top_y = next_y;
        }

        // March DOWN along column `cx`
        let mut bottom_y = cy;
        while bottom_y + 1 < self.height {
            let next_y = bottom_y + 1;
            let idx = (next_y as usize) * stride + col_offset;
            if color_diff_sq(raw, idx, base_r, base_g, base_b) > thresh_sq {
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
