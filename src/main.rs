use gdk::prelude::*;
use gio::prelude::*;
use glib::{signal_handler_block, signal_handler_unblock, signal_stop_emission_by_name};
use gtk::prelude::*;

use std::env::args;

use std::cell::RefCell;
use std::collections::HashMap;

mod dproc;
use self::dproc::*;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum StatusContext {
    DataOperation,
    DSPOperation,
}

struct Ui {
    window: gtk::Window,
    load_button: gtk::ToggleButton,
    load_button_clicked_signal: glib::SignalHandlerId,
    process_button: gtk::ToggleButton,
    process_button_clicked_signal: glib::SignalHandlerId,
    display_area: gtk::DrawingArea,
    text_buffer: gtk::TextBuffer,
    text_view: gtk::TextView,
    scrolled_text_view: gtk::ScrolledWindow,
    status_bar: gtk::Statusbar,
    status_bar_cmap: HashMap<StatusContext, u32>,
}

struct DSPState {
    waveform: Vec<f64>,
    processed_data: Vec<f64>,
}

thread_local!(
    static GLOBAL: RefCell<Option<(Ui, DProc, DSPState)>> = RefCell::new(None);
);

fn ui_init() {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Rust DSP Example");
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(400, 300);

    // Create the top toolbar
    let toolbar = gtk::Toolbar::new();
    toolbar.set_show_arrow(false);

    // Create load button
    let load_button = gtk::ToggleButton::new();
    load_button.set_tooltip_text("Load data");
    let load_image = gtk::Image::new_from_file("resources/file-download.png");
    load_button.set_image(&load_image);
    //    load_button.set_sensitive(false);
    let load_button_container = gtk::ToolItem::new();
    load_button_container.add(&load_button);
    toolbar.add(&load_button_container);

    // Create process button
    let process_button = gtk::ToggleButton::new();
    process_button.set_tooltip_text("Process data");
    let process_image = gtk::Image::new_from_file("resources/chart-bell-curve.png");
    process_button.set_image(&process_image);
    process_button.set_sensitive(false);
    let process_button_container = gtk::ToolItem::new();
    process_button_container.add(&process_button);
    toolbar.add(&process_button_container);

    // Create display area
    let display_area = gtk::DrawingArea::new();
    display_area.show();
    display_area.set_size_request(256, 256);

    // Create text view
    let text_buffer = gtk::TextBuffer::new(None);
    let text_view = gtk::TextView::new_with_buffer(&text_buffer);
    text_view.set_wrap_mode(gtk::WrapMode::Char);
    text_view.set_cursor_visible(false);

    let scrolled_text_view = gtk::ScrolledWindow::new(None, None);
    scrolled_text_view.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scrolled_text_view.add(&text_view);

    // Add a status bar
    let status_bar = gtk::Statusbar::new();
    // A context id for data operations
    let context_id_data_ops = status_bar.get_context_id("data operations");
    // A context id for dsp operations
    let context_id_dsp_ops = status_bar.get_context_id("dsp operations");
    let status_bar_cmap: HashMap<StatusContext, u32> = [
        (StatusContext::DataOperation, context_id_data_ops),
        (StatusContext::DSPOperation, context_id_dsp_ops),
    ]
    .iter()
    .cloned()
    .collect();

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.pack_start(&toolbar, false, false, 0);
    vbox.pack_start(&display_area, true, true, 0);
    vbox.pack_start(&scrolled_text_view, true, true, 0);
    vbox.pack_start(&status_bar, false, false, 0);
    window.add(&vbox);

    // Make sure all desired widgets are visible.
    window.show_all();

    // Set CSS styles for the entire application.
    let css_provider = gtk::CssProvider::new();
    let display = gdk::Display::get_default().expect("Couldn't open default GDK display");
    let screen = display.get_default_screen();
    gtk::StyleContext::add_provider_for_screen(
        &screen,
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    css_provider
        .load_from_path("resources/style.css")
        .expect("Failed to load CSS stylesheet");

    // Connect signals
    display_area.connect_draw(|w, c| {
        GLOBAL.with(|global| {
            if let Some((ref ui, ref dproc, ref dspstate)) = *global.borrow() {
                let style_context = w.get_style_context().unwrap();
                let foreground = style_context.get_color(w.get_state_flags());
                let mut background = foreground.clone();
                background.alpha *= 0.3;
                let background = background;
                let width = w.get_allocated_width() as f64;
                let height = w.get_allocated_height() as f64;
                let two_pi = 2.0 * std::f64::consts::PI;

                println!("Waveform: {}", dspstate.waveform.len());

                let mut x = 0.0f64;
                for y in &dspstate.waveform {
                    c.line_to(x, *y * 10.0);
                    x += 10.0;
                }
                c.stroke();
            } else {

            }
        });
        Inhibit(false)
    });

    let load_button_clicked_signal = load_button.connect_clicked(move |s| {
        println!("Load button clicked!");
        if s.get_active() {
            println!("Load button activated!");
            GLOBAL.with(|global| {
                // Deconstructs ui, dproc as references
                // Deconstructs dspstate as mutable reference
                if let Some((ref ui, ref dproc, ref mut dspstate)) = *global.borrow_mut() {
                    println!(
                        "dspstate in 'connect_clicked': {:p}",
                        dspstate as *const DSPState
                    );

                    dspstate.waveform = vec![
                        0.0, 0.0, 0.2, 0.4, 0.8, 1.0, 0.6, 0.2, 0.16, 0.1, 0.04, 0.0, 0.0,
                    ];

                    println!(
                        "address of dspstate.waveform: {:p}",
                        &(dspstate.waveform) as *const Vec<f64>
                    );

                    match dproc.send_data_cmd(&dspstate.waveform) {
                        Err(GeneralError::SendError(cmd)) => {
                            println!("Send error! {:?}", cmd);
                        }
                        Ok(_) => {
                            println!("Data sending OK!");
                        }
                    }
                }
            })
        } else {
            println!("Load button already activated, deactivating!");
        }
    });

    let process_button_clicked_signal = process_button.connect_clicked(move |s| {
        if s.get_active() {

        } else {

        }
    });

    // -------------------
    let ui = Ui {
        window,
        load_button,
        load_button_clicked_signal,
        process_button,
        process_button_clicked_signal,
        display_area,
        text_buffer,
        text_view,
        scrolled_text_view,
        status_bar,
        status_bar_cmap,
    };

    let mut dspstate = DSPState {
        waveform: Vec::new(),
        processed_data: Vec::new(),
    };

    println!("dspstate in 'ui_init': {:p}", &dspstate as *const DSPState);
    {
        let mut data = &dspstate;
        println!("dspstate in 'ui_init': {:p}", data as *const DSPState);
    }

    GLOBAL.with(move |global| {
        *global.borrow_mut() = Some((
            ui,
            DProc::new(|| {
                glib::idle_add(receive);
            }),
            dspstate,
        ));
    });

    GLOBAL.with(|global| {
        if let Some((ref ui, ref dproc, ref dspstate)) = *global.borrow_mut() {
            println!("dspstate in 'GLOBAL': {:p}", dspstate);
            println!(
                "address of dspstate.waveform: {:p}",
                &(dspstate.waveform) as *const Vec<f64>
            );
        }
    });
}

fn receive() -> glib::Continue {
    Continue(false)
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    ui_init();
    GLOBAL.with(|global| {
        if let Some((ref ui, _, _)) = *global.borrow() {
            // Set deleting the window to close the entire application
            ui.window.connect_delete_event(|_, _| {
                gtk::main_quit();
                Inhibit(false)
            });
        }
    });

    // Start our GUI main loop
    gtk::main();
}
