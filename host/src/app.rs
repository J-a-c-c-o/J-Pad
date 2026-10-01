use std::error::Error;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;

use eframe::egui;

use crate::device::{Device, DeviceError, DeviceStatus};
use crate::keycode_converter::{self, keycode_to_expr};
use crate::keycodes;
use crate::layout::{Layout, MAX_STEP_KEYS, MAX_STEPS, NUM_MACROS, Step};

const KEY_SIZE: egui::Vec2 = egui::vec2(138.0, 100.0);
const KEY_SPACING: egui::Vec2 = egui::vec2(10.0, 10.0);
const ROTARY_KEY: usize = NUM_MACROS - 1;
const EDITOR_WIDTH: f32 = 400.0;
const PAD_RADIUS: u8 = 10;

const EXPR_HINT: &str =
    "comma separated keycodes, e.g. KC_A, LCTL(KC_C), SWITCH_LAYER_1, 0x0104, MS_BTN1";

#[derive(Debug, Clone)]
enum Request {
    Connect,
    Disconnect,
    Refresh,
    Write(Box<Layout>),
    Save(Box<Layout>),
    Reset,
}

enum Event {
    Status(String),
    Error(String),
    Connected(String),
    Disconnected,
    StatusReport(Box<DeviceStatus>),
    Layout(Box<Layout>),
    Written(Box<Layout>),
}

struct DeviceWorker {
    requests: Option<Sender<Request>>,
    events: Receiver<Event>,
    thread: Option<JoinHandle<()>>,
}

impl DeviceWorker {
    fn start() -> Self {
        let (request_tx, request_rx) = std::sync::mpsc::channel();
        let (event_tx, event_rx) = std::sync::mpsc::channel();

        let thread = std::thread::Builder::new()
            .name("jpad-device".to_string())
            .spawn(move || {
                let mut device: Option<Device> = None;
                while let Ok(request) = request_rx.recv() {
                    for event in execute(&mut device, request) {
                        if event_tx.send(event).is_err() {
                            return;
                        }
                    }
                }
            })
            .expect("the device worker thread should start");

        Self {
            requests: Some(request_tx),
            events: event_rx,
            thread: Some(thread),
        }
    }

    fn send(&self, request: Request) {
        if let Some(requests) = &self.requests {
            let _ = requests.send(request);
        }
    }

    fn poll(&self) -> Vec<Event> {
        let mut events = Vec::new();
        while let Ok(event) = self.events.try_recv() {
            events.push(event);
        }
        events
    }
}

impl Drop for DeviceWorker {
    fn drop(&mut self) {
        if let Some(requests) = self.requests.take() {
            let _ = requests.send(Request::Disconnect);
        }

        if let Some(thread) = self.thread.take() {
            thread.join().ok();
        }
    }
}

fn with_device<T>(
    device: &mut Option<Device>,
    action: impl FnOnce(&mut Device) -> Result<T, DeviceError>,
) -> Result<T, DeviceError> {
    device
        .as_mut()
        .ok_or(DeviceError::NotConnected)
        .and_then(action)
}

