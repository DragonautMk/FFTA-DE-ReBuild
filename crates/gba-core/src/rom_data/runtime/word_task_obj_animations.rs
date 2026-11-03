//! Typed fields for u16-task banks; no guest timing or selector replacement.
use super::{obj_animations::ObjAnimationResources, WordTaskObjStats};
use crate::{
    ffta_rom,
    rom_data::{
        obj_animations::Banks, obj_lists::List, word_task_obj_animations::WordTaskObjAnimations,
        SourceRange,
    },
};

pub(super) struct WordTaskObjResources {
    fields: ObjAnimationResources,
}
impl WordTaskObjResources {
    pub(super) fn new(
        data: WordTaskObjAnimations,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() {
            return Err("Word-task OBJ source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("Word-task OBJ typed source binding mismatch".into());
            }
        }
        Ok(Self {
            fields: ObjAnimationResources::new(Banks {
                banks: data.banks,
                selector_tables: vec![],
            })?,
        })
    }
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x411 | 0x416)
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        if !Self::source_page(index) {
            return prior;
        }
        self.fields.guest_read_projected(index, rom, prior)
    }
    pub(super) fn host_list(&self, offset: u32) -> Option<&List> {
        self.fields.host_list(offset)
    }
    pub(super) fn stats(&self) -> WordTaskObjStats {
        self.fields.stats()
    }
    pub(super) fn resident_bytes(&self) -> usize {
        self.fields.resident_bytes()
    }
}
