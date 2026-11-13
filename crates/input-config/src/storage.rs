//! Portable bundles opt in with a marker beside the actual executable.
//! Resolve on each call: never depend on the working directory or persist the
//! absolute installation path, so moving the whole folder preserves its data.
use std::path::{Path, PathBuf};

pub const PORTABLE_MARKER: &str = "portable.txt";

/// Escape a UTF-8 path for the line-oriented settings format. In particular,
/// '#' must not become a comment and leading/trailing spaces must survive.
pub fn encode_path_value(value: &str) -> String {
    let mut out = String::new();
    for &b in value.as_bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'/' | b'\\' | b'.' | b'_' | b'-' | b':') {
            out.push(char::from(b));
        } else {
            use std::fmt::Write;
            write!(&mut out, "%{b:02X}").expect("writing to a String cannot fail");
        }
    }
    out
}

/// Decode a path value. Invalid escapes, UTF-8 and NUL are rejected.
pub fn decode_path_value(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let high = char::from(*bytes.get(i + 1)?).to_digit(16)?;
            let low = char::from(*bytes.get(i + 2)?).to_digit(16)?;
            decoded.push((high * 16 + low) as u8);
            i += 3;
        } else {
            decoded.push(bytes[i]);
            i += 1;
        }
    }
    if decoded.contains(&0) { return None; }
    String::from_utf8(decoded).ok()
}

/// Resolve a stored directory against the supplied application/workspace root.
/// Absolute user selections remain absolute; relative selections move with the root.
pub fn resolve_asset_root(base: &Path, selected: &str, default: &Path) -> PathBuf {
    let path = if selected.is_empty() { default } else { Path::new(selected) };
    if path.is_absolute() { path.to_path_buf() } else { base.join(path) }
}

/// An installed bundle uses its own root. Development binaries can discover
/// the workspace from their executable, independent of the shell directory.
pub fn asset_anchor() -> Option<PathBuf> {
    let portable = portable_root();
    if let Some(root) = &portable {
        if root.join("debug-session.json").is_file() { return Some(root.clone()); }
    }
    if let Some(exe) = super::exe_dir() {
        if let Some(workspace) = exe.ancestors().find(|p| p.join("Cargo.toml").is_file() && p.join(".git").exists()) {
            return Some(workspace.to_path_buf());
        }
    }
    if let Some(portable) = portable { return Some(portable); }
    super::config_root()
}

pub fn debug_asset_root(config: &super::AvConfig) -> Option<PathBuf> {
    let path = resolve_asset_root(&asset_anchor()?, &config.debug_asset_root, Path::new("../FFTARecomp-private"));
    let mut normalized = PathBuf::new();
    for part in path.components() {
        match part {
            std::path::Component::CurDir => {},
            std::path::Component::ParentDir => { if !normalized.pop() { return None; } },
            other => normalized.push(other.as_os_str()),
        }
    }
    Some(std::fs::canonicalize(&normalized).unwrap_or(normalized))
}

/// Store a relative setting on the same filesystem root, including siblings.
pub fn asset_setting(base: &Path, selected: &Path) -> Result<String, String> {
    let base = std::fs::canonicalize(base).map_err(|e| format!("Asset root base: {e}"))?;
    let selected = std::fs::canonicalize(selected).map_err(|e| format!("Asset folder: {e}"))?;
    if !selected.is_dir() { return Err("Asset folder must be a directory".into()); }
    let a: Vec<_> = base.components().collect();
    let b: Vec<_> = selected.components().collect();
    let shared = a.iter().zip(&b).take_while(|(left, right)| left == right).count();
    let stored = if shared == 0 {
        return Err("Asset folder must be on the same filesystem root as the application".into());
    } else {
        let mut relative = PathBuf::new();
        for _ in shared..a.len() { relative.push(".."); }
        for component in &b[shared..] { relative.push(component.as_os_str()); }
        if relative.as_os_str().is_empty() { relative.push("."); }
        relative
    };
    stored.to_str().map(str::to_owned).ok_or_else(|| "Asset folder path must be valid Unicode".into())
}

pub fn portable_root() -> Option<PathBuf> {
    portable_root_at(&super::exe_dir()?)
}

fn portable_root_at(exe_dir: &Path) -> Option<PathBuf> {
    exe_dir.join(PORTABLE_MARKER).is_file().then(|| exe_dir.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_opts_in_and_data_follows_a_moved_bundle() {
        let base = std::env::temp_dir().join(format!("ffta-portable-{}", std::process::id()));
        let original = base.join("original folder");
        let moved = base.join("moved folder");
        std::fs::create_dir_all(&original).unwrap();
        assert_eq!(portable_root_at(&original), None);
        // A directory of the same name must not opt in.
        std::fs::create_dir(original.join(PORTABLE_MARKER)).unwrap();
        assert_eq!(portable_root_at(&original), None);
        std::fs::remove_dir(original.join(PORTABLE_MARKER)).unwrap();
        std::fs::write(original.join(PORTABLE_MARKER), b"portable").unwrap();
        std::fs::create_dir_all(original.join("Data/config")).unwrap();
        std::fs::write(original.join("Data/config/input.cfg"), b"key.menu = Escape").unwrap();
        assert_eq!(portable_root_at(&original), Some(original.clone()));
        std::fs::rename(&original, &moved).unwrap();
        let root = portable_root_at(&moved).unwrap();
        assert_eq!(std::fs::read(root.join("Data/config/input.cfg")).unwrap(), b"key.menu = Escape");
        assert_eq!(portable_root_at(&original), None);
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn asset_path_roundtrips_through_line_oriented_av_config() {
        let mut config = crate::AvConfig::default();
        config.debug_asset_root = "../asset #100%/ cartella è ".into();
        let text = config.serialize();
        assert_eq!(crate::AvConfig::parse(&text).debug_asset_root, config.debug_asset_root);
        assert_eq!(crate::AvConfig::parse("debug.asset_root = bad%2\n").debug_asset_root, "");
        assert_eq!(crate::AvConfig::parse("debug.asset_root = bad%00path\n").debug_asset_root, "");
    }

    #[test]
    fn relative_asset_path_moves_with_its_anchor() {
        let first = Path::new("/somewhere/first");
        let moved = Path::new("/elsewhere/moved");
        assert_eq!(resolve_asset_root(first, "assets/debug", Path::new("debug_saves")), first.join("assets/debug"));
        assert_eq!(resolve_asset_root(moved, "assets/debug", Path::new("debug_saves")), moved.join("assets/debug"));
        assert_eq!(resolve_asset_root(moved, "", Path::new("debug_saves")), moved.join("debug_saves"));
    }

    #[test]
    fn sibling_selection_is_persisted_relative_to_application_root() {
        let root = std::env::temp_dir().join(format!("ffta-relative-assets-{}", std::process::id()));
        let app = root.join("FFTARecomp");
        let assets = root.join("FFTARecomp-private");
        std::fs::create_dir_all(&app).unwrap();
        std::fs::create_dir_all(&assets).unwrap();
        let setting = asset_setting(&app, &assets).unwrap();
        assert_eq!(Path::new(&setting), Path::new("../FFTARecomp-private"));
        assert_eq!(std::fs::canonicalize(resolve_asset_root(&app, &setting, Path::new("unused"))).unwrap(), std::fs::canonicalize(&assets).unwrap());
        std::fs::remove_dir_all(root).unwrap();
    }
}
