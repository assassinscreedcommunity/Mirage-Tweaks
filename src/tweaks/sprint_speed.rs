use anyhow::Result;

use crate::game::{Game, Patch};
use crate::tweaks::{Clamp, NumericTweak, Tweak};

const DEFAULT_VALUE: f32 = 6.8;

pub struct SprintSpeed {
    patch: Patch<f32>,
}

impl Tweak<f32> for SprintSpeed {
    fn setup(process: &Game) -> Result<Self> {
        let address = process
            .address(0x629A188)
            .read_address()?
            .offset(0x10)
            .read_address()?
            .read_address()?
            .offset(0x120)
            .read_address()?
            .offset(0x570);

        Ok(Self {
            patch: Patch::apply(address, &DEFAULT_VALUE)?,
        })
    }

    fn name(&self) -> &str {
        "sprint speed"
    }

    fn default_value(&self) -> f32 {
        DEFAULT_VALUE
    }

    fn set_value(&self, value: &f32) -> Result<()> {
        self.patch.update(value)
    }
}

impl NumericTweak<f32> for SprintSpeed {
    fn min_value(&self) -> f32 {
        0.0
    }

    fn max_value(&self) -> f32 {
        12.0
    }

    fn clamp(&self) -> Clamp {
        Clamp::Low
    }
}
