use anyhow::{anyhow, Result};
use eframe::{egui, CreationContext, IconData};

use crate::control::Control;

pub struct Menu {
    controls: Vec<Box<dyn Control>>,
}

impl Menu {
    pub fn new() -> Self {
        Menu {
            controls: Vec::new(),
        }
    }

    pub fn add_control(&mut self, control: impl Control + 'static) {
        self.controls.push(Box::new(control));
    }

    pub fn show(self) -> Result<()> {
        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(320.0, 180.0)),
            resizable: false,
            icon_data: Some(IconData::try_from_png_bytes(include_bytes!("../icon.png"))?),
            ..Default::default()
        };

        eframe::run_native(
            "Mirage Tweaks",
            options,
            Box::new(|context| self.setup(context)),
        )
        .or(Err(anyhow!("couldn't create window")))
    }

    fn setup(self, context: &CreationContext) -> Box<Self> {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            String::from("OpenSans-Regular"),
            egui::FontData::from_static(include_bytes!("../fonts/OpenSans-Regular.ttf")),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push(String::from("OpenSans-Regular"));
        context.egui_ctx.set_fonts(fonts);
        Box::new(self)
    }
}

impl eframe::App for Menu {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(context, |ui| {
            ui.heading("Mirage Tweaks");
            egui::Grid::new("tweaks").show(ui, |ui| {
                for control in &mut self.controls {
                    control.draw(ui);
                    ui.end_row();
                }
            });
        });
    }
}
