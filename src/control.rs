use crate::tweaks::{Clamp, NumericTweak};
use eframe::egui;
use eframe::egui::{Ui, Widget};
use eframe::emath::Numeric;

pub trait Control {
    fn draw(&mut self, ui: &mut Ui);
}

pub struct SliderControl<T: NumericTweak<N>, N: Numeric> {
    tweak: T,
    value: N,
    game_value: N,
}

impl<T: NumericTweak<N>, N: Numeric> SliderControl<T, N> {
    pub fn new(tweak: T) -> Self {
        let value = tweak.default_value();
        Self {
            tweak,
            value,
            game_value: value,
        }
    }

    fn write(&mut self) {
        if let Err(error) = self.tweak.set_value(&self.value) {
            self.value = self.game_value;
            crate::message::show(error.to_string());
            eprintln!("{error}");
        } else {
            self.game_value = self.value;
        }
    }
}

impl<T: NumericTweak<N>, N: Numeric> Control for SliderControl<T, N> {
    fn draw(&mut self, ui: &mut Ui) {
        let min = self.tweak.min_value();
        let max = self.tweak.max_value();
        ui.label(self.tweak.name());
        let slider = egui::Slider::new(&mut self.value, min..=max)
            .clamp_to_range(false)
            .ui(ui);
        let reset = ui.button("reset");

        if self.value != self.game_value {
            if slider.drag_released() {
                match self.tweak.clamp() {
                    Clamp::None => {}
                    Clamp::Low => {
                        if self.value < self.tweak.default_value() {
                            self.value = self.tweak.default_value();
                        }
                    }
                    Clamp::High => {
                        if self.value > self.tweak.default_value() {
                            self.value = self.tweak.default_value();
                        }
                    }
                }
                self.write();
            }

            if slider.lost_focus() {
                self.write();
            }
        }

        if reset.clicked() {
            self.value = self.tweak.default_value();
            self.write();
        }
    }
}
