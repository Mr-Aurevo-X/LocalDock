//! Compare GitHub release tags with the local app version.

pub fn normalize_version(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if let Some(rest) = trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
    {
        if rest.starts_with(|c: char| c.is_ascii_digit()) {
            return format!("v{rest}");
        }
    }
    if trimmed.starts_with(|c: char| c.is_ascii_digit()) {
        return format!("v{trimmed}");
    }
    trimmed.to_string()
}

fn version_tuple(raw: &str) -> Vec<u32> {
    let normalized = normalize_version(raw);
    let digits = normalized.trim_start_matches('v');
    let mut parts = Vec::new();
    for chunk in digits.split(|c: char| !c.is_ascii_digit()) {
        if chunk.is_empty() {
            continue;
        }
        if let Ok(n) = chunk.parse::<u32>() {
            parts.push(n);
        }
    }
    if parts.is_empty() {
        vec![0]
    } else {
        parts
    }
}

pub fn is_remote_newer(remote: &str, local: &str) -> bool {
    version_tuple(remote) > version_tuple(local)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_adds_v_prefix() {
        assert_eq!(normalize_version("0.1.0"), "v0.1.0");
        assert_eq!(normalize_version("v1.2.3"), "v1.2.3");
    }

    #[test]
    fn remote_tag_newer_than_local() {
        assert!(is_remote_newer("v0.2.0", "0.1.0"));
        assert!(!is_remote_newer("v0.1.0", "0.1.0"));
        assert!(!is_remote_newer("0.1.0", "v0.2.0"));
    }
}
