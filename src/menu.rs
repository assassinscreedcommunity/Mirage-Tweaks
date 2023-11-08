use crate::tweaks::{Tweak, TweakIntent};
use anyhow::{anyhow, Result};
use eframe::egui::{Button, Checkbox, Label, Slider};
use eframe::emath::Numeric;
use eframe::{egui, IconData};
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

pub struct State {
    pub controls: Vec<Box<dyn Control>>,
    pub status: Status,
}

pub enum Status {
    Loading,
    Done,
}

pub struct Menu {
    state: Arc<Mutex<State>>,
}

impl Menu {
    pub fn show(state: Arc<Mutex<State>>) -> Result<()> {
        let menu = Self { state };

        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(322.0, 132.0)),
            resizable: false,
            icon_data: Some(IconData::try_from_png_bytes(include_bytes!("../icon.png"))?),
            ..Default::default()
        };

        let result = eframe::run_native(
            "Mirage Tweaks",
            options,
            Box::new(|context| menu.setup(context)),
        );
        result.map_err(|error| anyhow!("{error}"))
    }

    fn setup(self, context: &eframe::CreationContext) -> Box<Self> {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "OpenSans-Regular".into(),
            egui::FontData::from_static(include_bytes!("../fonts/OpenSans-Regular.ttf")),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("OpenSans-Regular".into());
        context.egui_ctx.set_fonts(fonts);
        Box::new(self)
    }
}

impl eframe::App for Menu {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(context, |ui| {
            let mut state = self.state.lock().unwrap();
            ui.heading("Mirage Tweaks");
            match state.status {
                Status::Loading => {
                    ui.label("Loading Tweaks...");
                }
                Status::Done => {
                    egui::Grid::new("controls").show(ui, |ui| {
                        for control in &mut state.deref_mut().controls {
                            control.show(ui);
                            ui.end_row();
                        }
                    });
                }
            }
        });
    }
}

pub trait Control: Send {
    fn show(&mut self, ui: &mut egui::Ui);
}

pub struct SliderControl<T: Tweak<N>, N> {
    tweak: Result<T>,
    enabled: bool,
    value: N,
}

impl<T: Tweak<N>, N: Numeric> SliderControl<T, N> {
    pub fn new(tweak: Result<T>) -> Self {
        let enabled = tweak.as_ref().map_or(false, |tweak| tweak.enabled());
        let value = tweak.as_ref().map_or(T::DEFAULT, |tweak| tweak.value());
        Self {
            tweak,
            enabled,
            value,
        }
    }
}

impl<T: Tweak<N>, N: Numeric + Send> Control for SliderControl<T, N> {
    fn show(&mut self, ui: &mut egui::Ui) {
        match &mut self.tweak {
            Ok(tweak) => {
                ui.add(Label::new(T::NAME));
                ui.horizontal(|ui| {
                    let checkbox = ui.add(Checkbox::without_text(&mut self.enabled));
                    let slider = ui.add_enabled(
                        self.enabled,
                        Slider::new(&mut self.value, T::MIN..=T::MAX).clamp_to_range(false),
                    );
                    let reset = ui.add_enabled(self.enabled, Button::new("Reset"));

                    if checkbox.changed() {
                        if self.enabled {
                            tweak.enable();
                        } else {
                            tweak.disable();
                        }
                        self.enabled = tweak.enabled();
                    }

                    if slider.drag_released() {
                        match T::INTENT {
                            TweakIntent::Increase => {
                                if self.value < T::DEFAULT {
                                    self.value = T::DEFAULT;
                                }
                            }
                        }
                    }

                    if (slider.drag_released() || slider.lost_focus())
                        && self.value != tweak.value()
                    {
                        tweak.set_value(self.value);
                        self.value = tweak.value();
                    }

                    if reset.clicked() {
                        tweak.reset_value();
                        self.value = tweak.value();
                    }
                });
            }
            Err(error) => {
                ui.add_enabled(false, Label::new(T::NAME))
                    .on_disabled_hover_text(error.to_string());
                ui.horizontal(|ui| {
                    ui.add_enabled(false, Checkbox::without_text(&mut self.enabled))
                        .on_disabled_hover_text(error.to_string());
                    ui.add_enabled(false, Slider::new(&mut self.value, T::MIN..=T::MAX))
                        .on_disabled_hover_text(error.to_string());
                    ui.add_enabled(false, Button::new("Reset"))
                        .on_disabled_hover_text(error.to_string());
                });
            }
        };
    }
}
