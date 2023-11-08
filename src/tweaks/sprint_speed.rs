use super::{Tweak, TweakIntent};
use crate::config::{TweakConfig, CONFIG};
use crate::game::{Game, Patch};
use crate::process::Section;
use anyhow::Result;
use log::{error, info};
use std::ops::DerefMut;

const CONFIG_KEY: &str = "sprint-speed";

pub struct SprintSpeedTweak {
    game: Game,
    address: usize,
    state: State,
    value: f64,
}

enum State {
    Disabled,
    Enabled { patch: Patch<f32> },
}

impl SprintSpeedTweak {
    pub fn new(game: &Game) -> Result<Self> {
        let (region, offset) = game.process.find_pattern(
            Section::Heap,
            r"\x00\x00\x00\x00\x33\xFF\x33\x3E\x9A\x99\xD9\x40\x00\x00\x00\x00",
        )?;
        let address = region.address + offset + 8;

        Ok(Self {
            game: game.clone(),
            address,
            state: State::Disabled,
            value: Self::DEFAULT,
        })
    }

    pub fn load_config(&mut self) {
        info!("Loading Sprint Speed tweak config");
        let config = CONFIG.lock().unwrap();
        let config = config
            .tweaks
            .as_ref()
            .and_then(|tweaks| tweaks.get(CONFIG_KEY))
            .cloned();

        if let Some(config) = config {
            if let Some(value) = config.value.as_float() {
                self.value = value;
            } else {
                info!("Couldn't parse tweaks.{CONFIG_KEY}.value as float");
            }

            if config.enabled {
                if let Err(error) = self.enable() {
                    error!("Failed to enable Sprint Speed tweak: {error}");
                }
            }
        }
    }

    fn save_config(&self) {
        let mut config = CONFIG.lock().unwrap();
        config
            .deref_mut()
            .tweaks
            .get_or_insert_with(Default::default)
            .insert(
                CONFIG_KEY.to_owned(),
                TweakConfig {
                    enabled: self.enabled(),
                    value: self.value.into(),
                },
            );
        config.save();
    }

    fn enable(&mut self) -> Result<()> {
        info!("Enabling Sprint Speed tweak");
        let value = self.value as f32;
        let original = self.game.process.read_into(self.address)?;
        let patch = self.game.patch(self.address, &value, original)?;
        self.state = State::Enabled { patch };
        Ok(())
    }
}

impl Tweak<f64> for SprintSpeedTweak {
    const NAME: &'static str = "Sprint Speed";
    const DEFAULT: f64 = 6.8;
    const MIN: f64 = 0.0;
    const MAX: f64 = 12.0;
    const INTENT: TweakIntent = TweakIntent::Increase;

    fn enabled(&self) -> bool {
        match self.state {
            State::Disabled => false,
            State::Enabled { .. } => true,
        }
    }

    fn value(&self) -> f64 {
        self.value
    }

    fn enable(&mut self) {
        if let State::Disabled = self.state {
            if let Err(error) = self.enable() {
                error!("Failed to enable Sprint Speed tweak: {error}");
                return;
            }
            self.save_config();
        }
    }

    fn disable(&mut self) {
        if let State::Enabled { .. } = self.state {
            info!("Disabling Sprint Speed tweak");
            self.state = State::Disabled;
            self.save_config();
        }
    }

    fn set_value(&mut self, value: f64) {
        info!("Setting Sprint Speed to {value}");
        if let State::Enabled { patch, .. } = &self.state {
            if let Err(error) = patch.update(&(value as f32)) {
                error!("Failed to set Sprint Speed: {error}");
                return;
            }
        }
        self.value = value;
        self.save_config();
    }

    fn reset_value(&mut self) {
        info!("Resetting Sprint Speed to {}", Self::DEFAULT);
        if let State::Enabled { patch, .. } = &self.state {
            if let Err(error) = patch.update(&(Self::DEFAULT as f32)) {
                error!("Failed to reset Sprint Speed: {error}");
                return;
            }
        }
        self.value = Self::DEFAULT;
        self.save_config();
    }
}
