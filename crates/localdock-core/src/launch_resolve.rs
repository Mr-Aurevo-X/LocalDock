use crate::host_exec;
use crate::LocalDockError;
use serde_json::Value;
use std::path::Path;
use std::process::Stdio;

const PACKAGE_RUNNERS: &[&str] = &["pnpm", "npm", "yarn", "bun"];

pub fn resolve_launch(
    command: &str,
    args: &[String],
    cwd: &Path,
) -> Result<(String, Vec<String>), LocalDockError> {
    if program_available(command) {
        return Ok((command.to_string(), args.to_vec()));
    }

    if let Some((program, rest)) = unwrap_node_script(cwd, command, args) {
        if program_available(&program) {
            return Ok((program, rest));
        }
    }

    if matches!(command, "pnpm" | "yarn" | "bun") && program_available("npm") {
        return Ok(("npm".to_string(), args.to_vec()));
    }

    Err(LocalDockError::StartFailed(format!(
        "{command} introuvable sur Linux. Installe-le, ou enregistre un script `node …`."
    )))
}

pub fn unwrap_node_script(
    cwd: &Path,
    command: &str,
    args: &[String],
) -> Option<(String, Vec<String>)> {
    if !PACKAGE_RUNNERS.iter().any(|runner| *runner == command) {
        return None;
    }
    if args.first().map(String::as_str) != Some("run") {
        return None;
    }
    let script_name = args.get(1)?;
    let body = read_package_script(cwd, script_name)?;
    let (program, mut rest) = parse_node_invocation(&body)?;
    let extra = match args.get(2..) {
        Some([first, tail @ ..]) if first == "--" => tail,
        Some(extra) => extra,
        None => &[],
    };
    rest.extend(extra.iter().cloned());
    Some((program, rest))
}

pub fn parse_node_invocation(script: &str) -> Option<(String, Vec<String>)> {
    let trimmed = script.trim();
    if trimmed.is_empty()
        || trimmed.contains('&')
        || trimmed.contains('|')
        || trimmed.contains(';')
        || trimmed.contains('>')
        || trimmed.contains('<')
    {
        return None;
    }
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    let program = parts.first()?;
    let base = program.rsplit(['/', '\\']).next()?;
    if !base.eq_ignore_ascii_case("node") && !base.eq_ignore_ascii_case("nodejs") {
        return None;
    }
    Some((
        "node".to_string(),
        parts.iter().skip(1).map(|part| (*part).to_string()).collect(),
    ))
}

fn read_package_script(cwd: &Path, name: &str) -> Option<String> {
    let raw = std::fs::read_to_string(cwd.join("package.json")).ok()?;
    let json: Value = serde_json::from_str(&raw).ok()?;
    json.get("scripts")?
        .get(name)?
        .as_str()
        .map(str::to_string)
}

fn program_available(command: &str) -> bool {
    let path = Path::new(command);
    if path.components().count() > 1 {
        return path.is_file();
    }
    host_binary_exists(command)
}

fn host_binary_exists(name: &str) -> bool {
    if !is_safe_binary_name(name) {
        return false;
    }
    let mut cmd = host_exec::host_command(
        "sh",
        &["-c".into(), format!("command -v {name}")],
        None,
        &[],
    );
    cmd.stdout(Stdio::null()).stderr(Stdio::null());
    cmd.status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn is_safe_binary_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_node_script_body() {
        let (program, args) =
            parse_node_invocation("node scripts/dev-local.mjs").expect("node script");
        assert_eq!(program, "node");
        assert_eq!(args, vec!["scripts/dev-local.mjs"]);
        assert!(parse_node_invocation("pnpm --filter x run dev").is_none());
        assert!(parse_node_invocation("node a && node b").is_none());
    }

    #[test]
    fn unwraps_pnpm_run_to_node() {
        let dir = std::env::temp_dir().join(format!("ld-launch-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("package.json"),
            r#"{"scripts":{"dev:local":"node scripts/dev-local.mjs"}}"#,
        )
        .unwrap();
        let (program, args) = unwrap_node_script(
            &dir,
            "pnpm",
            &["run".into(), "dev:local".into()],
        )
        .expect("unwrap");
        assert_eq!(program, "node");
        assert_eq!(args, vec!["scripts/dev-local.mjs"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_pnpm_run_falls_back_to_node() {
        if host_binary_exists("pnpm") {
            return;
        }
        let dir = std::env::temp_dir().join(format!("ld-launch-fb-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("package.json"),
            r#"{"scripts":{"dev:local":"node scripts/dev-local.mjs"}}"#,
        )
        .unwrap();
        let (program, args) = resolve_launch(
            "pnpm",
            &["run".into(), "dev:local".into()],
            &dir,
        )
        .expect("fallback");
        assert_eq!(program, "node");
        assert_eq!(args, vec!["scripts/dev-local.mjs"]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
