pub mod renderer;

use crate::clipboard::{copy_to_clipboard, format_measurement};
use crate::config::Config;
use crate::ruler::bounds::Rect;
use crate::ruler::{EdgeDetector, EdgeMeasurement, RulerMode};
use crate::ui::renderer::EguiRenderer;
use eframe::egui;
use image::RgbaImage;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Helper to match keyboard keys against configured shortcut string
pub fn matches_key(key: &egui::Key, key_name: &str) -> bool {
    let name = key_name.trim().to_ascii_lowercase();
    match key {
        egui::Key::Tab => name == "tab",
        egui::Key::Escape => name == "esc" || name == "escape",
        egui::Key::Num1 => name == "1",
        egui::Key::Num2 => name == "2",
        egui::Key::Num3 => name == "3",
        egui::Key::Num4 => name == "4",
        egui::Key::B => name == "b",
        egui::Key::S => name == "s",
        egui::Key::H => name == "h",
        egui::Key::V => name == "v",
        egui::Key::Q => name == "q",
        egui::Key::Space => name == "space",
        _ => false,
    }
}

pub struct EdgedApp {
    config: Arc<Config>,
    detector: Arc<EdgeDetector>,
    screen_image: Arc<RgbaImage>,
    texture: Option<egui::TextureHandle>,
    mode: RulerMode,
    tolerance: u8,
    cursor: egui::Pos2,
    drag_start: Option<egui::Pos2>,
    pinned_rects: Vec<Rect>,
    active_rect: Option<Rect>,
    measurement: EdgeMeasurement,
    tolerance_hud_until: Option<Instant>,
    mode_hud_until: Option<Instant>,
    shift_held: bool,
    ctrl_held: bool,
    scroll_accumulator: f32,
}

impl EdgedApp {
    pub fn new(
        config: Arc<Config>,
        detector: Arc<EdgeDetector>,
        screen_image: Arc<RgbaImage>,
        initial_mode: RulerMode,
        initial_tolerance: u8,
    ) -> Self {
        Self {
            config,
            detector,
            screen_image,
            texture: None,
            mode: initial_mode,
            tolerance: initial_tolerance,
            cursor: egui::Pos2::ZERO,
            drag_start: None,
            pinned_rects: Vec::new(),
            active_rect: None,
            measurement: EdgeMeasurement::empty(0, 0),
            tolerance_hud_until: None,
            mode_hud_until: None,
            shift_held: false,
            ctrl_held: false,
            scroll_accumulator: 0.0,
        }
    }
}

