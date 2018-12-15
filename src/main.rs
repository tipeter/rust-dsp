use gio::prelude::*;
use gtk::prelude::*;
use glib::{signal_handler_block, signal_handler_unblock, signal_stop_emission_by_name};

use std::env::args;

use std::cell::RefCell;
use std::collections::HashMap;

mod dproc;
use self::dproc::DProc;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum StatusContext {
    DataOperation,
    DSPOperation,
}

struct Ui {
    window: gtk::Window,
    process_button: gtk::ToggleButton,
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
    static GLOBAL:RefCell<Option<(Ui, DProc, DSPState)>> = RefCell::new(None);
);

fn ui_init() {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Rust DSP Example");
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(400, 300);

    // Create the top toolbar
    let toolbar = gtk::Toolbar::new();
    toolbar.set_show_arrow(false);

    // Create process button
    let process_button = gtk::ToggleButton::new();
    process_button.set_tooltip_text("Process data");
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

    let vbox =gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.pack_start(&toolbar, false, false, 0);
    vbox.pack_start(&status_bar, false, false, 0);
    vbox.pack_start(&display_area, false, false, 0);
    vbox.pack_start(&scrolled_text_view, true, true, 0);
    window.add(&vbox);

    // -------------------
    let ui = Ui {
        window,
        process_button,
        display_area,
        text_buffer,
        text_view,
        scrolled_text_view,
        status_bar,
        status_bar_cmap,
    };

    let dspstate = DSPState {
        waveform: vec![0.0, 0.0, 0.2, 0.4, 0.8, 1.0, 0.6, 0.2, 0.16, 0.1, 0.04, 0.0, 0.0],
        processed_data: Vec::new(),
    };

    GLOBAL.with(move |global| {
        *global.borrow_mut() = Some((
            ui,
            DProc::new(|| {
                glib::idle_add(receive);
            }),
            dspstate,
        ));
    });
}

fn receive() -> glib::Continue {
    Continue(false)
}

fn main() {
}
