use std::path::{Path, PathBuf};
use anyhow::Result;
use sha2::{Digest, Sha256};

fn state_dir() -> Result<PathBuf> {
    let dir = if let Ok(d) = std::env::var("LAGENT_STATE_DIR") {
        PathBuf::from(d)
    } else {
        let home = std::env::var("HOME")
            .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
        PathBuf::from(home).join(".local/share/lagent")
    };
    std::fs::create_dir_all(&dir)
        .map_err(|e| anyhow::anyhow!("cannot create state directory {}: {}", dir.display(), e))?;
    Ok(dir)
}

fn hash_path(label: &str) -> Result<PathBuf> {
    Ok(state_dir()?.join(format!("{}.hash", label)))
}

/// Compute SHA256 of a plist file's raw bytes.
pub fn compute_hash(plist_path: &Path) -> Result<String> {
    let bytes = std::fs::read(plist_path)
        .map_err(|e| anyhow::anyhow!("cannot read {}: {}", plist_path.display(), e))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Read stored hash for a label. Returns None if no hash exists.
pub fn read_hash(label: &str) -> Result<Option<String>> {
    let path = hash_path(label)?;
    if path.exists() {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("cannot read hash file: {}", e))?;
        Ok(Some(content.trim().to_string()))
    } else {
        Ok(None)
    }
}

/// Write the hash of a plist file for a label.
pub fn write_hash(label: &str, plist_path: &Path) -> Result<()> {
    let hash = compute_hash(plist_path)?;
    std::fs::write(hash_path(label)?, hash)
        .map_err(|e| anyhow::anyhow!("cannot write hash file: {}", e))
}

/// Delete the stored hash for a label. No-op if it doesn't exist.
pub fn delete_hash(label: &str) -> Result<()> {
    let path = hash_path(label)?;
    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| anyhow::anyhow!("cannot delete hash file: {}", e))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup(tmp: &TempDir) -> PathBuf {
        let state = tmp.path().join("state");
        std::fs::create_dir_all(&state).unwrap();
        std::env::set_var("LAGENT_STATE_DIR", &state);
        let plist = tmp.path().join("test.plist");
        std::fs::write(&plist, b"<plist>test</plist>").unwrap();
        plist
    }

    #[test]
    fn test_compute_hash_is_deterministic() {
        let tmp = TempDir::new().unwrap();
        let plist = tmp.path().join("a.plist");
        std::fs::write(&plist, b"hello").unwrap();
        let h1 = compute_hash(&plist).unwrap();
        let h2 = compute_hash(&plist).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_compute_hash_differs_for_different_content() {
        let tmp = TempDir::new().unwrap();
        let p1 = tmp.path().join("a.plist");
        let p2 = tmp.path().join("b.plist");
        std::fs::write(&p1, b"hello").unwrap();
        std::fs::write(&p2, b"world").unwrap();
        assert_ne!(compute_hash(&p1).unwrap(), compute_hash(&p2).unwrap());
    }

    #[test]
    fn test_read_hash_returns_none_when_missing() {
        let tmp = TempDir::new().unwrap();
        setup(&tmp);
        let result = read_hash("com.example.nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_write_then_read_hash() {
        let tmp = TempDir::new().unwrap();
        let plist = setup(&tmp);
        write_hash("com.example.test", &plist).unwrap();
        let stored = read_hash("com.example.test").unwrap().unwrap();
        let expected = compute_hash(&plist).unwrap();
        assert_eq!(stored, expected);
    }

    #[test]
    fn test_delete_hash_removes_file() {
        let tmp = TempDir::new().unwrap();
        let plist = setup(&tmp);
        write_hash("com.example.test", &plist).unwrap();
        assert!(read_hash("com.example.test").unwrap().is_some());
        delete_hash("com.example.test").unwrap();
        assert!(read_hash("com.example.test").unwrap().is_none());
    }

    #[test]
    fn test_delete_hash_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        setup(&tmp);
        delete_hash("com.example.nonexistent").unwrap();
    }
}
