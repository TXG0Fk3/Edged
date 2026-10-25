use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Result of logger initialization containing paths and audit metrics
#[derive(Debug, Clone)]
pub struct LogInitResult {
    pub log_dir: PathBuf,
    pub log_file: PathBuf,
    pub files_pruned: usize,
    pub bytes_freed: u64,
}

/// Powerful thread-safe logger writing to both console (stderr) and daily log files.
pub struct EdgedLogger {
    level: LevelFilter,
    log_to_console: bool,
    log_to_file: bool,
    file_writer: Option<Mutex<File>>,
    #[allow(dead_code)]
    log_file_path: Option<PathBuf>,
}

impl log::Log for EdgedLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Allow user-specified level for edged; limit external libraries to Warn/Error unless in Debug/Trace
        if metadata.target().starts_with("edged") {
            metadata.level() <= self.level
        } else {
            metadata.level() <= self.level && metadata.level() <= Level::Warn
        }
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let (_, timestamp) = format_timestamp(SystemTime::now());
        let target = record.target();
        let args = record.args();

        // 1. Console Output (formatted with ANSI colors when appropriate)
        if self.log_to_console {
            let colored_level = match record.level() {
                Level::Error => "\x1b[1;31mERROR\x1b[0m",
                Level::Warn => "\x1b[1;33mWARN \x1b[0m",
                Level::Info => "\x1b[1;32mINFO \x1b[0m",
                Level::Debug => "\x1b[1;34mDEBUG\x1b[0m",
                Level::Trace => "\x1b[35mTRACE\x1b[0m",
            };
            eprintln!("[{}] [{}] [{}] {}", timestamp, colored_level, target, args);
        }

        // 2. File Output (clean, structured plain-text without ANSI escapes)
        if self.log_to_file {
            if let Some(ref writer_mutex) = self.file_writer {
                if let Ok(mut writer) = writer_mutex.lock() {
                    let _ = writeln!(
                        writer,
                        "[{}] [{:<5}] [{}] {}",
                        timestamp,
                        record.level(),
                        target,
                        args
                    );
                }
            }
        }
    }

    fn flush(&self) {
        if let Some(ref writer_mutex) = self.file_writer {
            if let Ok(mut writer) = writer_mutex.lock() {
                let _ = writer.flush();
            }
        }
    }
}

/// Initialize the global Edged logging subsystem.
///
/// Automatically creates the user log directory, prunes log files older than `retention_days` (default 4 days),
/// opens or appends to today's log file (`edged-YYYY-MM-DD.log`), and updates `latest.log`.
pub fn init(
    config: &crate::config::LoggingConfig,
    cli_level_override: Option<&str>,
) -> Result<LogInitResult, SetLoggerError> {
    let level_filter = if let Some(lvl) = cli_level_override {
        parse_level_filter(lvl)
    } else {
        parse_level_filter(&config.level)
    };

    let log_dir = get_log_dir();
    let _ = fs::create_dir_all(&log_dir);

    // Prune outdated logs older than the retention threshold (4 days by default)
    let (files_pruned, bytes_freed) = if config.enabled {
        prune_old_logs(&log_dir, config.retention_days)
    } else {
        (0, 0)
    };

    let (date_str, _) = format_timestamp(SystemTime::now());
    let log_file_name = format!("edged-{}.log", date_str);
    let log_file_path = log_dir.join(&log_file_name);

    let file_writer = if config.enabled && config.log_to_file {
        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)
        {
            Ok(file) => {
                // Update latest.log symlink or copy for convenience
                update_latest_log_link(&log_dir, &log_file_path);
                Some(Mutex::new(file))
            }
            Err(err) => {
                eprintln!("[edged] Warning: Failed to open log file {:?}: {}", log_file_path, err);
                None
            }
        }
    } else {
        None
    };

    let logger = EdgedLogger {
        level: if config.enabled { level_filter } else { LevelFilter::Off },
        log_to_console: config.enabled && config.log_to_console,
        log_to_file: config.enabled && config.log_to_file && file_writer.is_some(),
        file_writer,
        log_file_path: Some(log_file_path.clone()),
    };

    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(if config.enabled { level_filter } else { LevelFilter::Off });

    Ok(LogInitResult {
        log_dir,
        log_file: log_file_path,
        files_pruned,
        bytes_freed,
    })
}

/// Resolves the canonical user state log directory (respects XDG_STATE_HOME).
/// Typically: `~/.local/state/edged/logs/`
pub fn get_log_dir() -> PathBuf {
    if let Ok(custom) = std::env::var("EDGED_LOG_DIR") {
        return PathBuf::from(custom);
    }

    if let Some(state_dir) = dirs::state_dir() {
        state_dir.join("edged").join("logs")
    } else if let Some(data_dir) = dirs::data_local_dir() {
        data_dir.join("edged").join("logs")
    } else if let Some(cache_dir) = dirs::cache_dir() {
        cache_dir.join("edged").join("logs")
    } else {
        PathBuf::from("/tmp/edged-logs")
    }
}