impl eframe::App for EdgedApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Ensure immediate repaints for continuous, silky smooth measurement feedback
        ctx.request_repaint();
        ctx.set_cursor_icon(egui::CursorIcon::Crosshair);

        let viewport_rect = ctx.viewport_rect();
        let mut painter = ui.painter().clone();
        painter.set_clip_rect(viewport_rect);

        let img_w = self.screen_image.width() as f32;
        let img_h = self.screen_image.height() as f32;

        let scale_x = if viewport_rect.width() > 1.0 {
            img_w / viewport_rect.width()
        } else {
            ctx.pixels_per_point()
        };
        let scale_y = if viewport_rect.height() > 1.0 {
            img_h / viewport_rect.height()
        } else {
            ctx.pixels_per_point()
        };

        // 1. Draw freeze-frame desktop background perfectly covering the viewport
        let texture = self.texture.get_or_insert_with(|| {
            let size = [
                self.screen_image.width() as usize,
                self.screen_image.height() as usize,
            ];
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, self.screen_image.as_raw());
            ctx.load_texture("desktop_snapshot", color_image, egui::TextureOptions::NEAREST)
        });

        painter.image(
            texture.id(),
            viewport_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );

        if self.config.appearance.overlay_opacity > 0.0 {
            let dim_alpha = (self.config.appearance.overlay_opacity * 255.0).round() as u8;
            painter.rect_filled(viewport_rect, 0.0, egui::Color32::from_black_alpha(dim_alpha));
        }

        // 2. Process modifiers and cursor position
        ctx.input(|i| {
            self.shift_held = i.modifiers.shift;
            self.ctrl_held = i.modifiers.ctrl;
            if let Some(pos) = i.pointer.hover_pos() {
                self.cursor = pos;
            }
        });

        // 3. Process keyboard input
        let mut close_requested = false;
        let mut mode_changed = false;

        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) || i.key_pressed(egui::Key::Q) {
                close_requested = true;
            }

            // Tab mode cycling
            if matches_key(&egui::Key::Tab, &self.config.shortcuts.cycle_mode_key) && i.key_pressed(egui::Key::Tab) {
                if self.shift_held {
                    self.mode = self.mode.prev();
                } else {
                    self.mode = self.mode.next();
                }
                mode_changed = true;
            }

            // Direct mode number keys
            if i.key_pressed(egui::Key::Num1) {
                self.mode = RulerMode::Bounds;
                mode_changed = true;
            } else if i.key_pressed(egui::Key::Num2) {
                self.mode = RulerMode::Spacing;
                mode_changed = true;
            } else if i.key_pressed(egui::Key::Num3) {
                self.mode = RulerMode::Horizontal;
                mode_changed = true;
            } else if i.key_pressed(egui::Key::Num4) {
                self.mode = RulerMode::Vertical;
                mode_changed = true;
            }

            // Alternative mnemonic letters
            if i.key_pressed(egui::Key::B) {
                self.mode = RulerMode::Bounds;
                mode_changed = true;
            } else if i.key_pressed(egui::Key::S) {
                self.mode = RulerMode::Spacing;
                mode_changed = true;
            } else if i.key_pressed(egui::Key::H) {
                self.mode = RulerMode::Horizontal;
                mode_changed = true;
            } else if i.key_pressed(egui::Key::V) {
                self.mode = RulerMode::Vertical;
                mode_changed = true;
            }
        });

        if mode_changed {
            self.mode_hud_until = Some(Instant::now() + Duration::from_millis(1500));
        }

        // 4. Mouse wheel scrolling (discrete tolerance or mode cycle)
        let mut wheel_events = Vec::new();
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::MouseWheel { unit, delta, modifiers, .. } = event {
                    wheel_events.push((*unit, *delta, *modifiers));
                }
            }
        });

        for (unit, delta, modifiers) in wheel_events {
            let is_shift = modifiers.shift || self.shift_held;
            let is_ctrl = modifiers.ctrl || self.ctrl_held;

            // When Shift is held, some compositors map vertical scroll to horizontal delta.x
            let val = if delta.y.abs() >= delta.x.abs() {
                delta.y
            } else {
                delta.x
            };

            if val.abs() < 0.001 {
                continue;
            }

            let steps = match unit {
                egui::MouseWheelUnit::Line | egui::MouseWheelUnit::Page => {
                    val.signum() as i32
                }
                egui::MouseWheelUnit::Point => {
                    self.scroll_accumulator += val;
                    if self.scroll_accumulator.abs() >= 20.0 {
                        let s = self.scroll_accumulator.signum() as i32;
                        self.scroll_accumulator = 0.0;
                        s
                    } else {
                        0
                    }
                }
            };

            if steps == 0 {
                continue;
            }

            let cycles_mode = (is_shift && self.config.shortcuts.shift_scroll_cycles_mode)
                || (is_ctrl && self.config.shortcuts.ctrl_scroll_cycles_mode);

            if cycles_mode {
                if steps > 0 {
                    self.mode = self.mode.next();
                } else {
                    self.mode = self.mode.prev();
                }
                self.mode_hud_until = Some(Instant::now() + Duration::from_millis(1500));
            } else if self.mode != RulerMode::Bounds {
                let step = self.config.ruler.tolerance_step as i32;
                let delta_tol = if steps > 0 { step } else { -step };
                let new_tol = (self.tolerance as i32 + delta_tol).clamp(1, 100) as u8;
                self.tolerance = new_tol;
                self.tolerance_hud_until = Some(Instant::now() + Duration::from_millis(1200));
            }
        }

        // 5. Update edge detection measurements
        if self.mode != RulerMode::Bounds {
            let rel_x = self.cursor.x - viewport_rect.min.x;
            let rel_y = self.cursor.y - viewport_rect.min.y;
            let abs_x = (rel_x * scale_x).round().clamp(0.0, (img_w - 1.0).max(0.0)) as u32;
            let abs_y = (rel_y * scale_y).round().clamp(0.0, (img_h - 1.0).max(0.0)) as u32;
            self.measurement = self.detector.detect_edges(abs_x, abs_y, self.tolerance);
        }

        // 6. Handle clicks and bounds dragging
        let right_clicked = ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Secondary));
        if right_clicked {
            close_requested = true;
        }

        if self.mode == RulerMode::Bounds {
            let primary_pressed = ctx.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary));
            let primary_down = ctx.input(|i| i.pointer.button_down(egui::PointerButton::Primary));
            let primary_released = ctx.input(|i| i.pointer.button_released(egui::PointerButton::Primary));

            if primary_pressed {
                self.drag_start = Some(self.cursor);
            }

            if primary_down && self.drag_start.is_some() {
                let start = self.drag_start.unwrap();
                self.active_rect = Some(Rect::from_points(
                    start.x as f64,
                    start.y as f64,
                    self.cursor.x as f64,
                    self.cursor.y as f64,
                ));
            }

            if primary_released {
                if let Some(rect) = self.active_rect.take() {
                    if self.shift_held && self.config.bounds.shift_pins_bounds {
                        self.pinned_rects.push(rect);
                    } else {
                        let w_px = (rect.width as f32 * scale_x).round() as u32;
                        let h_px = (rect.height as f32 * scale_y).round() as u32;
                        let text = format_measurement(
                            RulerMode::Bounds,
                            w_px,
                            h_px,
                            self.config.clipboard.format,
                        );
                        copy_to_clipboard(&text);
                        if self.config.general.close_on_copy {
                            close_requested = true;
                        }
                    }
                }
                self.drag_start = None;
            }
        } else {
            let primary_clicked = ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary));
            if primary_clicked {
                let text = format_measurement(
                    self.mode,
                    self.measurement.width,
                    self.measurement.height,
                    self.config.clipboard.format,
                );
                copy_to_clipboard(&text);
                if self.config.general.close_on_copy {
                    close_requested = true;
                }
            }
        }

        // 7. Render UI overlays
        let now = Instant::now();
        let tol_hud = self
            .tolerance_hud_until
            .filter(|&until| now < until)
            .map(|_| self.tolerance);
        let mode_hud = self
            .mode_hud_until
            .filter(|&until| now < until)
            .map(|_| self.mode);

        let renderer = EguiRenderer::new(
            &painter,
            self.config.clone(),
            scale_x,
            scale_y,
            viewport_rect.min,
        );

        if self.mode == RulerMode::Bounds {
            renderer.draw_bounds(
                &self.pinned_rects,
                self.active_rect,
                self.cursor,
                self.shift_held,
                mode_hud,
            );
        } else {
            renderer.draw_measurement(
                self.mode,
                self.cursor,
                &self.measurement,
                tol_hud,
                mode_hud,
            );
        }

        if close_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
