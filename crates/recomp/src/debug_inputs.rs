//! Typed, pinned host references shared by the picker and existing tools.
use crate::debug_content::{Entry, Index, Kind, Root};
use menu::ContentSlot;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Clone)]
pub(crate) struct Reference {
    pub entry: Entry,
    pub path: PathBuf,
    index: Arc<Index>,
}
pub(crate) fn kinds(slot: ContentSlot) -> Vec<Kind> {
    use ContentSlot::*;
    vec![match slot {
        Replay => Kind::ReplayProfile,
        Catalog => Kind::SaveCatalog,
        CheatCatalog => Kind::CheatCatalog,
        Session => Kind::Session,
        Recording => Kind::Recording,
        Snapshot => Kind::Snapshot,
        Input => Kind::Input,
        Dataset => Kind::Dataset,
        RuntimeA | RuntimeB => Kind::Runtime,
        Library => Kind::NativeLibrary,
        Labels => Kind::Labels,
        HostTool => Kind::HostTool,
        HostRuntime => Kind::Runtime,
        Campaign => Kind::Campaign,
        Recipe => Kind::Recipe,
        GuestScripts => Kind::GuestScriptCatalog,
        KnownPlan => return vec![Kind::KnownManifest, Kind::Dataset],
    }]
}
impl Reference {
    pub fn source_file(&self)->Result<PathBuf,String>{
        let root=self.index.roots.iter().find(|r|r.id==self.entry.root_id).ok_or("Selected reference root missing")?;
        Ok(root.path.join(&self.entry.relative_path))
    }
    pub fn resolve(index: Arc<Index>, id: &str, slot: Option<ContentSlot>) -> Result<Self, String> {
        let (entry, path) = index.resolve(id, None)?;
        if entry.sha256.is_none() {
            return Err(
                "Selection was not hashed within the scan budget; narrow the root and refresh."
                    .into(),
            );
        }
        if let Some(slot) = slot {
            if !kinds(slot).contains(&entry.kind) {
                return Err("Content type does not match the requested tool input".into());
            }
            preflight(slot, &path, index.rom_sha256.as_deref())?;
        }
        Ok(Self { entry, path, index })
    }
    pub fn explicit(path: &Path, slot: ContentSlot, rom: &str) -> Result<Self, String> {
        let index = Arc::new(Index::scan(
            vec![Root::single_file(path)?],
            Some(rom.into()),
        )?);
        let id = index
            .entries
            .first()
            .ok_or("Explicit input is not an indexed content type")?
            .id
            .clone();
        Self::resolve(index, &id, Some(slot))
    }
    pub fn recheck(&self, slot: ContentSlot, rom: &str) -> Result<PathBuf, String> {
        if self.index.rom_sha256.as_deref() != Some(rom) {
            return Err(
                "Selected input belongs to a different ROM context; select it again.".into(),
            );
        }
        Self::resolve(self.index.clone(), &self.entry.id, Some(slot)).map(|r| r.path)
    }
}
fn preflight(slot: ContentSlot, path: &Path, rom: Option<&str>) -> Result<(), String> {
    use ContentSlot::*;
    match slot {
        Replay => {
            crate::debug_replay::issue_metadata(path)?;
        }
        Catalog => {
            crate::debug_catalog::Catalog::load(path)?;
        }
        CheatCatalog => { crate::debug_cheats::Catalog::load(path)?; }
        Session => {
            let _unstarted = crate::debug_session::replay_command(path)?;
        }
        Recording => {
            let data = crate::debug_recording::export(path)?;
            if rom.is_some_and(|r| r != data.manifest.rom_sha256) {
                return Err("Recording ROM differs from current ROM".into());
            }
        }
        Snapshot => {
            let b = crate::diagnostics::read(
                path,
                gba_core::save_state::MAX_STATE_BYTES as u64 + 240 * 160 * 4 + 48,
            )?;
            crate::save_state::reference_identity(&b, rom)?;
        }
        Input => {
            let b = crate::diagnostics::read(path, 8 * 1024 * 1024)?;
            crate::debug_recording::script(&b, 360_000)?;
        }
        Labels => {
            let b = crate::diagnostics::read(path, 1024 * 1024)?;
            recomp_core::labels::Labels::parse(
                std::str::from_utf8(&b).map_err(|e| e.to_string())?,
                path,
                rom.ok_or("Select a ROM before labels")?,
                32 * 1024 * 1024,
            )?;
        }
        Dataset | KnownPlan => {
            let file = if slot == Dataset {
                path.join("manifest.toml")
            } else if path.is_dir() {
                path.join("manifest.toml")
            } else {
                path.to_owned()
            };
            let bytes =
                crate::diagnostics::read(&file, gba_core::rom_data::MAX_MANIFEST_FILE_BYTES)?;
            let m: gba_core::rom_data::Manifest =
                toml::from_str(std::str::from_utf8(&bytes).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            if m.schema != gba_core::rom_data::SCHEMA
                || rom.is_some_and(|r| r != m.source_rom_sha256)
            {
                return Err("Resource manifest schema/ROM mismatch".into());
            }
        }
        Campaign | Recipe | GuestScripts => { crate::debug_tools::input_preflight(slot, path)?; }
        RuntimeA | RuntimeB | Library | HostRuntime | HostTool => {} // Selection never loads/executes code. Actual ABI validation stays with the worker.
    }
    Ok(())
}

#[cfg(test)]
mod tests;