/// Returns the path to the newest available log file if one exists.
pub fn get_latest_log_path() -> Option<PathBuf> {
    let dir = get_log_dir();
    let latest_symlink = dir.join("latest.log");
    if latest_symlink.exists() {
        return Some(latest_symlink);
    }

    // Find the latest by lexicographical date sort on filenames (edged-YYYY-MM-DD.log)
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.starts_with("edged-") && s.ends_with(".log") && s != "latest.log")
                    .unwrap_or(false)
        })
        .collect();

    files.sort();
    files.pop()
}

/// Prunes log files strictly older than `retention_days` (default 4 days).
///
/// Uses both file modification time (`SystemTime`) and filename calendar dates
/// (`edged-YYYY-MM-DD.log`) to guarantee that logs older than 4 days are cleanly erased.
pub fn prune_old_logs(dir: &Path, retention_days: u32) -> (usize, u64) {
    let mut files_removed = 0;
    let mut bytes_freed = 0;

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return (0, 0),
    };

    let now = SystemTime::now();
    let retention_limit = Duration::from_secs(retention_days as u64 * 86400);

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };

        // Only manage log files starting with 'edged-' and ending with '.log'
        // Skip 'latest.log'
        if !file_name.starts_with("edged-") || !file_name.ends_with(".log") || file_name == "latest.log" {
            continue;
        }

        let mut should_delete = false;

        if let Ok(metadata) = entry.metadata() {
            let file_size = metadata.len();

            // Method 1: File modification time age check
            if let Ok(modified) = metadata.modified() {
                if let Ok(age) = now.duration_since(modified) {
                    if age > retention_limit {
                        should_delete = true;
                    }
                }
            }

            // Method 2: Calendar date in filename 'edged-YYYY-MM-DD.log'
            if !should_delete {
                if let Some(date_part) = file_name
                    .strip_prefix("edged-")
                    .and_then(|s| s.strip_suffix(".log"))
                {
                    if let Some(days_ago) = days_since_date_str(date_part) {
                        if days_ago >= retention_days as i64 {
                            should_delete = true;
                        }
                    }
                }
            }

            if should_delete && fs::remove_file(&path).is_ok() {
                files_removed += 1;
                bytes_freed += file_size;
            }
        }
    }

    (files_removed, bytes_freed)
}

/// Deletes all Edged log files in the given directory. Returns the count of deleted files.
pub fn clear_all_logs(dir: &Path) -> io::Result<usize> {
    let mut count = 0;
    if dir.exists() {
        for entry in fs::read_dir(dir)?.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if (name.starts_with("edged-") && name.ends_with(".log")) || name == "latest.log" {
                        fs::remove_file(&path)?;
                        count += 1;
                    }
                }
            }
        }
    }
    Ok(count)
}

/// Prints diagnostic system environment information to the logger.
pub fn log_system_diagnostics(init_result: &LogInitResult, target_mon: Option<&crate::monitor::MonitorInfo>) {
    log::info!("============================================================");
    log::info!("Edged Screen Ruler v{} initialized", env!("CARGO_PKG_VERSION"));
    log::info!("Log directory: {}", init_result.log_dir.display());
    log::info!("Log session active: {}", init_result.log_file.display());
    if init_result.files_pruned > 0 {
        log::info!(
            "Auto-pruned {} outdated log file(s) (> 4 days old, freed {:.1} KB)",
            init_result.files_pruned,
            init_result.bytes_freed as f64 / 1024.0
        );
    }

    // Desktop and Wayland environment diagnostics
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "Unknown".to_string());
    let session = std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "Unknown".to_string());
    let wayland_display = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "None".to_string());
    let compositor = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .map(|_| "Hyprland")
        .or_else(|_| std::env::var("SWAYSOCK").map(|_| "Sway"))
        .unwrap_or("Generic Wayland");

    log::info!(
        "Environment: Desktop='{}', SessionType='{}', WaylandDisplay='{}', Compositor='{}'",
        desktop, session, wayland_display, compositor
    );

    if let Some(mon) = target_mon {
        log::info!(
            "Target Monitor: '{}' (Geometry: {}x{} at ({},{}), Scale: {:.2}, Primary: {}, Focused: {})",
            mon.name,
            mon.width,
            mon.height,
            mon.x,
            mon.y,
            mon.scale,
            mon.is_primary,
            mon.is_focused
        );
    } else {
        log::info!("Target Monitor: All displays / Virtual desktop bounds");
    }
    log::info!("============================================================");
}

