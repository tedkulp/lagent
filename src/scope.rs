use std::path::PathBuf;
use anyhow::Result;

pub fn target_dir(user: bool) -> Result<PathBuf> {
    if user {
        let home = std::env::var("HOME")
            .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
        Ok(PathBuf::from(home).join("Library/LaunchAgents"))
    } else {
        check_root()?;
        Ok(system_agents_dir())
    }
}

pub fn check_root() -> Result<()> {
    if !is_root() {
        anyhow::bail!("this operation requires root. Re-run with sudo.");
    }
    Ok(())
}

pub fn is_root() -> bool {
    std::process::Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim() == "0")
        .unwrap_or(false)
}

pub fn system_agents_dir() -> PathBuf {
    PathBuf::from("/Library/LaunchAgents")
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_scope_returns_home_library_path() {
        std::env::set_var("HOME", "/Users/testuser");
        let path = target_dir(true).unwrap();
        assert_eq!(path, PathBuf::from("/Users/testuser/Library/LaunchAgents"));
    }

    #[test]
    fn test_system_scope_returns_library_path() {
        let path = system_agents_dir();
        assert_eq!(path, PathBuf::from("/Library/LaunchAgents"));
    }

    #[test]
    fn test_is_root_returns_bool() {
        let _ = is_root();
    }
}