fn execute(device: &mut Option<Device>, request: Request) -> Vec<Event> {
    let result: Result<Vec<Event>, DeviceError> = match request {
        Request::Connect => match Device::open() {
            Err(error) => Err(error),
            Ok(opened) => {
                let description = opened.description().to_string();
                *device = Some(opened);

                let read = with_device(device, |device| {
                    let layout = device.read_layout()?;
                    let status = device.ping()?;
                    Ok((layout, status))
                });

                let mut events = vec![Event::Connected(description)];
                match read {
                    Ok((layout, status)) => events.extend([
                        Event::Layout(Box::new(layout)),
                        Event::StatusReport(Box::new(status)),
                        Event::Status("Read the layout from the device".to_string()),
                    ]),
                    Err(error) => events.push(Event::Error(error.to_string())),
                }
                Ok(events)
            }
        },
        Request::Disconnect => {
            *device = None;
            Ok(vec![Event::Disconnected])
        }
        Request::Refresh => with_device(device, |device| {
            let layout = device.read_layout()?;
            let status = device.ping()?;
            Ok(vec![
                Event::Layout(Box::new(layout)),
                Event::StatusReport(Box::new(status)),
                Event::Status("Read the layout from the device".to_string()),
            ])
        }),
        Request::Write(layout) => with_device(device, |device| {
            device.write_layout(&layout)?;
            let status = device.ping()?;
            Ok(vec![
                Event::Written(layout),
                Event::StatusReport(Box::new(status)),
                Event::Status(
                    "Layout written to the device, it is lost on a reboot until you save"
                        .to_string(),
                ),
            ])
        }),
        Request::Save(layout) => with_device(device, |device| {
            device.write_layout(&layout)?;
            device.save_layout()?;
            let status = device.ping()?;
            Ok(vec![
                Event::Written(layout),
                Event::StatusReport(Box::new(status)),
                Event::Status(
                    "Layout saved in flash, it survives a reboot from now on".to_string(),
                ),
            ])
        }),
        Request::Reset => with_device(device, |device| {
            device.reset_layout()?;
            let layout = device.read_layout()?;
            let status = device.ping()?;
            Ok(vec![
                Event::Layout(Box::new(layout)),
                Event::StatusReport(Box::new(status)),
                Event::Status(
                    "Device reset to the factory defaults, press Save to keep them".to_string(),
                ),
            ])
        }),
    };

    match result {
        Ok(events) => events,
        Err(error) => vec![Event::Error(error.to_string())],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Selection {
    Key(usize),
    Rotary,
}

struct KeyEditor {
    layer: usize,
    index: usize,
    steps: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaptureTarget {
    Step(usize),
    Counterclockwise,
    Clockwise,
}

struct EncoderEditor {
    layer: usize,
    counterclockwise: String,
    clockwise: String,
    error: Option<String>,
}

pub struct JPadApp {
    worker: DeviceWorker,
    layout: Layout,
    device_layout: Layout,
    connected: Option<String>,
    device_status: Option<DeviceStatus>,
    selected: Option<Selection>,
    layer: usize,
    key_editor: Option<KeyEditor>,
    encoder_editor: Option<EncoderEditor>,
    busy: bool,
    status: String,
    error: Option<String>,

    confirm: Option<Request>,
    layout_known: bool,
    styled: bool,
    recording: bool,
    captured: Option<String>,
    target: CaptureTarget,
    layer_pick: usize,
    help_open: bool,
    help_filter: String,
}

impl Default for JPadApp {
    fn default() -> Self {
        let layout = Layout::empty();
        Self {
            worker: DeviceWorker::start(),
            layout: layout.clone(),
            device_layout: layout,
            connected: None,
            device_status: None,
            selected: None,
            layer: 0,
            key_editor: None,
            encoder_editor: None,
            busy: false,
            status: "Not connected".to_string(),
            error: None,
            confirm: None,
            layout_known: false,
            styled: false,
            recording: false,
            captured: None,
            target: CaptureTarget::Step(0),
            layer_pick: 0,
            help_open: false,
            help_filter: String::new(),
        }
    }
}

impl eframe::App for JPadApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.apply_style(&ctx);

        for event in self.worker.poll() {
            self.busy = false;
            self.handle(event);
        }

        self.poll_capture(ui.ctx());
        if self.help_open {
            self.help_window(ui.ctx());
        }

        self.header(ui);
        self.editor_panel(ui);
        self.footer(ui);
        self.pad_panel(ui);
    }
}

impl JPadApp {
    fn apply_style(&mut self, ctx: &egui::Context) {
        if self.styled {
            return;
        }
        self.styled = true;

        ctx.global_style_mut(|style| {
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            style.spacing.button_padding = egui::vec2(12.0, 7.0);
            style.spacing.window_margin = egui::Margin::same(14);
            style.spacing.menu_margin = egui::Margin::same(10);
            style.spacing.slider_width = 220.0;
            style.spacing.interact_size.y = 30.0;
            style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(7);
            style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(7);
            style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(7);
            style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(7);
            style.visuals.widgets.open.corner_radius = egui::CornerRadius::same(7);
            style.visuals.window_corner_radius = egui::CornerRadius::same(10);
        });
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("header")
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().panel_fill)
                    .inner_margin(egui::epaint::MarginF32::symmetric(20.0, 14.0)),
            )
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("J-Pad")
                                .size(22.0)
                                .strong()
                                .color(ui.visuals().text_color()),
                        );
                        ui.small("layer editor");
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        self.connection_pill(ui);
                        ui.add_space(4.0);
                        self.save_pill(ui);
                        ui.add_space(4.0);
                        let help = ui
                            .selectable_label(self.help_open, "Keycodes")
                            .on_hover_text("browse every keycode the board understands");
                        if help.clicked() {
                            self.help_open = !self.help_open;
                        }
                    });
                });
                ui.add_space(8.0);
                ui.separator();
            });
    }

    fn connection_pill(&self, ui: &mut egui::Ui) {
        let (text, color) = match (&self.connected, &self.error) {
            (Some(_), Some(_)) => ("error".to_string(), ui.visuals().error_fg_color),
            (Some(name), None) => (name.clone(), ui.visuals().text_color()),
            (None, Some(_)) => ("error".to_string(), ui.visuals().error_fg_color),
            (None, None) => ("not connected".to_string(), ui.visuals().weak_text_color()),
        };

        let text = if self.busy {
            format!("{text} ...")
        } else {
            text
        };

        egui::Frame::new()
            .fill(ui.visuals().widgets.inactive.weak_bg_fill)
            .corner_radius(egui::CornerRadius::same(9))
            .inner_margin(egui::epaint::MarginF32::symmetric(11.0, 5.0))
            .show(ui, |ui| {
                ui.label(egui::RichText::new(text).color(color));
            });
    }

    fn save_pill(&self, ui: &mut egui::Ui) {
        let Some(status) = &self.device_status else {
            return;
        };

        let (text, color) = match status.unsaved_changes {
            true => ("not saved", ui.visuals().warn_fg_color),
            false => ("saved", ui.visuals().text_color()),
        };

        let frame = egui::Frame::new()
            .fill(ui.visuals().extreme_bg_color)
            .corner_radius(egui::CornerRadius::same(9))
            .inner_margin(egui::epaint::MarginF32::symmetric(10.0, 4.0));
        frame.show(ui, |ui| {
            ui.small(egui::RichText::new(text).color(color));
        });
    }

    fn editor_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::right("editor")
            .exact_size(EDITOR_WIDTH)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().extreme_bg_color)
                    .inner_margin(egui::epaint::MarginF32::symmetric(18.0, 16.0)),
            )
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| match self.selected {
                        Some(Selection::Key(_)) => self.key_editor_ui(ui),
                        Some(Selection::Rotary) => self.encoder_editor_ui(ui),
                        None => self.placeholder_ui(ui),
                    });
            });
    }

    fn placeholder_ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.label(egui::RichText::new("Nothing selected").size(16.0).strong());
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new("Click a key to edit its steps, or the knob in the top right cell for the rotary encoder.")
                .color(ui.visuals().weak_text_color()),
        );

        ui.add_space(18.0);
        ui.separator();
        ui.add_space(10.0);

        ui.label(egui::RichText::new("Device").strong());
        ui.add_space(6.0);
        if let Some(status) = &self.device_status {
            ui.label(status.summary());
            if let Some(error) = &self.error {
                ui.colored_label(ui.visuals().error_fg_color, error);
            }
        } else if let Some(error) = &self.error {
            ui.colored_label(ui.visuals().error_fg_color, error);
        } else {
            ui.label(
                egui::RichText::new("Connect to the board to read the layout it is running.")
                    .color(ui.visuals().weak_text_color()),
            );
        }
    }

    fn key_editor_ui(&mut self, ui: &mut egui::Ui) {
        if !matches!(self.selected, Some(Selection::Key(_))) {
            return;
        }
        let Some(editor) = &mut self.key_editor else {
            return;
        };
        let index = editor.index;
        let layer = editor.layer;

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("Key K{}", index + 1))
                    .size(18.0)
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = match self.recording {
                    true => "Recording",
                    false => "Record keys",
                };
                let button = egui::Button::new(label)
                    .selected(self.recording)
                    .min_size(egui::vec2(96.0, 0.0));
                let response = ui
                    .add(button)
                    .on_hover_text("press keys on your keyboard to turn them into steps");
                if response.clicked() {
                    self.recording = !self.recording;
                    self.captured = None;
                }
                if self.recording {
                    ui.colored_label(ui.visuals().error_fg_color, "press keys");
                }
            });
        });
        ui.small(format!("layer {layer}"));
        ui.add_space(12.0);

        if self.recording {
            ui.label(
                egui::RichText::new("every key press adds a step, hold Ctrl, Shift, Alt or Super for a combination, Esc stops")
                    .color(ui.visuals().error_fg_color),
            );
            ui.add_space(8.0);
        }

        ui.horizontal(|ui| {
            ui.label("Delay");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(
                    egui::DragValue::new(&mut self.layout.layers[layer].macros[index].delay_ms)
                        .range(0..=5000)
                        .suffix(" ms"),
                );
            });
        });
        ui.horizontal(|ui| {
            ui.label("Auto repeat");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(
                    egui::DragValue::new(&mut self.layout.layers[layer].macros[index].repeat_ms)
                        .range(0..=5000)
                        .suffix(" ms"),
                );
            });
        });
        ui.small(
            egui::RichText::new("0 turns auto repeat off").color(ui.visuals().weak_text_color()),
        );

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Steps").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let can_add = editor.steps.len() < MAX_STEPS;
                if ui
                    .add_enabled(can_add, egui::Button::new("Add empty step"))
                    .clicked()
                {
                    editor.steps.push(String::new());
                }
            });
        });

        let mut remove = None;
        let mut error = None;

        if let Some(expression) = self.captured.take() {
            let index = match self.target {
                CaptureTarget::Step(index) if index < editor.steps.len() => {
                    match editor.steps[index].trim().is_empty() {
                        true => index,
                        false => {
                            editor.steps.push(String::new());
                            editor.steps.len() - 1
                        }
                    }
                }
                _ => {
                    editor.steps.push(String::new());
                    editor.steps.len() - 1
                }
            };

            editor.steps[index] = expression;
            self.target = CaptureTarget::Step(index);
        }

        if editor.steps.is_empty() {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("No steps, this key does nothing.")
                    .color(ui.visuals().weak_text_color()),
            );
        }

        let gutter = 28.0;
        let remove_width = 32.0;
        let row_height = 26.0;
        let row = ui.available_width();
        let spacing = ui.spacing().item_spacing.x;
        let field_width = (row - gutter - remove_width - 2.0 * spacing - 8.0).max(80.0);

        for (step, text) in editor.steps.iter_mut().enumerate() {
            let focused = self.target == CaptureTarget::Step(step);
            let marker = match focused {
                true => ">",
                false => "",
            };
            let number = egui::RichText::new(format!("{marker} {}", step + 1))
                .color(match focused {
                    true => ui.visuals().text_color(),
                    false => ui.visuals().weak_text_color(),
                })
                .monospace();

            ui.horizontal(|ui| {
                ui.add_sized([gutter, row_height], egui::Label::new(number));
                let field = egui::TextEdit::singleline(text)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY);
                if ui
                    .add_sized([field_width, row_height], field)
                    .on_hover_text(EXPR_HINT)
                    .clicked()
                {
                    self.target = CaptureTarget::Step(step);
                }
                if ui
                    .add_sized([remove_width, row_height], egui::Button::new("x"))
                    .on_hover_text("remove this step")
                    .clicked()
                {
                    remove = Some(step);
                }
            });
            ui.add_space(4.0);

            if let Some(message) = first_bad_keycode(text) {
                error = Some(format!("step {}: {message}", step + 1));
            }
        }

        if let Some(step) = remove {
            editor.steps.remove(step);
            self.target = CaptureTarget::Step(step.min(editor.steps.len().saturating_sub(1)));
        }

        ui.horizontal(|ui| {
            let mut pick = self.layer_pick as i32;
            ui.label("Switch to layer");
            if ui
                .add(egui::DragValue::new(&mut pick).range(0..=15).speed(1))
                .changed()
            {
                self.layer_pick = pick.clamp(0, 15) as usize;
            }
            if ui.button("Add layer switch").clicked() {
                let name = format!("SWITCH_LAYER_{}", pick.clamp(0, 15));
                editor.steps.push(name);
                self.target = CaptureTarget::Step(editor.steps.len() - 1);
                self.layer_pick = pick.clamp(0, 15) as usize;
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Clear steps").clicked() {
                editor.steps.clear();
                self.target = CaptureTarget::Step(0);
            }
        });

        if let Some(error) = error {
            ui.colored_label(ui.visuals().error_fg_color, error);
        }

        let steps = editor.steps.clone();
        self.apply_steps(&steps, layer, index);
    }

    fn apply_steps(&mut self, steps: &[String], layer: usize, index: usize) {
        if let Ok(parsed) = parse_steps(steps) {
            let slot = &mut self.layout.layers[layer].macros[index].steps;
            if parsed != *slot {
                *slot = parsed;
            }
        }
    }

    fn encoder_editor_ui(&mut self, ui: &mut egui::Ui) {
        let Some(editor) = &mut self.encoder_editor else {
            return;
        };
        let layer = editor.layer;

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Rotary encoder").size(18.0).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let button = egui::Button::new(match self.recording {
                    true => "Recording",
                    false => "Record keys",
                })
                .selected(self.recording)
                .min_size(egui::vec2(96.0, 0.0));
                if ui
                    .add(button)
                    .on_hover_text("press keys to fill the direction you click next")
                    .clicked()
                {
                    self.recording = !self.recording;
                    self.captured = None;
                }
            });
        });
        ui.small(format!("layer {layer}"));
        ui.add_space(14.0);

        if self.recording {
            ui.label(
                egui::RichText::new("click the field to fill, then press keys, Esc stops")
                    .color(ui.visuals().error_fg_color),
            );
            ui.add_space(8.0);
        }

        let captured = self.captured.take();
        if let Some(expression) = &captured {
            match self.target {
                CaptureTarget::Counterclockwise => editor.counterclockwise = expression.clone(),
                CaptureTarget::Clockwise => editor.clockwise = expression.clone(),
                CaptureTarget::Step(_) => {}
            }
        }

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Counterclockwise").color(match self.target {
                    CaptureTarget::Counterclockwise => ui.visuals().text_color(),
                    _ => ui.visuals().weak_text_color(),
                }),
            );
            let response = ui
                .add(
                    egui::TextEdit::singleline(&mut editor.counterclockwise)
                        .font(egui::TextStyle::Monospace),
                )
                .on_hover_text(EXPR_HINT);
            if response.clicked() {
                self.target = CaptureTarget::Counterclockwise;
            }
        });
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Clockwise").color(match self.target {
                CaptureTarget::Clockwise => ui.visuals().text_color(),
                _ => ui.visuals().weak_text_color(),
            }));
            let response = ui
                .add(
                    egui::TextEdit::singleline(&mut editor.clockwise)
                        .font(egui::TextStyle::Monospace),
                )
                .on_hover_text(EXPR_HINT);
            if response.clicked() {
                self.target = CaptureTarget::Clockwise;
            }
        });

        let counterclockwise = keycode_converter::keycode(editor.counterclockwise.as_str());
        let clockwise = keycode_converter::keycode(editor.clockwise.as_str());

        match (counterclockwise, clockwise) {
            (Ok(counterclockwise), Ok(clockwise)) => {
                self.layout.layers[layer].counterclockwise = counterclockwise;
                self.layout.layers[layer].clockwise = clockwise;
                editor.error = None;
            }
            (counterclockwise, clockwise) => {
                editor.error = parse_error(counterclockwise.err(), clockwise.err());
            }
        }

        if let Some(error) = &editor.error {
            ui.colored_label(ui.visuals().error_fg_color, error.clone());
        }

        ui.add_space(10.0);
        ui.small(egui::RichText::new(EXPR_HINT).color(ui.visuals().weak_text_color()));
    }

    fn pad_panel(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().panel_fill)
                    .inner_margin(egui::epaint::MarginF32::symmetric(20.0, 16.0)),
            )
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Layer").strong());
                    ui.add_space(6.0);

                    let mut layer = self.layer as i32;
                    let last = self.layout.layers.len() as i32 - 1;
                    if ui
                        .add(egui::DragValue::new(&mut layer).range(0..=last).speed(1))
                        .changed()
                    {
                        self.layer = layer as usize;
                        self.follow_layer();
                    }
                    ui.label(
                        egui::RichText::new(format!("of {}", last + 1))
                            .color(ui.visuals().weak_text_color()),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.edited() {
                            ui.colored_label(ui.visuals().warn_fg_color, "unsaved changes");
                        }
                    });
                });

                ui.add_space(12.0);

                let pad = egui::Grid::new("pad").spacing(KEY_SPACING);
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        pad.show(ui, |ui| {
                            for row in 0..4 {
                                for col in 0..4 {
                                    let index = row * 4 + col;
                                    self.show_pad_cell(ui, index, index == ROTARY_KEY);
                                }
                                ui.end_row();
                            }
                        });
                    },
                );
            });
    }

    fn footer(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("footer")
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().extreme_bg_color)
                    .inner_margin(egui::epaint::MarginF32::symmetric(20.0, 12.0)),
            )
            .show_inside(ui, |ui| {
                if let Some(request) = self.confirm.take() {
                    ui.horizontal(|ui| {
                        ui.colored_label(
                            ui.visuals().warn_fg_color,
                            "this throws away the changes you made",
                        );
                        if ui.button("Continue").clicked() {
                            self.busy = true;
                            self.worker.send(request);
                        }
                        if ui.button("Cancel").clicked() {}
                    });
                    ui.add_space(6.0);
                }

                ui.horizontal(|ui| {
                    if self.connected.is_some() {
                        if ui.button("Disconnect").clicked() {
                            self.send(Request::Disconnect);
                        }
                    } else if ui.button("Connect").clicked() {
                        self.send(Request::Connect);
                    }

                    let idle = self.connected.is_some() && !self.busy;
                    if ui
                        .add_enabled(idle && self.layout_known, egui::Button::new("Read"))
                        .clicked()
                    {
                        self.send(Request::Refresh);
                    }
                    if ui
                        .add_enabled(self.can_write(), egui::Button::new("Write"))
                        .on_hover_text("send the layout to the board without storing it, lost on a reboot")
                        .clicked()
                    {
                        self.send(Request::Write(Box::new(self.layout.clone())));
                    }
                    if ui
                        .add_enabled(self.can_write(), egui::Button::new("Save"))
                        .on_hover_text("send the layout to the board and store it in flash so it survives a reboot")
                        .clicked()
                    {
                        self.send(Request::Save(Box::new(self.layout.clone())));
                    }
                    if ui
                        .add_enabled(idle, egui::Button::new("Reset"))
                        .on_hover_text("load the factory defaults on the board")
                        .clicked()
                    {
                        self.send(Request::Reset);
                    }

                    if self.busy {
                        ui.spinner();
                    }

                    if self.edited() {
                        ui.colored_label(
                            ui.visuals().warn_fg_color,
                            "you have changes that are not on the board",
                        );
                    }
                });

                let (text, color) = match &self.error {
                    Some(error) => (error.clone(), ui.visuals().error_fg_color),
                    None => (self.status.clone(), ui.visuals().weak_text_color()),
                };
                ui.label(egui::RichText::new(text).color(color).small());
            });
    }

    fn poll_capture(&mut self, ctx: &egui::Context) {
        if !self.recording {
            return;
        }

        let pressed: Vec<(egui::Key, egui::Modifiers)> = ctx.input(|input| {
            input
                .events
                .iter()
                .filter_map(|event| match event {
                    egui::Event::Key {
                        key,
                        pressed: true,
                        repeat: false,
                        modifiers,
                        ..
                    } => Some((*key, *modifiers)),
                    _ => None,
                })
                .collect()
        });

        let Some((key, modifiers)) = pressed.into_iter().next() else {
            return;
        };

        if key == egui::Key::Escape {
            self.recording = false;
            return;
        }

        if keycodes::is_modifier_key(key) {
            return;
        }

        let Some(expression) = keycodes::captured_expression(key, modifiers) else {
            return;
        };

        self.captured = Some(expression);
        ctx.memory_mut(|memory| memory.surrender_focus(egui::Id::NULL));
    }

    fn help_window(&mut self, ctx: &egui::Context) {
        let mut open = self.help_open;

        egui::Window::new("Keycodes")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([540.0, 600.0])
            .default_pos([440.0, 90.0])
            .show(ctx, |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.help_filter)
                        .hint_text("filter, e.g. volume or KC_F13")
                        .desired_width(f32::INFINITY),
                );
                ui.add_space(6.0);

                let needle = self.help_filter.trim().to_lowercase();
                let mut groups: Vec<(&'static str, Vec<&'static str>, u16)> = Vec::new();

                for (name, value) in keycode_converter::catalog() {
                    if keycodes::ALIASES.contains(&name.as_str()) {
                        continue;
                    }

                    if !needle.is_empty()
                        && !name.to_lowercase().contains(&needle)
                        && !format!("0x{value:04x}").contains(&needle)
                    {
                        continue;
                    }

                    let group = keycodes::group_of(name);
                    match groups
                        .iter_mut()
                        .find(|(existing, _, _)| *existing == group)
                    {
                        Some((_, names, _)) => names.push(name),
                        None => groups.push((group, vec![name], *value)),
                    }
                }

                if groups.is_empty() {
                    ui.label(
                        egui::RichText::new("nothing matches")
                            .color(ui.visuals().weak_text_color()),
                    );
                    return;
                }

                groups.sort_by_key(|(group, names, _)| (keycodes::group_order(group), names.len()));

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (group, names, _) in &groups {
                        ui.label(egui::RichText::new(*group).strong());
                        egui::Grid::new(format!("help-{}", group))
                            .num_columns(2)
                            .spacing([24.0, 2.0])
                            .show(ui, |ui| {
                                for name in names {
                                    let value = keycode_converter::keycode(name).unwrap_or(0);
                                    let response = ui
                                        .label(egui::RichText::new(*name).monospace())
                                        .on_hover_text("click to copy");
                                    if response.clicked() {
                                        ui.ctx().copy_text((*name).to_string());
                                    }
                                    ui.label(
                                        egui::RichText::new(format!("0x{value:04X}"))
                                            .monospace()
                                            .color(ui.visuals().weak_text_color()),
                                    );
                                    ui.end_row();
                                }
                            });
                        ui.add_space(6.0);
                    }
                });
            });

        self.help_open = open;
    }

    fn send(&mut self, request: Request) {
        if self.edited() && matches!(request, Request::Refresh | Request::Reset) {
            self.confirm = Some(request);
            return;
        }

        self.error = None;
        self.busy = true;
        self.worker.send(request);
    }

    fn can_write(&self) -> bool {
        self.connected.is_some() && self.layout_known && !self.busy
    }

    fn handle(&mut self, event: Event) {
        match event {
            Event::Status(status) => {
                self.error = None;
                self.status = status;
            }
            Event::Error(error) => {
                self.status = "The last action failed".to_string();
                self.error = Some(error);
            }
            Event::Connected(description) => {
                self.status = format!("Connected to {description}");
                self.connected = Some(description);
            }
            Event::Disconnected => {
                self.connected = None;
                self.device_status = None;
                self.layout_known = false;
                self.key_editor = None;
                self.encoder_editor = None;
                self.selected = None;
                self.status = "Not connected".to_string();
            }
            Event::StatusReport(status) => self.device_status = Some(*status),
            Event::Written(layout) => self.device_layout = *layout,
            Event::Layout(layout) => {
                self.device_layout = *layout;
                self.layout = self.device_layout.clone();
                self.layout_known = true;
            }
        }
    }

    fn edited(&self) -> bool {
        self.connected.is_some() && self.layout != self.device_layout
    }

    fn show_pad_cell(&mut self, ui: &mut egui::Ui, index: usize, with_rotary: bool) {
        let selected = self.selected
            == Some(if with_rotary {
                Selection::Rotary
            } else {
                Selection::Key(index)
            });
        let layer = &self.layout.layers[self.layer];
        let device_layer = &self.device_layout.layers[self.layer];

        let (label, detail, edited) = if with_rotary {
            let changed = layer.counterclockwise != device_layer.counterclockwise
                || layer.clockwise != device_layer.clockwise;
            ("ENC".to_string(), keycode_to_expr(layer.clockwise), changed)
        } else {
            let slot = &layer.macros[index];
            let steps = slot.steps.len();
            let detail = summarize(&slot.step_text(0));
            let detail = match steps > 1 {
                true => format!("{detail} +{}", steps - 1),
                false => detail,
            };
            (
                format!("K{}", index + 1),
                detail,
                slot != &device_layer.macros[index],
            )
        };

        let (rect, response) = ui.allocate_exact_size(KEY_SIZE, egui::Sense::click());
        let visuals = ui.style().interact_selectable(&response, selected);
        let accent = ui.visuals().selection.stroke.color;

        let fill = match selected {
            true => ui.visuals().selection.bg_fill,
            false if response.hovered() => visuals.weak_bg_fill.gamma_multiply(1.35),
            false => ui.visuals().extreme_bg_color,
        };
        let stroke = egui::Stroke::new(
            if selected || edited { 1.5 } else { 1.0 },
            match (selected, edited) {
                (true, _) => accent,
                (false, true) => accent.gamma_multiply(0.5),
                (false, false) => ui.visuals().widgets.noninteractive.bg_stroke.color,
            },
        );

        ui.painter().rect(
            rect,
            egui::CornerRadius::same(PAD_RADIUS),
            fill,
            stroke,
            egui::StrokeKind::Inside,
        );

        let muted = ui.visuals().weak_text_color();
        let small = egui::TextStyle::Small.resolve(ui.style());
        let body = egui::FontId::proportional(13.0);
        let inner = rect.width() - 16.0;

        ui.painter().text(
            rect.left_top() + egui::vec2(10.0, 7.0),
            egui::Align2::LEFT_TOP,
            label,
            small.clone(),
            muted,
        );

        if with_rotary {
            let detail = match detail.as_str() {
                "KC_NO" | "" => "not set".to_string(),
                _ => detail,
            };
            let center = egui::pos2(rect.center().x, rect.center().y - 6.0);
            let radius = 15.0;
            ui.painter().circle_stroke(
                center,
                radius,
                egui::Stroke::new(2.0, accent.gamma_multiply(0.8)),
            );
            ui.painter()
                .circle_filled(egui::pos2(center.x, center.y - radius), 2.5, accent);
            ui.painter().text(
                egui::pos2(center.x, rect.bottom() - 8.0),
                egui::Align2::CENTER_BOTTOM,
                fit_text(ui, &detail, &small, inner),
                small.clone(),
                ui.visuals().text_color(),
            );
        } else if !detail.is_empty() {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                fit_text(ui, &detail, &body, inner),
                body.clone(),
                ui.visuals().text_color(),
            );
        }

        if response.clicked() {
            let position = response.interact_pointer_pos().unwrap_or(rect.center());
            let on_rotary = with_rotary && position.distance(rect.center()) <= 18.0;

            if on_rotary {
                self.select_rotary();
            } else {
                self.select_key(index);
            }
        }
    }

    /// Keeps the open editor on the same key while the layer changes, so the
    /// selection follows instead of pointing at a key that is not on screen.
    fn follow_layer(&mut self) {
        match self.selected {
            Some(Selection::Key(index)) => {
                let slot = &self.layout.layers[self.layer].macros[index];
                self.key_editor = Some(KeyEditor {
                    layer: self.layer,
                    index,
                    steps: (0..slot.steps.len().max(1))
                        .map(|step| slot.step_text(step))
                        .collect(),
                });
                self.target = CaptureTarget::Step(0);
            }
            Some(Selection::Rotary) => {
                let layer = &self.layout.layers[self.layer];
                self.encoder_editor = Some(EncoderEditor {
                    layer: self.layer,
                    counterclockwise: keycode_to_expr(layer.counterclockwise),
                    clockwise: keycode_to_expr(layer.clockwise),
                    error: None,
                });
                self.target = CaptureTarget::Counterclockwise;
            }
            None => {}
        }
    }

    fn select_key(&mut self, index: usize) {
        self.encoder_editor = None;
        self.captured = None;

        if self.selected == Some(Selection::Key(index)) {
            self.selected = None;
            self.key_editor = None;
            return;
        }

        self.selected = Some(Selection::Key(index));
        let slot = &self.layout.layers[self.layer].macros[index];
        self.key_editor = Some(KeyEditor {
            layer: self.layer,
            index,
            steps: (0..slot.steps.len().max(1))
                .map(|step| slot.step_text(step))
                .collect(),
        });
    }

    fn select_rotary(&mut self) {
        self.key_editor = None;
        self.captured = None;

        if self.selected == Some(Selection::Rotary) {
            self.selected = None;
            self.encoder_editor = None;
            return;
        }

        self.selected = Some(Selection::Rotary);
        let layer = &self.layout.layers[self.layer];
        self.encoder_editor = Some(EncoderEditor {
            layer: self.layer,
            counterclockwise: keycode_to_expr(layer.counterclockwise),
            clockwise: keycode_to_expr(layer.clockwise),
            error: None,
        });
    }
}

