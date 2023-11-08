use crate::process::Process;
use anyhow::Result;
use log::{info, warn};
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Clone)]
pub struct Game {
    pub process: Arc<Process>,
}

impl Game {
    pub fn attach() -> Result<Self> {
        let process = Arc::new(Process::attach()?);
        Ok(Self { process })
    }

    pub fn patch<T: Debug>(&self, address: usize, value: &T, original: T) -> Result<Patch<T>> {
        info!("Patching {address:#X} with {value:X?}");
        self.process.write(address, value)?;
        Ok(Patch {
            process: self.process.clone(),
            address,
            original,
        })
    }
}

pub struct Patch<T> {
    process: Arc<Process>,
    address: usize,
    original: T,
}

impl<T: Debug> Patch<T> {
    pub fn update(&self, value: &T) -> Result<()> {
        info!("Updating patch at {:#X} to {value:X?}", self.address);
        self.process.write(self.address, value)
    }
}

impl<T> Drop for Patch<T> {
    fn drop(&mut self) {
        info!("Restoring patch at {:#X}", self.address);
        if let Err(error) = self.process.write(self.address, &self.original) {
            warn!("Couldn't restore patch ({error})");
        }
    }
}
