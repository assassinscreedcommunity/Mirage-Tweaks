use log::{Level, LevelFilter, Log, Metadata, Record};
use once_cell::sync::Lazy;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

static TEMP_FILE: Lazy<Mutex<Option<File>>> =
    Lazy::new(|| Mutex::new(File::create(temp_path()).ok()));
static DID_ERROR: AtomicBool = AtomicBool::new(false);

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("[{}] {}", record.level(), record.args());
            if let Some(file) = TEMP_FILE.lock().unwrap().as_mut() {
                let _ = writeln!(file, "[{}] {}", record.level(), record.args());
            }
            if record.level() == Level::Error {
                DID_ERROR.store(true, Ordering::Relaxed);
            }
        }
    }

    fn flush(&self) {}
}

pub struct LogGuard;

impl Drop for LogGuard {
    fn drop(&mut self) {
        if DID_ERROR.load(Ordering::Relaxed) {
            let _ = std::fs::copy(temp_path(), "mirage-tweaks.log");
        }
    }
}

fn temp_path() -> PathBuf {
    let mut temp_path = std::env::temp_dir();
    temp_path.push("mirage-tweaks.log");
    temp_path
}

#[must_use]
pub fn set_logger() -> LogGuard {
    if log::set_logger(&Logger).is_ok() {
        log::set_max_level(LevelFilter::Info);
    }
    LogGuard
}
