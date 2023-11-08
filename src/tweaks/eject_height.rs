use super::{Tweak, TweakIntent};
use crate::config::{TweakConfig, CONFIG};
use crate::game::{Game, Patch};
use crate::process::Section;
use anyhow::{bail, Result};
use log::{error, info};
use std::ops::DerefMut;

const CONFIG_KEY: &str = "eject-height";
const INSTRUCTION_SIZE: usize = 8;

pub struct EjectHeightTweak {
    game: Game,
    instruction_address: usize,
    cave_offset: i32,
    state: State,
    value: f64,
}

enum State {
    Disabled,
    Enabled {
        _instruction_patch: Patch<[u8; INSTRUCTION_SIZE]>,
        value_patch: Patch<f32>,
    },
}

impl EjectHeightTweak {
    pub fn new(game: &Game) -> Result<Self> {
        let (region, instruction_offset) = game.process.find_pattern(
            Section::Code,
            r"\xF3\x0F\x10\x25[\x00-\xFF]{4}\xF3\x0F\x10\x6C\x24\x58",
        )?;
        let instruction_address = region.address + instruction_offset;
        info!("Found Eject Height instruction at {instruction_address:#X}");

        let mut cave_offset = ((4 - (instruction_address % 4)) % 4) as i32;
        loop {
            let offset = instruction_offset + INSTRUCTION_SIZE + cave_offset as usize;
            if region.data[offset..(offset + 4)] == [0xCC, 0xCC, 0xCC, 0xCC] {
                let cave_address = region.address + offset;
                info!("Found code cave for Eject Height at {cave_address:#X}");
                break;
            }

            if cave_offset as usize + 8 > region.data.len() {
                bail!("Couldn't find code cave for Eject Height");
            }
            cave_offset += 4;
        }

        Ok(Self {
            game: game.clone(),
            instruction_address,
            cave_offset,
            state: State::Disabled,
            value: Self::DEFAULT,
        })
    }

    pub fn load_config(&mut self) {
        info!("Loading Eject Height tweak config");
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
                    error!("Failed to enable Eject Height tweak: {error}");
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
        info!("Enabling Eject Height tweak");
        let full_cave_offset = INSTRUCTION_SIZE + self.cave_offset as usize;
        let size = full_cave_offset + 4;
        let memory = self.game.process.read(self.instruction_address, size)?;

        let address = self.instruction_address + full_cave_offset;
        let value = self.value as f32;
        let original =
            f32::from_ne_bytes(memory[full_cave_offset..(full_cave_offset + 4)].try_into()?);
        let value_patch = self.game.patch(address, &value, original)?;

        let address = self.instruction_address;
        let cave = self.cave_offset.to_ne_bytes();
        let instruction = [0xF3, 0x0F, 0x10, 0x25, cave[0], cave[1], cave[2], cave[3]]; // movss xmm4 [cave]
        let mut original = [0; INSTRUCTION_SIZE];
        original.copy_from_slice(&memory[..INSTRUCTION_SIZE]);
        let instruction_patch = self.game.patch(address, &instruction, original)?;

        self.state = State::Enabled {
            _instruction_patch: instruction_patch,
            value_patch,
        };

        Ok(())
    }
}

impl Tweak<f64> for EjectHeightTweak {
    const NAME: &'static str = "Eject Height";
    const DEFAULT: f64 = 1.3;
    const MIN: f64 = 0.0;
    const MAX: f64 = 6.0;
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
                error!("Failed to enable Eject Height tweak: {error}");
                return;
            }
            self.save_config();
        }
    }

    fn disable(&mut self) {
        if let State::Enabled { .. } = self.state {
            info!("Disabling Eject Height tweak");
            self.state = State::Disabled;
            self.save_config();
        }
    }

    fn set_value(&mut self, value: f64) {
        info!("Setting Eject Height to {value}");
        if let State::Enabled { value_patch, .. } = &self.state {
            if let Err(error) = value_patch.update(&(value as f32)) {
                error!("Failed to set Eject Height: {error}");
                return;
            }
        }
        self.value = value;
        self.save_config();
    }

    fn reset_value(&mut self) {
        info!("Resetting Eject Height to {}", Self::DEFAULT);
        if let State::Enabled { value_patch, .. } = &self.state {
            if let Err(error) = value_patch.update(&(Self::DEFAULT as f32)) {
                error!("Failed to reset Eject Height: {error}");
                return;
            }
        }
        self.value = Self::DEFAULT;
        self.save_config();
    }
}
