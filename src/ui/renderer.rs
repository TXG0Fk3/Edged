use crate::config::{parse_color, Config, SecondaryUnit};
use crate::ruler::bounds::Rect;
use crate::ruler::{EdgeMeasurement, RulerMode};
use cairo::Context;
use pangocairo::pango::{self, FontDescription};
use std::f64::consts::{FRAC_PI_2, PI};

pub struct Renderer<'a> {
    cr: &'a Context,
    config: &'a Config,
}

impl<'a> Renderer<'a> {
    pub fn new(cr: &'a Context, config: &'a Config) -> Self {
        Self { cr, config }
    }

    /// Desenha o overlay semi-transparente escurecido se configurado
    pub fn draw_overlay_background(&self, _width: f64, _height: f64) {
        if self.config.appearance.overlay_opacity > 0.0 {
            self.cr.set_source_rgba(
                0.0,
                0.0,
                0.0,
                self.config.appearance.overlay_opacity.clamp(0.0, 1.0),
            );
            let _ = self.cr.paint();
        }
    }

    /// Desenha as réguas dos modos Spacing, Horizontal ou Vertical
    pub fn draw_spacing_ruler(
        &self,
        measurement: &EdgeMeasurement,
        mode: RulerMode,
        tolerance_hud: Option<u8>,
    ) {
        let cx = measurement.cursor_x as f64;
        let cy = measurement.cursor_y as f64;
        let left_x = measurement.left_x as f64;
        let right_x = measurement.right_x as f64;
        let top_y = measurement.top_y as f64;
        let bottom_y = measurement.bottom_y as f64;

        let feet_size = self.config.ruler.feet_size;
        let draw_feet = self.config.ruler.draw_feet;
        let line_thickness = self.config.ruler.line_thickness;

        let (lr, lg, lb, la) = parse_color(&self.config.appearance.line_color);
        let (sr, sg, sb, sa) = parse_color(&self.config.appearance.line_shadow_color);

        // 1. Linha Horizontal (para Spacing e Horizontal)
        if mode == RulerMode::Spacing || mode == RulerMode::Horizontal {
            // Sombra / Contorno para contraste perfeito em qualquer fundo
            self.cr.set_source_rgba(sr, sg, sb, sa);
            self.cr.set_line_width(line_thickness + 2.0);
            self.cr.move_to(left_x, cy);
            self.cr.line_to(right_x, cy);
            let _ = self.cr.stroke();

            // Pés (sombra)
            if draw_feet {
                self.cr.move_to(left_x, cy - feet_size / 2.0);
                self.cr.line_to(left_x, cy + feet_size / 2.0);
                self.cr.move_to(right_x, cy - feet_size / 2.0);
                self.cr.line_to(right_x, cy + feet_size / 2.0);
                let _ = self.cr.stroke();
            }

            // Linha principal
            self.cr.set_source_rgba(lr, lg, lb, la);
            self.cr.set_line_width(line_thickness);
            self.cr.move_to(left_x, cy);
            self.cr.line_to(right_x, cy);
            let _ = self.cr.stroke();

            // Pés (principal)
            if draw_feet {
                self.cr.move_to(left_x, cy - feet_size / 2.0);
                self.cr.line_to(left_x, cy + feet_size / 2.0);
                self.cr.move_to(right_x, cy - feet_size / 2.0);
                self.cr.line_to(right_x, cy + feet_size / 2.0);
                let _ = self.cr.stroke();
            }

            // Etiqueta da medida horizontal
            let badge_text = self.format_length(measurement.width);
            let badge_x = (left_x + right_x) / 2.0;
            // Se for modo Spacing, coloca o badge horizontal ligeiramente deslocado para cima
            let badge_y = if mode == RulerMode::Spacing {
                cy - 22.0
            } else {
                cy - 18.0
            };
            self.draw_badge(badge_x, badge_y, &badge_text);
        }

        // 2. Linha Vertical (para Spacing e Vertical)
        if mode == RulerMode::Spacing || mode == RulerMode::Vertical {
            // Sombra / Contorno
            self.cr.set_source_rgba(sr, sg, sb, sa);
            self.cr.set_line_width(line_thickness + 2.0);
            self.cr.move_to(cx, top_y);
            self.cr.line_to(cx, bottom_y);
            let _ = self.cr.stroke();

            // Pés (sombra)
            if draw_feet {
                self.cr.move_to(cx - feet_size / 2.0, top_y);
                self.cr.line_to(cx + feet_size / 2.0, top_y);
                self.cr.move_to(cx - feet_size / 2.0, bottom_y);
                self.cr.line_to(cx + feet_size / 2.0, bottom_y);
                let _ = self.cr.stroke();
            }

            // Linha principal
            self.cr.set_source_rgba(lr, lg, lb, la);
            self.cr.set_line_width(line_thickness);
            self.cr.move_to(cx, top_y);
            self.cr.line_to(cx, bottom_y);
            let _ = self.cr.stroke();

            // Pés (principal)
            if draw_feet {
                self.cr.move_to(cx - feet_size / 2.0, top_y);
                self.cr.line_to(cx + feet_size / 2.0, top_y);
                self.cr.move_to(cx - feet_size / 2.0, bottom_y);
                self.cr.line_to(cx + feet_size / 2.0, bottom_y);
                let _ = self.cr.stroke();
            }

            // Etiqueta da medida vertical
            let badge_text = self.format_length(measurement.height);
            let badge_x = if mode == RulerMode::Spacing {
                cx + 28.0
            } else {
                cx + 24.0
            };
            let badge_y = (top_y + bottom_y) / 2.0;
            self.draw_badge(badge_x, badge_y, &badge_text);
        }

        // 3. Mira central no cursor
        if self.config.ruler.draw_crosshair {
            self.draw_crosshair(cx, cy);
        }

        // 4. Indicador HUD de Tolerância (quando a roda do mouse é girada)
        if let Some(tol) = tolerance_hud {
            self.draw_tolerance_hud(cx, cy, tol);
        }
    }

