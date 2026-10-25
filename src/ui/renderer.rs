use crate::config::{parse_color, Config, SecondaryUnit};
use crate::ruler::bounds::Rect;
use crate::ruler::{EdgeMeasurement, RulerMode};
use egui::{Color32, FontId, Pos2, Rect as EguiRect, Stroke, StrokeKind, Vec2};
use std::sync::Arc;

pub fn to_egui_color(color_str: &str) -> Color32 {
    let (r, g, b, a) = parse_color(color_str);
    Color32::from_rgba_unmultiplied(
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
        (a * 255.0).round() as u8,
    )
}

pub struct EguiRenderer<'a> {
    pub painter: &'a egui::Painter,
    pub config: Arc<Config>,
    pub scale_x: f32,
    pub scale_y: f32,
    pub viewport_origin: Pos2,
}

impl<'a> EguiRenderer<'a> {
    pub fn new(
        painter: &'a egui::Painter,
        config: Arc<Config>,
        scale_x: f32,
        scale_y: f32,
        viewport_origin: Pos2,
    ) -> Self {
        Self {
            painter,
            config,
            scale_x,
            scale_y,
            viewport_origin,
        }
    }

    /// Renders measurement lines, feet, and dimension badges for Spacing, Horizontal, and Vertical modes
    pub fn draw_measurement(
        &self,
        mode: RulerMode,
        cursor: Pos2,
        measurement: &EdgeMeasurement,
        tolerance_hud: Option<u8>,
        mode_hud: Option<RulerMode>,
    ) {
        let cx = cursor.x;
        let cy = cursor.y;
        let left_x = self.viewport_origin.x + measurement.left_x as f32 / self.scale_x;
        let right_x = self.viewport_origin.x + measurement.right_x as f32 / self.scale_x;
        let top_y = self.viewport_origin.y + measurement.top_y as f32 / self.scale_y;
        let bottom_y = self.viewport_origin.y + measurement.bottom_y as f32 / self.scale_y;

        let feet_size = self.config.ruler.feet_size as f32;
        let draw_feet = self.config.ruler.draw_feet;
        let line_thickness = self.config.ruler.line_thickness as f32;

        let line_color = to_egui_color(&self.config.appearance.line_color);
        let shadow_color = to_egui_color(&self.config.appearance.line_shadow_color);

        let stroke_shadow = Stroke::new(line_thickness + 2.0, shadow_color);
        let stroke_line = Stroke::new(line_thickness, line_color);

        // 1. Horizontal line
        if mode == RulerMode::Spacing || mode == RulerMode::Horizontal {
            // Shadow
            self.painter.line_segment([Pos2::new(left_x, cy), Pos2::new(right_x, cy)], stroke_shadow);
            if draw_feet {
                self.painter.line_segment(
                    [Pos2::new(left_x, cy - feet_size / 2.0), Pos2::new(left_x, cy + feet_size / 2.0)],
                    stroke_shadow,
                );
                self.painter.line_segment(
                    [Pos2::new(right_x, cy - feet_size / 2.0), Pos2::new(right_x, cy + feet_size / 2.0)],
                    stroke_shadow,
                );
            }

            // Primary line
            self.painter.line_segment([Pos2::new(left_x, cy), Pos2::new(right_x, cy)], stroke_line);
            if draw_feet {
                self.painter.line_segment(
                    [Pos2::new(left_x, cy - feet_size / 2.0), Pos2::new(left_x, cy + feet_size / 2.0)],
                    stroke_line,
                );
                self.painter.line_segment(
                    [Pos2::new(right_x, cy - feet_size / 2.0), Pos2::new(right_x, cy + feet_size / 2.0)],
                    stroke_line,
                );
            }

            // Badge
            let badge_text = self.format_length(measurement.width);
            let badge_x = (left_x + right_x) / 2.0;
            let badge_y = if mode == RulerMode::Spacing {
                cy - 20.0
            } else {
                cy - 16.0
            };
            self.draw_badge(Pos2::new(badge_x, badge_y), &badge_text);
        }

        // 2. Vertical line
        if mode == RulerMode::Spacing || mode == RulerMode::Vertical {
            // Shadow
            self.painter.line_segment([Pos2::new(cx, top_y), Pos2::new(cx, bottom_y)], stroke_shadow);
            if draw_feet {
                self.painter.line_segment(
                    [Pos2::new(cx - feet_size / 2.0, top_y), Pos2::new(cx + feet_size / 2.0, top_y)],
                    stroke_shadow,
                );
                self.painter.line_segment(
                    [Pos2::new(cx - feet_size / 2.0, bottom_y), Pos2::new(cx + feet_size / 2.0, bottom_y)],
                    stroke_shadow,
                );
            }

            // Primary line
            self.painter.line_segment([Pos2::new(cx, top_y), Pos2::new(cx, bottom_y)], stroke_line);
            if draw_feet {
                self.painter.line_segment(
                    [Pos2::new(cx - feet_size / 2.0, top_y), Pos2::new(cx + feet_size / 2.0, top_y)],
                    stroke_line,
                );
                self.painter.line_segment(
                    [Pos2::new(cx - feet_size / 2.0, bottom_y), Pos2::new(cx + feet_size / 2.0, bottom_y)],
                    stroke_line,
                );
            }

            // Badge
            let badge_text = self.format_length(measurement.height);
            let badge_x = if mode == RulerMode::Spacing {
                cx + 28.0
            } else {
                cx + 24.0
            };
            let badge_y = (top_y + bottom_y) / 2.0;
            self.draw_badge(Pos2::new(badge_x, badge_y), &badge_text);
        }

        // 3. Subtle cursor crosshair
        if self.config.ruler.draw_crosshair {
            let arm = 7.0;
            let ch_shadow = Stroke::new(3.0, shadow_color);
            let ch_primary = Stroke::new(1.5, Color32::from_rgb(240, 240, 240));

            self.painter.line_segment([Pos2::new(cx - arm, cy), Pos2::new(cx + arm, cy)], ch_shadow);
            self.painter.line_segment([Pos2::new(cx, cy - arm), Pos2::new(cx, cy + arm)], ch_shadow);
            self.painter.line_segment([Pos2::new(cx - arm, cy), Pos2::new(cx + arm, cy)], ch_primary);
            self.painter.line_segment([Pos2::new(cx, cy - arm), Pos2::new(cx, cy + arm)], ch_primary);
        }

        // 4. Tolerance HUD
        if let Some(tol) = tolerance_hud {
            self.draw_tolerance_hud(cx, cy, tol);
        }

        // 5. Mode switch HUD
        if let Some(active_mode) = mode_hud {
            self.draw_mode_hud(cx, cy, active_mode);
        }
    }

