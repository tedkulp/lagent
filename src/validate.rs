use std::path::Path;
use std::process::Command;
use anyhow::Result;

pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

const KNOWN_KEYS: &[&str] = &[
    "Label", "Program", "ProgramArguments", "WorkingDirectory",
    "StandardOutPath", "StandardErrorPath", "EnvironmentVariables",
    "RunAtLoad", "StartInterval", "StartCalendarInterval", "KeepAlive",
    "OnDemand", "UserName", "GroupName", "Disabled", "LimitLoadToSessionType",
    "AbandonProcessGroup", "WatchPaths", "QueueDirectories", "SoftResourceLimits",
    "HardResourceLimits", "Nice", "ProcessType", "ThrottleInterval", "ExitTimeout",
    "TimeOut", "inetdCompatibility", "Sockets", "SessionCreate", "Debug",
    "WaitForDebugger", "EnableGlobbing", "EnableTransactions", "LaunchOnlyOnce",
    "MachServices", "BundleProgram",
];

pub fn validate(path: &Path) -> Result<ValidationResult> {
    let mut result = ValidationResult {
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    // Stage 1: syntax via plutil
    let path_s = path.to_str()
        .ok_or_else(|| anyhow::anyhow!("path contains non-UTF-8 characters: {}", path.display()))?;
    let output = Command::new("plutil")
        .args(["-lint", path_s])
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run plutil: {}", e))?;

    if !output.status.success() {
        let msg = String::from_utf8_lossy(&output.stderr);
        result.errors.push(format!("syntax error: {}", msg.trim()));
        return Ok(result);
    }

    // Stage 2: schema via plist crate
    let value: plist::Value = plist::from_file(path)
        .map_err(|e| anyhow::anyhow!("failed to parse plist: {}", e))?;

    let dict = match value.as_dictionary() {
        Some(d) => d,
        None => {
            result.errors.push("plist root must be a dictionary".to_string());
            return Ok(result);
        }
    };

    // Required: Label (non-empty string)
    match dict.get("Label").and_then(|v| v.as_string()) {
        None => result.errors.push("missing required key: Label".to_string()),
        Some(l) if l.is_empty() => result.errors.push("Label must not be empty".to_string()),
        _ => {}
    }

    // Required: Program or ProgramArguments
    if dict.get("Program").is_none() && dict.get("ProgramArguments").is_none() {
        result.errors.push("missing required key: Program or ProgramArguments".to_string());
    }

    // Warn on unknown top-level keys
    for key in dict.keys() {
        if !KNOWN_KEYS.contains(&key.as_str()) {
            result.warnings.push(format!("unknown key: {}", key));
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_valid_plist(dir: &Path, label: &str) -> std::path::PathBuf {
        let path = dir.join("test.plist");
        fs::write(&path, format!(
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
        )).unwrap();
        path
    }

    fn write_plist_without_program(dir: &Path) -> std::path::PathBuf {
        let path = dir.join("noprog.plist");
        fs::write(&path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.example.test</string>
</dict>
</plist>"#
        ).unwrap();
        path
    }

    fn write_plist_without_label(dir: &Path) -> std::path::PathBuf {
        let path = dir.join("nolabel.plist");
        fs::write(&path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>ProgramArguments</key>
    <array><string>/usr/bin/true</string></array>
</dict>
</plist>"#
        ).unwrap();
        path
    }

    fn write_plist_with_unknown_key(dir: &Path) -> std::path::PathBuf {
        let path = dir.join("unknown.plist");
        fs::write(&path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.example.test</string>
    <key>ProgramArguments</key>
    <array><string>/usr/bin/true</string></array>
    <key>SuperUnknownKey</key>
    <string>value</string>
</dict>
</plist>"#
        ).unwrap();
        path
    }

    #[test]
    fn test_valid_plist_has_no_errors() {
        let tmp = TempDir::new().unwrap();
        let path = write_valid_plist(tmp.path(), "com.example.test");
        let result = validate(&path).unwrap();
        assert!(result.errors.is_empty(), "expected no errors, got: {:?}", result.errors);
    }

    #[test]
    fn test_missing_program_is_error() {
        let tmp = TempDir::new().unwrap();
        let path = write_plist_without_program(tmp.path());
        let result = validate(&path).unwrap();
        assert!(result.errors.iter().any(|e| e.contains("Program")));
    }

    #[test]
    fn test_missing_label_is_error() {
        let tmp = TempDir::new().unwrap();
        let path = write_plist_without_label(tmp.path());
        let result = validate(&path).unwrap();
        assert!(result.errors.iter().any(|e| e.contains("Label")));
    }

    #[test]
    fn test_unknown_key_is_warning_not_error() {
        let tmp = TempDir::new().unwrap();
        let path = write_plist_with_unknown_key(tmp.path());
        let result = validate(&path).unwrap();
        assert!(result.errors.is_empty());
        assert!(result.warnings.iter().any(|w| w.contains("SuperUnknownKey")));
    }
}