    /// Desenha o modo Bounds (retângulo ativo + retângulos fixados com Shift)
    pub fn draw_bounds(
        &self,
        pinned: &[Rect],
        active: Option<Rect>,
        cursor: (f64, f64),
        shift_held: bool,
    ) {
        let (br, bg, bb, ba) = parse_color(&self.config.appearance.bounds_border_color);
        let (fr, fg, fb, fa) = parse_color(&self.config.appearance.bounds_fill_color);
        let (sr, sg, sb, sa) = parse_color(&self.config.appearance.line_shadow_color);
        let border_thickness = self.config.bounds.border_thickness;

        // Desenha retângulos previamente fixados (Shift)
        for rect in pinned {
            // Preenchimento
            self.cr.set_source_rgba(fr, fg, fb, fa * 0.7);
            self.cr.rectangle(rect.x, rect.y, rect.width, rect.height);
            let _ = self.cr.fill();

            // Borda
            self.cr.set_source_rgba(br, bg, bb, ba * 0.85);
            self.cr.set_line_width(border_thickness);
            self.cr.rectangle(rect.x, rect.y, rect.width, rect.height);
            let _ = self.cr.stroke();

            // Badge central
            let label = self.format_dimensions(rect.width as u32, rect.height as u32);
            let (cx, cy) = rect.center();
            self.draw_badge(cx, cy, &label);
        }

        // Desenha o retângulo que está sendo arrastado atualmente
        if let Some(rect) = active {
            // Sombra da borda
            self.cr.set_source_rgba(sr, sg, sb, sa);
            self.cr.set_line_width(border_thickness + 2.0);
            self.cr.rectangle(rect.x, rect.y, rect.width, rect.height);
            let _ = self.cr.stroke();

            // Preenchimento
            self.cr.set_source_rgba(fr, fg, fb, fa);
            self.cr.rectangle(rect.x, rect.y, rect.width, rect.height);
            let _ = self.cr.fill();

            // Borda principal
            self.cr.set_source_rgba(br, bg, bb, ba);
            self.cr.set_line_width(border_thickness);
            self.cr.rectangle(rect.x, rect.y, rect.width, rect.height);
            let _ = self.cr.stroke();

            // Badge com dimensões
            let label = self.format_dimensions(rect.width as u32, rect.height as u32);
            let (cx, cy) = rect.center();
            self.draw_badge(cx, cy, &label);
        } else {
            // Se não está arrastando, desenha a mira na posição do cursor
            if self.config.ruler.draw_crosshair {
                self.draw_crosshair(cursor.0, cursor.1);
            }
        }

        // Dica visual de Shift caso esteja pressionado
        if shift_held && self.config.bounds.shift_pins_bounds {
            self.draw_badge(cursor.0, cursor.1 - 24.0, "[Shift] Fixar");
        }
    }

