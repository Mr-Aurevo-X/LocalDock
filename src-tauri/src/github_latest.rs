use localdock_core::version::{is_remote_newer, normalize_version};
use serde::{Deserialize, Serialize};
use std::process::Command;

const PRODUCT_REPO: &str = "Mr-Aurevo-X/LocalDock";
const LATEST_API: &str = "https://api.github.com/repos/Mr-Aurevo-X/LocalDock/releases/latest";
pub const RELEASES_PAGE: &str = "https://github.com/Mr-Aurevo-X/LocalDock/releases/latest";
pub const REPO_PAGE: &str = "https://github.com/Mr-Aurevo-X/LocalDock";

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestCheck {
    pub ok: bool,
    pub skipped: bool,
    pub newer: bool,
    pub local: String,
    pub remote: String,
    pub html_url: String,
    pub repo: String,
}

pub fn check_latest(local_version: &str, enabled: bool) -> Result<LatestCheck, String> {
    let local = normalize_version(local_version);
    if !enabled {
        return Ok(LatestCheck {
            ok: true,
            skipped: true,
            newer: false,
            local,
            remote: String::new(),
            html_url: RELEASES_PAGE.to_string(),
            repo: PRODUCT_REPO.to_string(),
        });
    }

    let body = fetch_latest_json()?;
    parse_latest_body(&body, &local)
}

pub(crate) fn parse_latest_body(body: &str, local: &str) -> Result<LatestCheck, String> {
    let local = normalize_version(local);
    if body.trim().is_empty() {
        return Ok(no_remote_release(local));
    }
    let value: serde_json::Value = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(err) => {
            if body.contains("404") || body.contains("Not Found") {
                return Ok(no_remote_release(local));
            }
            return Err(format!("github latest json: {err}"));
        }
    };
    let tag = value
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if tag.is_empty() {
        return Ok(no_remote_release(local));
    }
    let html = value
        .get("html_url")
        .and_then(|v| v.as_str())
        .filter(|url| url.starts_with("https://github.com/Mr-Aurevo-X/LocalDock"))
        .unwrap_or(RELEASES_PAGE)
        .to_string();
    let remote = normalize_version(&tag);

    Ok(LatestCheck {
        ok: true,
        skipped: false,
        newer: is_remote_newer(&remote, &local),
        local,
        remote,
        html_url: html,
        repo: PRODUCT_REPO.to_string(),
    })
}

fn no_remote_release(local: String) -> LatestCheck {
    LatestCheck {
        ok: true,
        skipped: false,
        newer: false,
        local,
        remote: String::new(),
        html_url: RELEASES_PAGE.to_string(),
        repo: PRODUCT_REPO.to_string(),
    }
}

fn fetch_latest_json() -> Result<String, String> {
    let mut cmd = Command::new(curl_bin());
    cmd.args([
        "-sS",
        "--max-time",
        "8",
        "-H",
        "Accept: application/vnd.github+json",
        "-H",
        "User-Agent: LocalDock-ReleaseNotice",
        LATEST_API,
    ]);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd
        .output()
        .map_err(|err| format!("github latest fetch failed to start: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "github latest fetch failed with status {}: {stderr}",
            output.status
        ));
    }

    String::from_utf8(output.stdout).map_err(|err| format!("github latest encoding: {err}"))
}

fn curl_bin() -> &'static str {
    #[cfg(windows)]
    {
        "curl.exe"
    }
    #[cfg(not(windows))]
    {
        "curl"
    }
}

pub fn allowlisted_release_url(url: &str) -> Result<String, String> {
    let url = url.trim();
    if url == RELEASES_PAGE
        || url == REPO_PAGE
        || url.starts_with("https://github.com/Mr-Aurevo-X/LocalDock/releases")
    {
        return Ok(url.to_string());
    }
    Err("only the LocalDock GitHub repo / releases URLs are allowed".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_latest_body_treats_not_found_as_no_release() {
        let info = parse_latest_body(
            r#"{"message":"Not Found"}"#,
            "0.1.0",
        )
        .unwrap();
        assert!(info.ok);
        assert!(!info.newer);
        assert!(info.remote.is_empty());
    }

    #[test]
    fn parse_latest_body_reads_tag() {
        let info = parse_latest_body(
            r#"{"tag_name":"v0.2.0","html_url":"https://github.com/Mr-Aurevo-X/LocalDock/releases/tag/v0.2.0"}"#,
            "0.1.0",
        )
        .unwrap();
        assert!(info.newer);
        assert_eq!(info.remote, "v0.2.0");
    }
}
