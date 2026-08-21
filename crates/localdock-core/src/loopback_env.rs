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

fn rewrite_inline_host(arg: &str, long: &str, short: Option<&str>) -> Option<String> {
    let long_prefix = format!("{long}=");
    if arg.starts_with(&long_prefix) {
        return Some(format!("{long}={LOOPBACK_HOST}"));
    }
    if let Some(short) = short {
        let short_prefix = format!("{short}=");
        if arg.starts_with(&short_prefix) {
            return Some(format!("{short}={LOOPBACK_HOST}"));
        }
    }
    None
}

fn extend_args(args: &[String], framework: Framework) -> Vec<String> {
    let (long, short) = framework.host_flag();
    let mut extended = Vec::with_capacity(args.len() + 2);
    let mut replaced_value = false;
    let mut idx = 0;

    while idx < args.len() {
        let arg = &args[idx];
        if let Some(rewritten) = rewrite_inline_host(arg, long, short) {
            extended.push(rewritten);
            replaced_value = true;
            idx += 1;
            continue;
        }

        let matches_flag = arg == long || short.is_some_and(|flag| arg == flag);
        if matches_flag {
            extended.push(arg.clone());
            if args.get(idx + 1).is_some_and(|next| !next.starts_with('-')) {
                extended.push(LOOPBACK_HOST.to_string());
                replaced_value = true;
                idx += 2;
                continue;
            }
            idx += 1;
            continue;
        }

        extended.push(arg.clone());
        idx += 1;
    }

    if !replaced_value {
        extended.push(long.to_string());
        extended.push(LOOPBACK_HOST.to_string());
    }

    extended
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
