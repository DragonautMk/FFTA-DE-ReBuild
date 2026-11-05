//! Staged typed owner for conditional layered OBJ data; guest timing stays live.
use super::{obj_animations::ObjAnimationResources, ObjAnimationStats};
use crate::ffta_rom;
use crate::rom_data::{
    layered_obj_animations::{Descriptor, LayeredObjAnimations, DESCRIPTOR_BYTES, DESCRIPTOR_ROOT},
    obj_animations::Banks,
    obj_lists::List,
    SourceRange,
};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayeredObjStats {
    pub fields: ObjAnimationStats,
    pub descriptor_reads: u64,
    pub descriptor_bytes: u64,
}

pub(super) struct LayeredObjResources {
    descriptors: Vec<Descriptor>,
    fields: ObjAnimationResources,
    descriptor_reads: AtomicU64,
    descriptor_bytes: AtomicU64,
}

impl LayeredObjResources {
    pub(super) fn new(data: LayeredObjAnimations, sources: &[SourceRange]) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() {
            return Err("Layered OBJ source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("Layered OBJ typed source binding mismatch".into());
            }
        }
        let mut banks = data.banks;
        banks.sort_by_key(|bank| bank.object_root);
        Ok(Self {
            descriptors: data.descriptors,
            fields: ObjAnimationResources::new(Banks {
                banks,
                selector_tables: vec![],
            })?,
            descriptor_reads: AtomicU64::new(0),
            descriptor_bytes: AtomicU64::new(0),
        })
    }

    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x3ae | 0xb8e..=0xb93)
    }

    fn descriptor_byte(&self, index: usize) -> Option<u8> {
        let at = index.checked_sub(DESCRIPTOR_ROOT as usize)?;
        let descriptor = self.descriptors.get(at / DESCRIPTOR_BYTES)?;
        let offset = at % DESCRIPTOR_BYTES;
        match offset {
            0..=3 => Some((descriptor.object_root + 0x08000000).to_le_bytes()[offset]),
            4..=7 => Some((descriptor.sequence_root + 0x08000000).to_le_bytes()[offset - 4]),
            _ => descriptor.opaque.get(offset - 8).copied(),
        }
    }

    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        if !Self::source_page(index)
            && !Self::source_page(index.saturating_add(N.saturating_sub(1)))
        {
            return prior;
        }
        let prior = self.fields.guest_read_projected(index, rom, prior);
        let mut used = 0;
        let bytes = std::array::from_fn(|i| match self.descriptor_byte(index + i) {
            Some(byte) => {
                used += 1;
                byte
            }
            None => prior.as_ref().map_or_else(|| rom[index + i], |p| p[i]),
        });
        if used == 0 {
            return prior;
        }
        self.descriptor_reads.fetch_add(1, Ordering::Relaxed);
        self.descriptor_bytes.fetch_add(used, Ordering::Relaxed);
        Some(bytes)
    }

    pub(super) fn host_list(&self, offset: u32) -> Option<&List> {
        self.fields.host_list(offset)
    }

    pub(super) fn stats(&self) -> LayeredObjStats {
        LayeredObjStats {
            fields: self.fields.stats(),
            descriptor_reads: self.descriptor_reads.load(Ordering::Relaxed),
            descriptor_bytes: self.descriptor_bytes.load(Ordering::Relaxed),
        }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.descriptors.capacity() * std::mem::size_of::<Descriptor>()
            + self.fields.resident_bytes()
            - std::mem::size_of::<ObjAnimationResources>()
    }
}

#[cfg(test)]
mod tests;
