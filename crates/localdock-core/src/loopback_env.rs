use std::net::IpAddr;

const LOOPBACK_HOST: &str = "127.0.0.1";

enum Framework {
    Vite,
    Next,
    Uvicorn,
}

impl Framework {
    fn host_flag(&self) -> (&'static str, Option<&'static str>) {
        match self {
            Framework::Vite | Framework::Uvicorn => ("--host", None),
            Framework::Next => ("-H", None),
        }
    }
}

fn normalize_framework_token(value: &str) -> String {
    let basename = value.rsplit(['/', '\\']).next().unwrap_or(value);
    let lower = basename.to_ascii_lowercase();

    for suffix in [".cmd", ".bat", ".exe", ".js", ".mjs"] {
        if let Some(stripped) = lower.strip_suffix(suffix) {
            return stripped.to_string();
        }
    }

    lower
}

fn contains_token(command: &str, args: &[String], token: &str) -> bool {
    normalize_framework_token(command) == token
        || args
            .iter()
            .any(|arg| normalize_framework_token(arg) == token)
}

fn detect_framework(command: &str, args: &[String]) -> Option<Framework> {
    if contains_token(command, args, "vite") {
        Some(Framework::Vite)
    } else if contains_token(command, args, "next") {
        Some(Framework::Next)
    } else if contains_token(command, args, "uvicorn") {
        Some(Framework::Uvicorn)
    } else {
        None
    }
}

fn is_loopback_host(value: &str) -> bool {
    let trimmed = value.trim().trim_matches(|c| c == '[' || c == ']');
    if trimmed.eq_ignore_ascii_case("localhost") {
        return true;
    }
    trimmed
        .parse::<IpAddr>()
        .map(|ip| ip.is_loopback())
        .unwrap_or(false)
}

fn is_host_flag(arg: &str, long: &str, short: Option<&str>) -> bool {
    arg == long || short.is_some_and(|s| arg == s)
}

fn rewrite_inline_host(arg: &str, long: &str, short: Option<&str>) -> Option<String> {
    let long_prefix = format!("{long}=");
    if let Some(value) = arg.strip_prefix(&long_prefix) {
        return Some(if is_loopback_host(value) {
            arg.to_string()
        } else {
            format!("{long}={LOOPBACK_HOST}")
        });
    }
    if let Some(short) = short {
        let short_prefix = format!("{short}=");
        if let Some(value) = arg.strip_prefix(&short_prefix) {
            return Some(if is_loopback_host(value) {
                arg.to_string()
            } else {
                format!("{short}={LOOPBACK_HOST}")
            });
        }
    }
    None
}

fn extend_args(args: &[String], framework: Framework) -> Vec<String> {
    let (long, short) = framework.host_flag();
    rewrite_or_inject_host(args, long, short)
}

fn rewrite_or_inject_host(args: &[String], long: &str, short: Option<&str>) -> Vec<String> {
    let mut out = Vec::with_capacity(args.len() + 2);
    let mut found_valued = false;
    let mut idx = 0;

    while idx < args.len() {
        let arg = &args[idx];
        if let Some(rewritten) = rewrite_inline_host(arg, long, short) {
            out.push(rewritten);
            found_valued = true;
            idx += 1;
            continue;
        }

        if is_host_flag(arg, long, short) {
            out.push(arg.clone());
            if let Some(next) = args.get(idx + 1) {
                if !next.starts_with('-') {
                    if is_loopback_host(next) {
                        out.push(next.clone());
                    } else {
                        out.push(LOOPBACK_HOST.to_string());
                    }
                    found_valued = true;
                    idx += 2;
                    continue;
                }
            }
            out.push(LOOPBACK_HOST.to_string());
            found_valued = true;
            idx += 1;
            continue;
        }

        out.push(arg.clone());
        idx += 1;
    }

    if !found_valued {
        out.push(long.to_string());
        out.push(LOOPBACK_HOST.to_string());
    }
    out
}

pub fn apply_loopback(
    command: &str,
    args: &[String],
    preferred_port: Option<u16>,
) -> (Vec<(String, String)>, Vec<String>) {
    let mut env = vec![
        ("HOST".to_string(), LOOPBACK_HOST.to_string()),
        ("HOSTNAME".to_string(), LOOPBACK_HOST.to_string()),
    ];
    if let Some(port) = preferred_port {
        env.push(("PORT".to_string(), port.to_string()));
    }

    let out_args = match detect_framework(command, args) {
        Some(framework) => extend_args(args, framework),
        None => args.to_vec(),
    };

    (env, out_args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_framework_from_npx_vite() {
        assert!(matches!(
            detect_framework("npx", &["vite".into()]),
            Some(Framework::Vite)
        ));
    }
}
