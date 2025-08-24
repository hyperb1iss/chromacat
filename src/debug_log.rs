/// Debug logging to file for terminal apps
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref DEBUG_FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);
}

/// Initialize debug logging to a file
pub fn init_debug_log() {
    if let Ok(mut guard) = DEBUG_FILE.lock() {
        match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("/tmp/chromacat_debug.log")
        {
            Ok(file) => {
                *guard = Some(file);
                // Don't call debug_log while we still hold the lock!
                drop(guard); // Release the lock before calling debug_log
                let _ = debug_log("=== ChromaCat Debug Log Started ===");
            }
            Err(_e) => {
                // Silent failure - log file creation failed
            }
        }
    }
}

/// Write a debug message to the log file
pub fn debug_log(msg: &str) -> std::io::Result<()> {
    // eprintln!("DEBUG LOG: {}", msg); // Comment out stderr output for cleaner terminal
    if let Ok(mut guard) = DEBUG_FILE.lock() {
        if let Some(ref mut file) = *guard {
            writeln!(file, "[{}] {}", chrono::Local::now().format("%H:%M:%S%.3f"), msg)?;
            file.flush()?;
        }
    }
    Ok(())
}

/// Convenience macro for debug logging
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        let _ = $crate::debug_log::debug_log(&format!($($arg)*));
    };
}