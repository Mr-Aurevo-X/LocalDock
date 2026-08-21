use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

const MAX_EVENTS: usize = 80;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEvent {
    pub id: String,
    pub at: String,
    pub kind: String,
    pub detail: String,
}

fn history_path(registry_path: &Path) -> PathBuf {
    registry_path
        .parent()
        .map(|dir| dir.join("history.json"))
        .unwrap_or_else(|| PathBuf::from("history.json"))
}

pub fn list(registry_path: &Path) -> Result<Vec<HistoryEvent>, String> {
    let path = history_path(registry_path);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let data = std::fs::read_to_string(&path).map_err(|err| err.to_string())?;
    serde_json::from_str(&data).map_err(|err| err.to_string())
}

pub fn append(registry_path: &Path, kind: &str, detail: &str) -> Result<Vec<HistoryEvent>, String> {
    let mut events = list(registry_path)?;
    events.insert(
        0,
        HistoryEvent {
            id: Uuid::new_v4().to_string(),
            at: chrono_like_now(),
            kind: kind.to_string(),
            detail: detail.to_string(),
        },
    );
    events.truncate(MAX_EVENTS);
    let path = history_path(registry_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let data = serde_json::to_string_pretty(&events).map_err(|err| err.to_string())?;
    std::fs::write(&path, data).map_err(|err| err.to_string())?;
    Ok(events)
}

fn chrono_like_now() -> String {
    // Local ISO-8601 without extra crate: YYYY-MM-DDTHH:MM:SS
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format_unix(now)
}

fn format_unix(unix: u64) -> String {
    let days = unix / 86400;
    let secs = unix % 86400;
    let hour = secs / 3600;
    let min = (secs % 3600) / 60;
    let sec = secs % 60;
    let (year, month, day) = civil_from_days(days as i64);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i32 + era as i32 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_unix_known_epoch_day() {
        assert_eq!(format_unix(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_unix(1_704_067_200), "2024-01-01T00:00:00Z");
    }
}
