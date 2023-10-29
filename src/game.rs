use anyhow::Result;
use std::rc::Rc;

use crate::process::Process;

#[derive(Clone)]
pub struct Game {
    process: Rc<Process>,
}

impl Game {
    pub fn attach() -> Result<Self> {
        Ok(Self {
            process: Rc::new(Process::attach()?),
        })
    }

    pub fn address(&self, address: usize) -> Address {
        Address {
            process: self.process.clone(),
            address: self.process.base_address + address,
        }
    }
}

pub struct Address {
    process: Rc<Process>,
    address: usize,
}

impl Address {
    pub fn offset(&self, offset: isize) -> Self {
        Self {
            process: self.process.clone(),
            address: (self.address as isize + offset) as usize,
        }
    }

    pub fn read_address(&self) -> Result<Address> {
        Ok(Self {
            process: self.process.clone(),
            address: self.read()?,
        })
    }

    pub fn read<T>(&self) -> Result<T> {
        self.process.read(self.address)
    }

    pub fn write<T>(&self, data: &T) -> Result<()> {
        self.process.write(self.address, data)
    }
}

pub struct Patch<T> {
    address: Address,
    original: T,
}

impl<T> Patch<T> {
    pub fn apply(address: Address, data: &T) -> Result<Self> {
        let original = address.read()?;
        address.write(data)?;
        Ok(Self { address, original })
    }

    pub fn update(&self, data: &T) -> Result<()> {
        self.address.write(data)?;
        Ok(())
    }
}

impl<T> Drop for Patch<T> {
    fn drop(&mut self) {
        let _ = self.address.write(&self.original);
    }
}
