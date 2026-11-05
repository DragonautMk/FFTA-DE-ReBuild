//! Per-catalog favorite IDs; never stores or executes cheat payloads.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, io::Write, path::{Path, PathBuf}};

const MAX_BYTES: u64 = 64 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Data { schema: u32, catalog_sha256: String, favorites: Vec<String> }

fn valid_hash(hash: &str) -> bool {
    hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn validate(data: &Data, catalog: &str) -> Result<(), String> {
    let mut unique = BTreeSet::new();
    if data.schema != 1 || data.catalog_sha256 != catalog || !valid_hash(catalog)
        || data.favorites.len() > 512 || data.favorites.iter().any(|id| {
            id.len() != 7 || !id.starts_with("CH-") || !id[3..].bytes().all(|b| b.is_ascii_digit()) || !unique.insert(id)
        }) { return Err("Invalid cheat favorites; previous file preserved".into()); }
    Ok(())
}
fn read(path: &Path) -> Result<Option<Vec<u8>>, String> {
    match std::fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
        Ok(_) => { crate::debug_content::ordinary(path)?; Ok(Some(crate::diagnostics::read(path, MAX_BYTES)?)) }
    }
}

pub(crate) struct Favorites { path: PathBuf, catalog: String, digest: Option<String>, saved: Vec<String> }
impl Favorites {
    pub fn load(config: &Path, catalog: &str) -> Result<Self, String> {
        if !valid_hash(catalog) { return Err("Invalid cheat catalog identity".into()); }
        let path = config.join("cheat-favorites").join(format!("{catalog}.json"));
        let bytes = read(&path)?;
        let saved = if let Some(bytes) = &bytes {
            let data: Data = serde_json::from_slice(bytes).map_err(|e| format!("Invalid cheat favorites: {e}"))?;
            validate(&data, catalog)?; data.favorites
        } else { Vec::new() };
        Ok(Self { path, catalog: catalog.into(), digest: bytes.as_deref().map(gba_core::ffta_rom::digest), saved })
    }
    pub fn catalog(&self) -> &str { &self.catalog }
    pub fn saved(&self) -> Vec<String> { self.saved.clone() }
    pub fn snapshot(&self) -> Result<Vec<u8>, String> {
        let data=Data{schema:1,catalog_sha256:self.catalog.clone(),favorites:self.saved.clone()};
        validate(&data,&self.catalog)?;serde_json::to_vec_pretty(&data).map_err(|e|e.to_string())
    }
    pub fn validate_snapshot(bytes: &[u8], catalog: &str) -> Result<(), String> {
        if bytes.len() as u64 > MAX_BYTES {return Err("Cheat favorites exceed 64 KiB".into());}
        let data:Data=serde_json::from_slice(bytes).map_err(|e|format!("Invalid cheat favorites: {e}"))?;
        validate(&data,catalog)
    }
    pub fn save(&mut self, favorites: Vec<String>) -> Result<(), String> {
        let data = Data { schema: 1, catalog_sha256: self.catalog.clone(), favorites };
        validate(&data, &self.catalog)?;
        if data.favorites == self.saved { return Ok(()); }
        let actual = read(&self.path)?.as_deref().map(gba_core::ffta_rom::digest);
        if actual != self.digest { return Err("Cheat favorites changed externally; reopen before saving".into()); }
        let parent = self.path.parent().ok_or("Cheat favorites need a directory")?;
        for ancestor in parent.ancestors() { if ancestor.exists() { crate::debug_session::regular(ancestor)?; } }
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        crate::debug_content::ordinary(parent)?;
        let bytes = serde_json::to_vec_pretty(&data).map_err(|e| e.to_string())?;
        if bytes.len() as u64 > MAX_BYTES { return Err("Cheat favorites exceed 64 KiB".into()); }
        let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        temporary.write_all(&bytes).and_then(|_| temporary.as_file().sync_all()).map_err(|e| e.to_string())?;
        temporary.persist(&self.path).map_err(|e| e.to_string())?;
        self.digest = Some(gba_core::ffta_rom::digest(&bytes)); self.saved = data.favorites;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn favorites_survive_reopen_are_catalog_scoped_and_refuse_external_edits() {
        let dir=tempfile::tempdir().unwrap();let one="a".repeat(64);let two="b".repeat(64);
        let mut first=Favorites::load(dir.path(),&one).unwrap();
        first.save(vec!["CH-0341".into()]).unwrap();
        assert_eq!(Favorites::load(dir.path(),&one).unwrap().saved(),vec!["CH-0341"]);
        assert!(Favorites::load(dir.path(),&two).unwrap().saved().is_empty());
        let mut other=Favorites::load(dir.path(),&one).unwrap();
        first.save(vec!["CH-0334".into()]).unwrap();
        assert!(other.save(vec!["CH-0320".into()]).is_err());
        assert_eq!(Favorites::load(dir.path(),&one).unwrap().saved(),vec!["CH-0334"]);
        assert!(first.save(vec!["../bad".into()]).is_err());
    }
    #[test]
    fn malformed_favorites_are_not_overwritten() {
        let dir=tempfile::tempdir().unwrap();let hash="a".repeat(64);
        let path=dir.path().join("cheat-favorites").join(format!("{hash}.json"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();std::fs::write(&path,b"broken").unwrap();
        assert!(Favorites::load(dir.path(),&hash).is_err());assert_eq!(std::fs::read(path).unwrap(),b"broken");
    }
}