    /// Renders Bounds mode rectangles (active dragging rectangle + Shift-pinned rectangles)
    pub fn draw_bounds(
        &self,
        pinned: &[Rect],
        active: Option<Rect>,
        cursor: Pos2,
        shift_held: bool,
        mode_hud: Option<RulerMode>,
    ) {
        let border_thickness = self.config.bounds.border_thickness as f32;
        let border_color = to_egui_color(&self.config.appearance.bounds_border_color);
        let fill_color = to_egui_color(&self.config.appearance.bounds_fill_color);
        let shadow_color = to_egui_color(&self.config.appearance.line_shadow_color);

        // Render previously pinned rectangles
        for r in pinned {
            let egui_r = EguiRect::from_min_size(Pos2::new(r.x as f32, r.y as f32), Vec2::new(r.width as f32, r.height as f32));
            self.painter.rect_filled(egui_r, 0.0, Color32::from_rgba_unmultiplied(fill_color.r(), fill_color.g(), fill_color.b(), (fill_color.a() as f32 * 0.7) as u8));
            self.painter.rect_stroke(egui_r, 0.0, Stroke::new(border_thickness, border_color), StrokeKind::Middle);

            let w_px = (r.width as f32 * self.scale_x).round() as u32;
            let h_px = (r.height as f32 * self.scale_y).round() as u32;
            let label = self.format_dimensions(w_px, h_px);
            let (cx, cy) = r.center();
            self.draw_badge(Pos2::new(cx as f32, cy as f32), &label);
        }

        // Render active rectangle being drawn
        if let Some(r) = active {
            let egui_r = EguiRect::from_min_size(Pos2::new(r.x as f32, r.y as f32), Vec2::new(r.width as f32, r.height as f32));
            self.painter.rect_stroke(egui_r, 0.0, Stroke::new(border_thickness + 2.0, shadow_color), StrokeKind::Middle);
            self.painter.rect_filled(egui_r, 0.0, fill_color);
            self.painter.rect_stroke(egui_r, 0.0, Stroke::new(border_thickness, border_color), StrokeKind::Middle);

            let w_px = (r.width as f32 * self.scale_x).round() as u32;
            let h_px = (r.height as f32 * self.scale_y).round() as u32;
            let label = self.format_dimensions(w_px, h_px);
            let (cx, cy) = r.center();
            self.draw_badge(Pos2::new(cx as f32, cy as f32), &label);
        } else if self.config.ruler.draw_crosshair {
            let arm = 7.0;
            let ch_shadow = Stroke::new(3.0, shadow_color);
            let ch_primary = Stroke::new(1.5, Color32::from_rgb(240, 240, 240));

            self.painter.line_segment([Pos2::new(cursor.x - arm, cursor.y), Pos2::new(cursor.x + arm, cursor.y)], ch_shadow);
            self.painter.line_segment([Pos2::new(cursor.x, cursor.y - arm), Pos2::new(cursor.x, cursor.y + arm)], ch_shadow);
            self.painter.line_segment([Pos2::new(cursor.x - arm, cursor.y), Pos2::new(cursor.x + arm, cursor.y)], ch_primary);
            self.painter.line_segment([Pos2::new(cursor.x, cursor.y - arm), Pos2::new(cursor.x, cursor.y + arm)], ch_primary);
        }

        if shift_held && self.config.bounds.shift_pins_bounds {
            self.draw_badge(Pos2::new(cursor.x, cursor.y - 30.0), "Shift: Pin Mode");
        }

        if let Some(active_mode) = mode_hud {
            self.draw_mode_hud(cursor.x, cursor.y, active_mode);
        }
    }

