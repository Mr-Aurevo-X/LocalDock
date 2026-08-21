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

fn contains_token(command: &str, args: &[String], token: &str) -> bool {
    command == token
        || command.ends_with(&format!("/{token}"))
        || command.ends_with(&format!("\\{token}"))
        || args.iter().any(|arg| arg == token)
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

fn has_host_flag(args: &[String], long: &str, short: Option<&str>) -> bool {
    args.iter().any(|arg| {
        arg == long
            || short.is_some_and(|s| arg == s)
            || arg.starts_with(&format!("{long}="))
            || short.is_some_and(|s| arg.starts_with(&format!("{s}=")))
    })
}

fn extend_args(args: &[String], framework: Framework) -> Vec<String> {
    let (long, short) = framework.host_flag();
    if has_host_flag(args, long, short) {
        return args.to_vec();
    }

    let mut extended = args.to_vec();
    extended.push(long.to_string());
    extended.push(LOOPBACK_HOST.to_string());
    extended
}

pub fn apply_loopback(
    command: &str,
    args: &[String],
    preferred_port: Option<u16>,
) -> (Vec<(String, String)>, Vec<String>) {
    let mut env = vec![("HOST".to_string(), LOOPBACK_HOST.to_string())];
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