fn text_width(ui: &egui::Ui, text: &str, font: &egui::FontId) -> f32 {
    let job = egui::text::LayoutJob::simple(
        text.to_string(),
        font.clone(),
        egui::Color32::PLACEHOLDER,
        f32::INFINITY,
    );
    ui.painter().layout_job(job).size().x
}

fn fit_text(ui: &egui::Ui, text: &str, font: &egui::FontId, width: f32) -> String {
    if text_width(ui, text, font) <= width {
        return text.to_string();
    }

    let mut fitted = String::new();
    for character in text.chars() {
        if text_width(ui, &format!("{fitted}{character}.."), font) > width {
            break;
        }
        fitted.push(character);
    }
    format!("{fitted}..")
}

fn summarize(text: &str) -> String {
    text.lines().next().unwrap_or_default().trim().to_string()
}

fn first_bad_keycode(text: &str) -> Option<String> {
    for part in text.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Err(message) = keycode_converter::keycode(part) {
            return Some(message.to_string());
        }
    }
    None
}

fn parse_steps(steps: &[String]) -> Result<Vec<Step>, String> {
    let mut parsed = Vec::new();

    for (index, text) in steps.iter().enumerate() {
        let text = text.trim();
        if text.is_empty() {
            continue;
        }

        let mut keycodes = Vec::new();
        for part in text.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            match keycode_converter::keycode(part) {
                Ok(keycode) => keycodes.push(keycode),
                Err(error) => return Err(format!("step {}: {error}", index + 1)),
            }
        }

        if keycodes.is_empty() {
            continue;
        }

        if keycodes.len() > MAX_STEP_KEYS {
            return Err(format!(
                "step {} presses {keycodes_len} keycodes, at most {MAX_STEP_KEYS} fit in a step",
                index + 1,
                keycodes_len = keycodes.len(),
            ));
        }

        parsed.push(Step::new(keycodes));
    }

    Ok(parsed)
}

fn parse_error(
    counterclockwise: Option<Box<dyn Error>>,
    clockwise: Option<Box<dyn Error>>,
) -> Option<String> {
    match (counterclockwise, clockwise) {
        (Some(error), _) => Some(format!("counterclockwise: {error}")),
        (None, Some(error)) => Some(format!("clockwise: {error}")),
        (None, None) => None,
    }
}