    /// Desenha uma mira sutil de 4 pixels no ponto especificado
    fn draw_crosshair(&self, x: f64, y: f64) {
        let (lr, lg, lb, la) = parse_color(&self.config.appearance.line_color);
        self.cr.set_source_rgba(0.0, 0.0, 0.0, 0.7);
        self.cr.set_line_width(3.0);
        self.cr.move_to(x - 5.0, y);
        self.cr.line_to(x + 5.0, y);
        self.cr.move_to(x, y - 5.0);
        self.cr.line_to(x, y + 5.0);
        let _ = self.cr.stroke();

        self.cr.set_source_rgba(lr, lg, lb, la);
        self.cr.set_line_width(1.5);
        self.cr.move_to(x - 4.0, y);
        self.cr.line_to(x + 4.0, y);
        self.cr.move_to(x, y - 4.0);
        self.cr.line_to(x, y + 4.0);
        let _ = self.cr.stroke();
    }

    /// Desenha uma etiqueta numérica (badge/pill) com fundo arredondado e borda elegante
    pub fn draw_badge(&self, center_x: f64, center_y: f64, text: &str) {
        let layout = pangocairo::functions::create_layout(self.cr);
        let mut font_desc = FontDescription::from_string(&self.config.appearance.label_font_family);
        font_desc.set_size((self.config.appearance.label_font_size * pango::SCALE as f64) as i32);
        font_desc.set_weight(pango::Weight::Bold);
        layout.set_font_description(Some(&font_desc));
        layout.set_text(text);

        let (text_w, text_h) = layout.pixel_size();
        let pad_x = self.config.appearance.label_padding_x;
        let pad_y = self.config.appearance.label_padding_y;
        let radius = self.config.appearance.label_border_radius;

        let badge_w = text_w as f64 + pad_x * 2.0;
        let badge_h = text_h as f64 + pad_y * 2.0;
        let badge_x = center_x - badge_w / 2.0;
        let badge_y = center_y - badge_h / 2.0;

        // Fundo do badge
        let (bgr, bgg, bgb, bga) = parse_color(&self.config.appearance.label_bg_color);
        self.cr.set_source_rgba(bgr, bgg, bgb, bga);
        draw_rounded_rect(self.cr, badge_x, badge_y, badge_w, badge_h, radius);
        let _ = self.cr.fill();

        // Borda do badge
        let (br, bg, bb, ba) = parse_color(&self.config.appearance.label_border_color);
        self.cr.set_source_rgba(br, bg, bb, ba);
        self.cr.set_line_width(1.0);
        draw_rounded_rect(self.cr, badge_x, badge_y, badge_w, badge_h, radius);
        let _ = self.cr.stroke();

        // Texto
        let (tr, tg, tb, ta) = parse_color(&self.config.appearance.label_text_color);
        self.cr.set_source_rgba(tr, tg, tb, ta);
        self.cr.move_to(badge_x + pad_x, badge_y + pad_y);
        pangocairo::functions::show_layout(self.cr, &layout);
    }

