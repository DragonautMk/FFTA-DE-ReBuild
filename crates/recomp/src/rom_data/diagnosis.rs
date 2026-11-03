//! Read-only inspection; never repairs or publishes managed data.
use super::*;
use serde_json::{json, Value};

pub(crate) fn managed_directory(rom: &[u8]) -> Result<PathBuf, String> {
    let source = Extraction::from_rom(rom)?;
    let m = &source.manifest;
    let revision = preparation::storage_root()?
        .join(&m.source_rom_sha256).join(format!("v{}", m.schema))
        .join(format!("e{}-d{}", m.extractor_revision, m.decoder_revision));
    let bytes = read_bounded(&revision.join("current"), 80)?;
    let name = std::str::from_utf8(&bytes).map_err(|_| "Invalid generation pointer")?
        .trim_end_matches('\n');
    if !name.starts_with("generation-") || name.len() <= "generation-".len()
        || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_') {
        return Err("Invalid generation pointer; run recomp rom-data prepare with the selected ROM".into());
    }
    Ok(revision.join(name))
}

pub(crate) fn inspect(rom: &[u8], path: &Path) -> Result<Value, String> {
    let dataset = read_dataset(path)?;
    let resources = gba_core::rom_data::RuntimeResources::from_base_files(rom, dataset.files())?;
    Ok(json!({
        "directory": std::path::absolute(path).map_err(|e| e.to_string())?,
        "manifest_sha256": gba_core::ffta_rom::digest(&read_bounded(&path.join(MANIFEST_FILE), MAX_MANIFEST_FILE_BYTES)?),
        "dataset_sha256": resources.dataset_sha256(),
        "source_sha256": resources.source_sha256(),
        "schema": dataset.manifest.schema,
        "extractor_revision": dataset.manifest.extractor_revision,
        "decoder_revision": dataset.manifest.decoder_revision,
        "resources": dataset.manifest.coverage,
        "validation": "source-authenticated"
    }))
}

pub(crate) fn storage_root() -> Result<PathBuf, String> { preparation::storage_root() }
