pub mod renderer;

use crate::clipboard::{copy_to_clipboard, format_measurement};
use crate::config::Config;
use crate::ruler::bounds::Rect;
use crate::ruler::{EdgeDetector, EdgeMeasurement, RulerMode};
use crate::ui::renderer::Renderer;
use cairo::ImageSurface;
use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4_layer_shell::LayerShell;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

struct AppState {
    mode: RulerMode,
    tolerance: u8,
    cursor: (f64, f64),
    shift_held: bool,
    drag_start: Option<(f64, f64)>,
    pinned_rects: Vec<Rect>,
    measurement: EdgeMeasurement,
    tolerance_hud_until: Option<Instant>,
}

pub fn create_and_show_overlay(
    app: &gtk4::Application,
    config: Arc<Config>,
    detector: Arc<EdgeDetector>,
    surface: Arc<ImageSurface>,
    mode: RulerMode,
    initial_tolerance: u8,
) {
    let display = gdk::Display::default().expect("Wayland display not found");

    // CSS styling for full window transparency
    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data("window { background-color: transparent; }");
    gtk4::style_context_add_provider_for_display(
        &display,
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let state = Rc::new(RefCell::new(AppState {
        mode,
        tolerance: initial_tolerance,
        cursor: (0.0, 0.0),
        shift_held: false,
        drag_start: None,
        pinned_rects: Vec::new(),
        measurement: EdgeMeasurement::empty(0, 0),
        tolerance_hud_until: None,
    }));

    let layer_shell_supported = gtk4_layer_shell::is_supported();
    let monitors = display.monitors();
    let monitor_count = monitors.n_items();

    if monitor_count == 0 {
        build_overlay_window(
            app,
            None,
            config.clone(),
            detector.clone(),
            surface.clone(),
            state.clone(),
            layer_shell_supported,
        );
    } else {
        for i in 0..monitor_count {
            if let Some(item) = monitors.item(i) {
                if let Ok(monitor) = item.downcast::<gdk::Monitor>() {
                    build_overlay_window(
                        app,
                        Some(monitor),
                        config.clone(),
                        detector.clone(),
                        surface.clone(),
                        state.clone(),
                        layer_shell_supported,
                    );
                }
            }
        }
    }
}

fn build_overlay_window(
    app: &gtk4::Application,
    monitor: Option<gdk::Monitor>,
    config: Arc<Config>,
    detector: Arc<EdgeDetector>,
    surface: Arc<ImageSurface>,
    state: Rc<RefCell<AppState>>,
    layer_shell_supported: bool,
) {
    let window = gtk4::ApplicationWindow::new(app);
    window.set_decorated(false);

    if layer_shell_supported {
        window.init_layer_shell();
        window.set_layer(gtk4_layer_shell::Layer::Overlay);
        window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
        window.set_anchor(gtk4_layer_shell::Edge::Top, true);
        window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
        window.set_anchor(gtk4_layer_shell::Edge::Left, true);
        window.set_anchor(gtk4_layer_shell::Edge::Right, true);
        window.set_namespace(Some("edged"));

        if let Some(ref m) = monitor {
            window.set_monitor(Some(m));
        }
    } else {
        // Fallback for compositors that do not implement layer-shell (e.g. GNOME Mutter)
        if let Some(ref m) = monitor {
            window.fullscreen_on_monitor(m);
        } else {
            window.fullscreen();
        }
    }

    // Set cursor to crosshair
    let cursor = gdk::Cursor::from_name("crosshair", None);
    window.set_cursor(cursor.as_ref());

    let drawing_area = gtk4::DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);

    // Monitor geometry for global desktop coordinate mapping
    let (monitor_x, monitor_y) = if let Some(ref m) = monitor {
        let geom = m.geometry();
        (geom.x() as f64, geom.y() as f64)
    } else {
        (0.0, 0.0)
    };

    // Cairo Drawing
    {
        let config = config.clone();
        let state = state.clone();
        let surface = surface.clone();
        let win = window.clone();

        drawing_area.set_draw_func(move |_, cr, width, height| {
            let st = state.borrow();
            let renderer = Renderer::new(cr, &config);
            let scale = win.scale_factor() as f64;

            // Render background desktop screenshot + overlay tint
            renderer.draw_background(
                &surface,
                monitor_x,
                monitor_y,
                scale,
                width as f64,
                height as f64,
            );

            if st.mode == RulerMode::Bounds {
                let active = st.drag_start.map(|start| {
                    Rect::from_points(start.0, start.1, st.cursor.0, st.cursor.1)
                });
                renderer.draw_bounds(
                    &st.pinned_rects,
                    active,
                    st.cursor,
                    scale,
                    st.shift_held,
                );
            } else {
                let hud_val = st.tolerance_hud_until.and_then(|until| {
                    if Instant::now() < until {
                        Some(st.tolerance)
                    } else {
                        None
                    }
                });
                renderer.draw_spacing_ruler(
                    &st.measurement,
                    st.mode,
                    monitor_x,
                    monitor_y,
                    scale,
                    hud_val,
                );
            }
        });
    }

    // Mouse Motion Controller
    {
        let state = state.clone();
        let detector = detector.clone();
        let da = drawing_area.clone();
        let win = window.clone();
        let motion = gtk4::EventControllerMotion::new();

        motion.connect_motion(move |_, x, y| {
            let mut st = state.borrow_mut();
            st.cursor = (x, y);

            let scale = win.scale_factor() as f64;
            let abs_x = ((monitor_x + x) * scale).round() as u32;
            let abs_y = ((monitor_y + y) * scale).round() as u32;

            if st.mode != RulerMode::Bounds {
                st.measurement = detector.detect_edges(abs_x, abs_y, st.tolerance);
            }

            da.queue_draw();
        });

        window.add_controller(motion);
    }

    // Mouse Click Controller
    {
        let state = state.clone();
        let config = config.clone();
        let da = drawing_area.clone();
        let app = app.clone();
        let win = window.clone();
        let click = gtk4::GestureClick::new();
        click.set_button(0); // Capture all mouse buttons

        click.connect_pressed({
            let state = state.clone();
            move |gesture, _, x, y| {
                let button = gesture.current_button();
                if button == gdk::BUTTON_PRIMARY {
                    let mut st = state.borrow_mut();
                    if st.mode == RulerMode::Bounds {
                        st.drag_start = Some((x, y));
                    }
                }
            }
        });

        click.connect_released(move |gesture, _, x, y| {
            let button = gesture.current_button();

            if button == gdk::BUTTON_SECONDARY {
                // Right click cancels and exits
                app.quit();
                return;
            }

            if button == gdk::BUTTON_PRIMARY {
                let mut st = state.borrow_mut();
                let scale = win.scale_factor() as f64;

                if st.mode == RulerMode::Bounds {
                    if let Some(start) = st.drag_start {
                        let rect = Rect::from_points(start.0, start.1, x, y);
                        let w_px = (rect.width * scale).round() as u32;
                        let h_px = (rect.height * scale).round() as u32;

                        if st.shift_held && config.bounds.shift_pins_bounds {
                            // Holding Shift: keep rectangle pinned to screen and continue measuring
                            st.pinned_rects.push(rect);
                            st.drag_start = None;
                            da.queue_draw();
                            return;
                        }

                        // Normal release: copy bounds measurement to clipboard and exit
                        let text = format_measurement(
                            RulerMode::Bounds,
                            w_px,
                            h_px,
                            config.clipboard.format,
                        );
                        copy_to_clipboard(&text);

                        if config.general.close_on_copy {
                            app.quit();
                        }
                    }
                } else {
                    // Spacing / Horizontal / Vertical modes: copy measurement and exit
                    let text = format_measurement(
                        st.mode,
                        st.measurement.width,
                        st.measurement.height,
                        config.clipboard.format,
                    );
                    copy_to_clipboard(&text);

                    if config.general.close_on_copy {
                        app.quit();
                    }
                }
            }
        });

        window.add_controller(click);
    }

    // Mouse Scroll Wheel Controller (Dynamic tolerance adjustment)
    {
        let state = state.clone();
        let detector = detector.clone();
        let config = config.clone();
        let da = drawing_area.clone();
        let win = window.clone();
        let scroll = gtk4::EventControllerScroll::new(gtk4::EventControllerScrollFlags::VERTICAL);

        scroll.connect_scroll(move |_, _, dy| {
            let mut st = state.borrow_mut();

            if st.mode != RulerMode::Bounds {
                let step = config.ruler.tolerance_step as i32;
                let current = st.tolerance as i32;
                // Scroll up (dy < 0) increases tolerance; scroll down (dy > 0) decreases tolerance
                let new_tol = if dy < 0.0 {
                    (current + step).min(100)
                } else if dy > 0.0 {
                    (current - step).max(1)
                } else {
                    current
                } as u8;

                st.tolerance = new_tol;
                st.tolerance_hud_until = Some(Instant::now() + Duration::from_millis(1500));

                let scale = win.scale_factor() as f64;
                let abs_x = ((monitor_x + st.cursor.0) * scale).round() as u32;
                let abs_y = ((monitor_y + st.cursor.1) * scale).round() as u32;
                st.measurement = detector.detect_edges(abs_x, abs_y, st.tolerance);

                da.queue_draw();
            }

            glib::Propagation::Stop
        });

        window.add_controller(scroll);
    }

    // Keyboard Controller (Esc, Shift, etc.)
    {
        let state = state.clone();
        let da = drawing_area.clone();
        let app = app.clone();
        let key = gtk4::EventControllerKey::new();

        key.connect_key_pressed({
            let state = state.clone();
            let da = da.clone();
            let app = app.clone();
            move |_, keyval, _, _| {
                match keyval {
                    gdk::Key::Escape | gdk::Key::q | gdk::Key::Q => {
                        app.quit();
                        glib::Propagation::Stop
                    }
                    gdk::Key::Shift_L | gdk::Key::Shift_R => {
                        let mut st = state.borrow_mut();
                        st.shift_held = true;
                        da.queue_draw();
                        glib::Propagation::Proceed
                    }
                    _ => glib::Propagation::Proceed,
                }
            }
        });

        key.connect_key_released({
            let state = state.clone();
            let da = da.clone();
            move |_, keyval, _, _| {
                match keyval {
                    gdk::Key::Shift_L | gdk::Key::Shift_R => {
                        let mut st = state.borrow_mut();
                        st.shift_held = false;
                        da.queue_draw();
                    }
                    _ => {}
                }
            }
        });

        window.add_controller(key);
    }

    window.set_child(Some(&drawing_area));
    window.present();
}