    /// Desenha o HUD de tolerância dinâmico próximo ao cursor quando o scroll é utilizado
    fn draw_tolerance_hud(&self, cx: f64, cy: f64, tolerance: u8) {
        let text = format!("Tolerância: {}%", tolerance);
        let hud_x = cx + 20.0;
        let hud_y = cy - 35.0;

        let layout = pangocairo::functions::create_layout(self.cr);
        let mut font_desc = FontDescription::from_string(&self.config.appearance.label_font_family);
        font_desc.set_size((11.0 * pango::SCALE as f64) as i32);
        layout.set_font_description(Some(&font_desc));
        layout.set_text(&text);

        let (tw, th) = layout.pixel_size();
        let bar_h = 3.0;
        let pad = 6.0;
        let w = (tw as f64).max(80.0) + pad * 2.0;
        let h = (th as f64) + bar_h + pad * 2.5;

        // Fundo
        self.cr.set_source_rgba(0.06, 0.09, 0.16, 0.95);
        draw_rounded_rect(self.cr, hud_x, hud_y, w, h, 6.0);
        let _ = self.cr.fill();

        // Borda
        self.cr.set_source_rgba(0.28, 0.33, 0.41, 0.9);
        self.cr.set_line_width(1.0);
        draw_rounded_rect(self.cr, hud_x, hud_y, w, h, 6.0);
        let _ = self.cr.stroke();

        // Texto
        self.cr.set_source_rgba(0.97, 0.98, 0.99, 1.0);
        self.cr.move_to(hud_x + pad, hud_y + pad);
        pangocairo::functions::show_layout(self.cr, &layout);

        // Barra de progresso da tolerância
        let bar_y = hud_y + pad + (th as f64) + 3.0;
        let bar_w = w - pad * 2.0;
        let progress_w = bar_w * (tolerance as f64 / 100.0);

        // Fundo da barra
        self.cr.set_source_rgba(0.2, 0.25, 0.33, 1.0);
        self.cr.rectangle(hud_x + pad, bar_y, bar_w, bar_h);
        let _ = self.cr.fill();

        // Preenchimento ativo da barra
        let (lr, lg, lb, _) = parse_color(&self.config.appearance.line_color);
        self.cr.set_source_rgba(lr, lg, lb, 1.0);
        self.cr.rectangle(hud_x + pad, bar_y, progress_w, bar_h);
        let _ = self.cr.fill();
    }

    /// Formata uma medida unidimensional (com unidade primária e secundária opcional)
    fn format_length(&self, px: u32) -> String {
        let mut s = if self.config.units.show_pixel_unit {
            format!("{} px", px)
        } else {
            format!("{}", px)
        };

        if let Some(sec) = self.format_secondary(px as f64) {
            s.push_str(&format!(" ({})", sec));
        }
        s
    }

    /// Formata largura e altura (ex: "124 × 86 px")
    fn format_dimensions(&self, w: u32, h: u32) -> String {
        let mut s = if self.config.units.show_pixel_unit {
            format!("{} × {} px", w, h)
        } else {
            format!("{} × {}", w, h)
        };

        if let (Some(sw), Some(sh)) = (
            self.format_secondary(w as f64),
            self.format_secondary(h as f64),
        ) {
            s.push_str(&format!(" ({} × {})", sw, sh));
        }
        s
    }

    /// Converte pixels para unidade secundária se configurada
    fn format_secondary(&self, px: f64) -> Option<String> {
        let dpi = self.config.units.screen_dpi.max(1.0);
        match self.config.units.secondary_unit {
            SecondaryUnit::None => None,
            SecondaryUnit::Inch => Some(format!("{:.2} in", px / dpi)),
            SecondaryUnit::Cm => Some(format!("{:.2} cm", (px / dpi) * 2.54)),
            SecondaryUnit::Mm => Some(format!("{:.1} mm", (px / dpi) * 25.4)),
            SecondaryUnit::Pt => Some(format!("{:.1} pt", (px / dpi) * 72.0)),
        }
    }
}

/// Helper para desenhar retângulos com cantos arredondados no Cairo
fn draw_rounded_rect(cr: &Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    let r = r.min(w / 2.0).min(h / 2.0);
    cr.new_sub_path();
    cr.arc(x + w - r, y + r, r, -FRAC_PI_2, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, FRAC_PI_2);
    cr.arc(x + r, y + h - r, r, FRAC_PI_2, PI);
    cr.arc(x + r, y + r, r, PI, 3.0 * FRAC_PI_2);
    cr.close_path();
}
