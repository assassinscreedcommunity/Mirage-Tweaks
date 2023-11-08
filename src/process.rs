use crate::config::CONFIG;
use anyhow::{bail, Result};
use log::{info, warn};
use regex::bytes::RegexBuilder;
use std::ffi::CStr;
use std::mem::size_of;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Module32First, Process32First, Process32Next,
    CREATE_TOOLHELP_SNAPSHOT_FLAGS, MODULEENTRY32, PROCESSENTRY32, TH32CS_SNAPMODULE,
    TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Memory::{
    VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_GUARD, PAGE_NOACCESS,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_ACCESS_RIGHTS, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION,
    PROCESS_VM_READ, PROCESS_VM_WRITE,
};

pub struct Process {
    handle: HANDLE,
    base_address: usize,
}

impl Process {
    pub fn attach() -> Result<Self> {
        let module_names = CONFIG
            .lock()
            .unwrap()
            .module_names
            .clone()
            .unwrap_or(vec!["ACMirage.exe".into(), "ACMirage_plus.exe".into()]);

        unsafe {
            let snapshot = Snapshot::new(TH32CS_SNAPPROCESS, 0)?;

            let mut process = PROCESSENTRY32 {
                dwSize: size_of::<PROCESSENTRY32>() as u32,
                ..Default::default()
            };

            Process32First(snapshot.handle, &mut process)?;

            loop {
                if let Ok(name) = CStr::from_ptr(process.szExeFile.as_ptr() as _).to_str() {
                    if module_names.contains(&name.into()) {
                        let pid = process.th32ProcessID;
                        info!("Attaching to process {pid} ({name})");

                        let rights = PROCESS_ACCESS_RIGHTS::default()
                            | PROCESS_QUERY_INFORMATION
                            | PROCESS_VM_OPERATION
                            | PROCESS_VM_READ
                            | PROCESS_VM_WRITE;
                        let handle = OpenProcess(rights, false, pid)?;

                        let snapshot = Snapshot::new(TH32CS_SNAPMODULE, pid)?;
                        let mut module = MODULEENTRY32 {
                            dwSize: size_of::<MODULEENTRY32>() as u32,
                            ..Default::default()
                        };
                        Module32First(snapshot.handle, &mut module)?;
                        let base_address = module.modBaseAddr as usize;

                        return Ok(Self {
                            handle,
                            base_address,
                        });
                    }
                }

                if Process32Next(snapshot.handle, &mut process).is_err() {
                    break;
                }
            }
        }

        bail!("Couldn't find Assassin's Creed Mirage, is the game running?");
    }

    pub fn find_pattern(&self, section: Section, pattern: &str) -> Result<(Region, usize)> {
        let regex = RegexBuilder::new(pattern).unicode(false).build()?;

        unsafe {
            let mut address = match section {
                Section::Code => self.base_address,
                Section::Heap => 0,
            };

            let mut information = std::mem::zeroed();
            let mut buffer = Vec::new();

            while VirtualQueryEx(
                self.handle,
                Some(address as _),
                &mut information,
                size_of::<MEMORY_BASIC_INFORMATION>(),
            ) != 0
            {
                if information.State == MEM_COMMIT
                    && !information.Protect.contains(PAGE_NOACCESS)
                    && !information.Protect.contains(PAGE_GUARD)
                {
                    if buffer.capacity() < information.RegionSize {
                        buffer.reserve(information.RegionSize - buffer.len());
                    }

                    match ReadProcessMemory(
                        self.handle,
                        address as _,
                        buffer.as_mut_ptr() as _,
                        information.RegionSize,
                        None,
                    ) {
                        Ok(_) => {
                            buffer.set_len(information.RegionSize);
                            if let Some(result) = regex.find(buffer.as_slice()) {
                                let offset = result.start();
                                let region = Region {
                                    address,
                                    data: buffer,
                                };
                                return Ok((region, offset));
                            }
                        }
                        Err(error) => {
                            warn!("Couldn't read process memory at {address:#X} ({error})");
                        }
                    }
                }

                address += information.RegionSize;
            }
        }

        bail!("Couldn't find pattern \"{pattern}\"");
    }

    pub fn read(&self, address: usize, size: usize) -> Result<Vec<u8>> {
        unsafe {
            let mut buffer = Vec::with_capacity(size);
            ReadProcessMemory(
                self.handle,
                address as _,
                buffer.as_mut_ptr() as _,
                size,
                None,
            )?;
            buffer.set_len(size);
            Ok(buffer)
        }
    }

    pub fn read_into<T>(&self, address: usize) -> Result<T> {
        unsafe {
            let mut buffer = std::mem::zeroed();
            ReadProcessMemory(
                self.handle,
                address as _,
                &mut buffer as *mut T as _,
                size_of::<T>(),
                None,
            )?;
            Ok(buffer)
        }
    }

    pub fn write<T>(&self, address: usize, data: &T) -> Result<()> {
        unsafe {
            WriteProcessMemory(
                self.handle,
                address as _,
                data as *const T as _,
                size_of::<T>(),
                None,
            )?;
        }
        Ok(())
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

pub enum Section {
    Code,
    Heap,
}

pub struct Region {
    pub address: usize,
    pub data: Vec<u8>,
}

struct Snapshot {
    handle: HANDLE,
}

impl Snapshot {
    fn new(flags: CREATE_TOOLHELP_SNAPSHOT_FLAGS, pid: u32) -> Result<Self> {
        unsafe {
            Ok(Self {
                handle: CreateToolhelp32Snapshot(flags, pid)?,
            })
        }
    }
}

impl Drop for Snapshot {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}