    /// Renders a dimension or status badge
    pub fn draw_badge(&self, center: Pos2, text: &str) {
        let font_size = self.config.appearance.label_font_size as f32;
        let font_id = FontId::proportional(font_size);
        let pad_x = self.config.appearance.label_padding_x as f32;
        let pad_y = self.config.appearance.label_padding_y as f32;
        let radius = self.config.appearance.label_border_radius as f32;

        let text_color = to_egui_color(&self.config.appearance.label_text_color);
        let bg_color = to_egui_color(&self.config.appearance.label_bg_color);
        let border_color = to_egui_color(&self.config.appearance.label_border_color);

        let galley = self.painter.layout_no_wrap(text.to_string(), font_id, text_color);
        let badge_w = galley.size().x + pad_x * 2.0;
        let badge_h = galley.size().y + pad_y * 2.0;

        let badge_rect = EguiRect::from_center_size(center, Vec2::new(badge_w, badge_h));

        self.painter.rect_filled(badge_rect, radius, bg_color);
        self.painter.rect_stroke(badge_rect, radius, Stroke::new(1.0, border_color), StrokeKind::Middle);
        self.painter.galley(Pos2::new(badge_rect.min.x + pad_x, badge_rect.min.y + pad_y), galley, text_color);
    }

    /// Renders dynamic tolerance adjustment HUD
    pub fn draw_tolerance_hud(&self, cx: f32, cy: f32, tolerance: u8) {
        let hud_y = (cy - 48.0).max(40.0);
        let label = format!("Sensitivity: {}%", tolerance);
        self.draw_badge(Pos2::new(cx, hud_y), &label);
    }

    /// Renders mode switch notification HUD
    pub fn draw_mode_hud(&self, cx: f32, cy: f32, mode: RulerMode) {
        let hud_y = (cy + 42.0).max(40.0);
        let label = format!("Mode: {}", mode.name());
        self.draw_badge(Pos2::new(cx, hud_y), &label);
    }

    pub fn format_length(&self, pixels: u32) -> String {
        let show_px = self.config.units.show_pixel_unit;
        let px_str = if show_px {
            format!("{} px", pixels)
        } else {
            format!("{}", pixels)
        };

        if self.config.units.secondary_unit != SecondaryUnit::None {
            let sec_str = self.calculate_secondary(pixels);
            format!("{} ({})", px_str, sec_str)
        } else {
            px_str
        }
    }

    pub fn format_dimensions(&self, width: u32, height: u32) -> String {
        let show_px = self.config.units.show_pixel_unit;
        let px_str = if show_px {
            format!("{} × {} px", width, height)
        } else {
            format!("{} × {}", width, height)
        };

        if self.config.units.secondary_unit != SecondaryUnit::None {
            let sec_w = self.calculate_secondary(width);
            let sec_h = self.calculate_secondary(height);
            format!("{} ({} × {})", px_str, sec_w, sec_h)
        } else {
            px_str
        }
    }

    fn calculate_secondary(&self, pixels: u32) -> String {
        let dpi = self.config.units.screen_dpi;
        let inches = pixels as f64 / dpi;

        match self.config.units.secondary_unit {
            SecondaryUnit::None => String::new(),
            SecondaryUnit::Inch => format!("{:.2} in", inches),
            SecondaryUnit::Cm => format!("{:.2} cm", inches * 2.54),
            SecondaryUnit::Mm => format!("{:.1} mm", inches * 25.4),
            SecondaryUnit::Pt => format!("{:.1} pt", inches * 72.0),
        }
    }
}
