use crate::game::{Address, Game, Patch};
use crate::tweaks::{Clamp, NumericTweak};
use anyhow::Result;

use super::Tweak;

const DEFAULT_VALUE: f32 = 1.3;
const VALUE_ADDRESS: i32 = 0x74E74C;
const INSTRUCTION_ADDRESS: i32 = 0x74E818;
const INSTRUCTION_LENGTH: usize = 8;

pub struct EjectHeight {
    _patch: Patch<[u8; INSTRUCTION_LENGTH]>,
    value_address: Address,
}

impl Tweak<f32> for EjectHeight {
    fn setup(game: &Game) -> Result<Self> {
        let value_address = game.address(VALUE_ADDRESS as usize);
        value_address.write(&DEFAULT_VALUE)?;

        let mut code = [0; INSTRUCTION_LENGTH];
        code[0..4].copy_from_slice(&[0xF3, 0x0F, 0x10, 0b00_100_101]); // movss xmm4, [x]
        let displacement: i32 = VALUE_ADDRESS - INSTRUCTION_ADDRESS - INSTRUCTION_LENGTH as i32;
        code[4..8].copy_from_slice(&displacement.to_ne_bytes());
        let patch = Patch::apply(game.address(INSTRUCTION_ADDRESS as usize), &code)?;

        Ok(Self {
            _patch: patch,
            value_address,
        })
    }

    fn name(&self) -> &str {
        "eject height"
    }

    fn default_value(&self) -> f32 {
        DEFAULT_VALUE
    }

    fn set_value(&self, value: &f32) -> Result<()> {
        self.value_address.write(value)
    }
}

impl NumericTweak<f32> for EjectHeight {
    fn min_value(&self) -> f32 {
        0.0
    }

    fn max_value(&self) -> f32 {
        6.0
    }

    fn clamp(&self) -> Clamp {
        Clamp::Low
    }
}
