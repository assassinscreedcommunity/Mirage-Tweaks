use std::ffi::c_void;
use std::mem::size_of;

use anyhow::{anyhow, Result};
use windows::Win32::Foundation::{CloseHandle, HANDLE, HMODULE};
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::ProcessStatus::{
    EnumProcessModules, EnumProcesses, GetModuleBaseNameA,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_ACCESS_RIGHTS, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION,
    PROCESS_VM_READ, PROCESS_VM_WRITE,
};

pub enum Version {
    Standard,
    UbisoftPlus,
}

pub struct Process {
    handle: HANDLE,
    pub base_address: usize,
    pub version: Version,
}

impl Process {
    pub fn attach() -> Result<Self> {
        let mut processes = [0u32; 1024];
        let mut returned = 0;
        unsafe { EnumProcesses(processes.as_mut_ptr(), 1024 * 4, &mut returned) }?;

        for i in 0..(returned / 4) {
            let pid = processes[i as usize];
            let rights = PROCESS_ACCESS_RIGHTS::default()
                | PROCESS_QUERY_INFORMATION
                | PROCESS_VM_OPERATION
                | PROCESS_VM_READ
                | PROCESS_VM_WRITE;
            let handle = unsafe { OpenProcess(rights, false, pid) };
            let Ok(handle) = handle else { continue; };

            let mut modules = [HMODULE(0); 1024];
            let mut returned = 0;
            unsafe { EnumProcessModules(handle, modules.as_mut_ptr(), 1024 * 4, &mut returned) }?;

            for i in 0..(returned / 4) {
                let module = modules[i as usize];

                let mut name = [0u8; 32];
                unsafe { GetModuleBaseNameA(handle, module, &mut name) };

                if name.starts_with(b"ACMirage.exe\0") {
                    let HMODULE(base_address) = module;
                    return Ok(Self {
                        handle,
                        base_address: base_address as usize,
                        version: Version::Standard,
                    });
                }

                if name.starts_with(b"ACMirage_plus.exe\0") {
                    let HMODULE(base_address) = module;
                    return Ok(Self {
                        handle,
                        base_address: base_address as usize,
                        version: Version::UbisoftPlus,
                    });
                }
            }

            let _ = unsafe { CloseHandle(handle) };
        }

        Err(anyhow!(
            "Couldn't find Assassin's Creed Mirage, is the game running?"
        ))
    }

    pub fn read<T>(&self, address: usize) -> Result<T> {
        unsafe {
            let mut value = std::mem::zeroed();
            ReadProcessMemory(
                self.handle,
                address as *const c_void,
                &mut value as *mut T as *mut c_void,
                size_of::<T>(),
                None,
            )?;
            Ok(value)
        }
    }

    pub fn write<T>(&self, address: usize, value: &T) -> Result<()> {
        unsafe {
            WriteProcessMemory(
                self.handle,
                address as *mut c_void,
                value as *const T as *const c_void,
                size_of::<T>(),
                None,
            )?;
        }
        Ok(())
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.handle) };
    }
}
