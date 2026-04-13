use std::path::{Path, PathBuf};
use anyhow::Result;

#[derive(Debug)]
pub struct Agent {
    pub path: PathBuf,
    pub label: String,
}

/// Resolve an agent by label or filename stem in the given directory.
pub fn resolve(id: &str, dir: &Path) -> Result<Agent> {
    let needle = id.trim_end_matches(".plist");
    let mut matches: Vec<Agent> = Vec::new();

    for entry in std::fs::read_dir(dir)
        .map_err(|e| anyhow::anyhow!("cannot read directory {}: {}", dir.display(), e))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("plist") {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        if stem == needle {
            let label = read_label(&path).unwrap_or_else(|_| stem.clone());
            matches.push(Agent { path, label });
            continue;
        }

        if let Ok(label) = read_label(&path) {
            if label == needle {
                matches.push(Agent { path, label });
            }
        }
    }

    match matches.len() {
        0 => anyhow::bail!("no agent found matching '{}'", id),
        1 => Ok(matches.remove(0)),
        _ => {
            let names: Vec<_> = matches.iter().map(|a| a.label.as_str()).collect();
            anyhow::bail!(
                "ambiguous agent name '{}', matches: {}",
                id,
                names.join(", ")
            )
        }
    }
}

/// Read the Label key from a plist file.
pub fn read_label(path: &Path) -> Result<String> {
    let value: plist::Value = plist::from_file(path)
        .map_err(|e| anyhow::anyhow!("failed to parse {}: {}", path.display(), e))?;
    let dict = value
        .as_dictionary()
        .ok_or_else(|| anyhow::anyhow!("plist root is not a dictionary"))?;
    dict.get("Label")
        .and_then(|v| v.as_string())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Label key not found or not a string"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_plist(dir: &Path, stem: &str, label: &str) -> PathBuf {
        let path = dir.join(format!("{}.plist", stem));
        let content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array><string>/usr/bin/true</string></array>
</dict>
</plist>"#
        );
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_resolve_by_stem() {
        let dir = TempDir::new().unwrap();
        write_plist(dir.path(), "com.example.myapp", "com.example.myapp");
        let agent = resolve("com.example.myapp", dir.path()).unwrap();
        assert_eq!(agent.label, "com.example.myapp");
    }

    #[test]
    fn test_resolve_strips_plist_suffix() {
        let dir = TempDir::new().unwrap();
        write_plist(dir.path(), "com.example.myapp", "com.example.myapp");
        let agent = resolve("com.example.myapp.plist", dir.path()).unwrap();
        assert_eq!(agent.label, "com.example.myapp");
    }

    #[test]
    fn test_resolve_by_label_when_stem_differs() {
        let dir = TempDir::new().unwrap();
        write_plist(dir.path(), "myapp", "com.example.myapp");
        let agent = resolve("com.example.myapp", dir.path()).unwrap();
        assert_eq!(agent.label, "com.example.myapp");
    }

    #[test]
    fn test_resolve_no_match_returns_error() {
        let dir = TempDir::new().unwrap();
        let result = resolve("nonexistent", dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no agent found"));
    }

    #[test]
    fn test_resolve_ambiguous_returns_error() {
        let dir = TempDir::new().unwrap();
        write_plist(dir.path(), "com.example.myapp", "com.example.myapp");
        write_plist(dir.path(), "com.example.myapp-alt", "com.example.myapp");
        let result = resolve("com.example.myapp", dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("ambiguous"));
    }

    #[test]
    fn test_resolve_path_is_correct() {
        let dir = TempDir::new().unwrap();
        let expected_path = write_plist(dir.path(), "com.example.myapp", "com.example.myapp");
        let agent = resolve("com.example.myapp", dir.path()).unwrap();
        assert_eq!(agent.path, expected_path);
    }
}