/// Updates or creates the `latest.log` symlink / copy in the log directory.
fn update_latest_log_link(dir: &Path, current_log: &Path) {
    let latest_path = dir.join("latest.log");
    let _ = fs::remove_file(&latest_path);

    #[cfg(unix)]
    {
        if let Some(file_name) = current_log.file_name() {
            let _ = std::os::unix::fs::symlink(file_name, &latest_path);
        }
    }

    #[cfg(not(unix))]
    {
        let _ = fs::copy(current_log, &latest_path);
    }
}

/// Formats a `SystemTime` into `(date_string, full_timestamp)` using local timezone with UTC fallback.
pub fn format_timestamp(now: SystemTime) -> (String, String) {
    let duration = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs() as i64;
    let millis = duration.subsec_millis();

    let mut tm = libc::tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: std::ptr::null(),
    };

    let res = unsafe { libc::localtime_r(&secs as *const i64, &mut tm) };
    if !res.is_null() {
        let year = 1900 + tm.tm_year;
        let month = 1 + tm.tm_mon;
        let day = tm.tm_mday;
        let hour = tm.tm_hour;
        let min = tm.tm_min;
        let sec = tm.tm_sec;

        let date_str = format!("{:04}-{:02}-{:02}", year, month, day);
        let timestamp_str = format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
            year, month, day, hour, min, sec, millis
        );
        (date_str, timestamp_str)
    } else {
        // Fallback civil day calculation
        let mut days = secs / 86400;
        let time_of_day = (secs % 86400) as u32;
        let hour = time_of_day / 3600;
        let min = (time_of_day % 3600) / 60;
        let sec = time_of_day % 60;

        days += 719468;
        let era = if days >= 0 { days } else { days - 146096 } / 146097;
        let doe = (days - era * 146097) as u32;
        let yoe = (doe - doe / 1024 + doe / 1461 - doe / 146096) / 365;
        let y = (yoe as i64) + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };

        let date_str = format!("{:04}-{:02}-{:02}", y, m, d);
        let timestamp_str = format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03} UTC",
            y, m, d, hour, min, sec, millis
        );
        (date_str, timestamp_str)
    }
}

/// Calculate elapsed days between a `YYYY-MM-DD` date string and now.
fn days_since_date_str(date_str: &str) -> Option<i64> {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let y: i64 = parts[0].parse().ok()?;
    let m: i64 = parts[1].parse().ok()?;
    let d: i64 = parts[2].parse().ok()?;

    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }

    let m_adj = if m <= 2 { m + 9 } else { m - 3 };
    let y_adj = if m <= 2 { y - 1 } else { y };
    let era = if y_adj >= 0 { y_adj } else { y_adj - 399 } / 400;
    let yoe = (y_adj - era * 400) as u32;
    let doy = (153 * m_adj + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy as u32;
    let file_days = era * 146097 + doe as i64 - 719468;

    let now_secs = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    let now_days = (now_secs / 86400) as i64;

    Some(now_days - file_days)
}

/// Parse string representation of log level into `LevelFilter`.
pub fn parse_level_filter(s: &str) -> LevelFilter {
    match s.trim().to_lowercase().as_str() {
        "off" => LevelFilter::Off,
        "error" => LevelFilter::Error,
        "warn" | "warning" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_format_timestamp() {
        let (date, full) = format_timestamp(SystemTime::now());
        assert_eq!(date.len(), 10); // YYYY-MM-DD
        assert!(full.contains(&date));
        assert!(full.contains(':'));
    }

    #[test]
    fn test_days_since_date_str() {
        let (today_str, _) = format_timestamp(SystemTime::now());
        let days = days_since_date_str(&today_str);
        assert_eq!(days, Some(0));

        let ancient = days_since_date_str("2020-01-01");
        assert!(ancient.unwrap() > 1000);
    }

    #[test]
    fn test_prune_old_logs() {
        let temp_dir = std::env::temp_dir().join(format!("edged_test_prune_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        // Create log files: 1 old (10 days ago), 1 recent (today)
        let old_log = temp_dir.join("edged-2026-01-01.log");
        let new_log = temp_dir.join("edged-2026-10-25.log");
        let non_log = temp_dir.join("something_else.txt");

        File::create(&old_log).unwrap();
        File::create(&new_log).unwrap();
        File::create(&non_log).unwrap();

        // Prune logs older than 4 days
        let (pruned_count, _) = prune_old_logs(&temp_dir, 4);

        assert_eq!(pruned_count, 1);
        assert!(!old_log.exists(), "Old log should have been deleted");
        assert!(new_log.exists(), "New log should be preserved");
        assert!(non_log.exists(), "Non-log file should not be touched");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_clear_all_logs() {
        let temp_dir = std::env::temp_dir().join(format!("edged_test_clear_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        File::create(temp_dir.join("edged-2026-10-24.log")).unwrap();
        File::create(temp_dir.join("edged-2026-10-25.log")).unwrap();
        File::create(temp_dir.join("latest.log")).unwrap();

        let cleared = clear_all_logs(&temp_dir).unwrap();
        assert_eq!(cleared, 3);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
