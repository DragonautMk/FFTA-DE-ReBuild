//! Project authenticated Thumb literal operands without replacing guest code.
use super::*;
use crate::rom_data::thumb_pc_literals as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ThumbPcLiteralsStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub literals_mask: [u64; 6],
    pub additional_source_bytes: u64,
    pub additional_mask: [u64; 2],
    pub observed_source_bytes: u64,
    pub observed_mask: [u64; 7],
    pub home_anchored_bytes: u64,
    pub home_anchored_mask: u64,
    pub home_observed_bytes: u64,
    pub home_observed_mask: u64,
    pub red_spring_observed_bytes: u64,
    pub red_spring_observed_mask: [u64; 4],
    pub red_spring_anchored_bytes: u64,
    pub red_spring_anchored_mask: u64,
    pub post_indexed_observed_bytes: u64,
    pub post_indexed_observed_mask: u64,
    pub post_indexed_anchored_bytes: u64,
    pub post_indexed_anchored_mask: u64,
    pub bank_observed_bytes: u64,
    pub bank_observed_mask: [u64; 2],
    pub blue_screw_observed_bytes: u64,
    pub blue_screw_observed_mask: u64,
    pub soil_evidence_observed_bytes: u64,
    pub soil_evidence_observed_mask: [u64; 3],
    pub soil_146_observed_bytes: u64,
    pub soil_146_observed_mask: u64,
    pub firaga_observed_bytes: u64,
    pub firaga_observed_mask: u64,
    pub firaga_2d2e_observed_bytes: u64,
    pub firaga_2d2e_observed_mask: u64,
    pub firaga_eb83_ebc_observed_bytes: u64,
    pub firaga_eb83_ebc_observed_mask: u64,
    pub firaga_code_observed_bytes: u64,
    pub firaga_code_observed_mask: u64,
    pub firaga_mixed_observed_bytes: u64,
    pub firaga_mixed_observed_mask: u64,
    pub firaga_residual_observed_bytes: u64,
    pub firaga_residual_observed_mask: u64,
    pub white_flowers_observed_bytes: u64,
    pub white_flowers_observed_mask: u64,
    pub white_flowers_additional_bytes: u64,
    pub white_flowers_additional_mask: u64,
    pub white_flowers_later_bytes: u64,
    pub white_flowers_later_mask: u64,
    pub white_flowers_thirteen_bytes: u64,
    pub white_flowers_thirteen_mask: u64,
    pub white_flowers_many_bytes: u64,
    pub white_flowers_many_mask: [u64; 2],
    pub white_flowers_bulk_bytes: u64,
    pub white_flowers_bulk_mask: [u64; 2],
    pub white_flowers_remaining_bytes: u64,
    pub white_flowers_remaining_mask: [u64; 3],
    pub white_flowers_mixed_bytes: u64,
    pub white_flowers_mixed_mask: u64,
    pub white_flowers_final_bytes: u64,
    pub white_flowers_final_mask: u64,
    pub star_cross_observed_bytes: u64,
    pub star_cross_observed_mask: u64,
}

pub(super) struct ThumbPcLiteralsResources {
    data: model::ThumbPcLiterals,
    reads: AtomicU64,
    bytes: AtomicU64,
    literals: [AtomicU64; 6],
    additional_bytes: AtomicU64,
    additional: [AtomicU64; 2],
    observed_bytes: AtomicU64,
    observed: [AtomicU64; 7],
    home_anchored_bytes: AtomicU64,
    home_anchored: AtomicU64,
    home_observed_bytes: AtomicU64,
    home_observed: AtomicU64,
    red_spring_observed_bytes: AtomicU64,
    red_spring_observed: [AtomicU64; 4],
    red_spring_anchored_bytes: AtomicU64,
    red_spring_anchored: AtomicU64,
    post_indexed_observed_bytes: AtomicU64,
    post_indexed_observed: AtomicU64,
    post_indexed_anchored_bytes: AtomicU64,
    post_indexed_anchored: AtomicU64,
    bank_observed_bytes: AtomicU64,
    bank_observed: [AtomicU64; 2],
    blue_screw_observed_bytes: AtomicU64,
    blue_screw_observed: AtomicU64,
    soil_evidence_observed_bytes: AtomicU64,
    soil_evidence_observed: [AtomicU64; 3],
    soil_146_observed_bytes: AtomicU64,
    soil_146_observed: AtomicU64,
    firaga_observed_bytes: AtomicU64,
    firaga_observed: AtomicU64,
    firaga_2d2e_observed_bytes: AtomicU64,
    firaga_2d2e_observed: AtomicU64,
    firaga_eb83_ebc_observed_bytes: AtomicU64,
    firaga_eb83_ebc_observed: AtomicU64,
    firaga_code_observed_bytes: AtomicU64,
    firaga_code_observed: AtomicU64,
    firaga_mixed_observed_bytes: AtomicU64,
    firaga_mixed_observed: AtomicU64,
    firaga_residual_observed_bytes: AtomicU64,
    firaga_residual_observed: AtomicU64,
    white_flowers_observed_bytes: AtomicU64,
    white_flowers_observed: AtomicU64,
    white_flowers_additional_bytes: AtomicU64,
    white_flowers_additional: AtomicU64,
    white_flowers_later_bytes: AtomicU64,
    white_flowers_later: AtomicU64,
    white_flowers_thirteen_bytes: AtomicU64,
    white_flowers_thirteen: AtomicU64,
    white_flowers_many_bytes: AtomicU64,
    white_flowers_many: [AtomicU64; 2],
    white_flowers_bulk_bytes: AtomicU64,
    white_flowers_bulk: [AtomicU64; 2],
    white_flowers_remaining_bytes: AtomicU64,
    white_flowers_remaining: [AtomicU64; 3],
    white_flowers_mixed_bytes: AtomicU64,
    white_flowers_mixed: AtomicU64,
    white_flowers_final_bytes: AtomicU64,
    white_flowers_final: AtomicU64,
    star_cross_observed_bytes: AtomicU64,
    star_cross_observed: AtomicU64,
}

impl ThumbPcLiteralsResources {
    pub(super) fn source_page(index: usize) -> bool {
        let first = index >> 12;
        let last = index.saturating_add(3) >> 12;
        model::SOURCE_PAGES.binary_search(&first).is_ok()
            || model::SOURCE_PAGES.binary_search(&last).is_ok()
    }

    pub(super) fn new(
        data: model::ThumbPcLiterals,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        data.validate()?;
        if sources.len()
            != model::LITERAL_COUNT
                + model::ADDITIONAL_COUNT
                + model::OBSERVED_COUNT
                + model::HOME_ANCHORED_COUNT
                + model::HOME_OBSERVED_COUNT
                + model::RED_SPRING_OBSERVED_COUNT
                + model::RED_SPRING_ANCHORED_COUNT
                + model::POST_INDEXED_OBSERVED_COUNT
                + model::POST_INDEXED_ANCHORED_COUNT
                + model::BANK_OBSERVED_COUNT
                + model::BLUE_SCREW_OBSERVED_COUNT
                + model::SOIL_EVIDENCE_OBSERVED_COUNT
                + model::SOIL_146_OBSERVED_COUNT
                + model::FIRAGA_OBSERVED_COUNT
                + model::FIRAGA_2D2E_OBSERVED_COUNT
                + model::FIRAGA_EB83_EBC_OBSERVED_COUNT
                + model::FIRAGA_CODE_OBSERVED_COUNT
                + model::FIRAGA_MIXED_OBSERVED_COUNT
                + model::FIRAGA_RESIDUAL_OBSERVED_COUNT
                + data.white_flowers_observed.as_ref().map_or(0, Vec::len)
                + data.white_flowers_additional.as_ref().map_or(0, Vec::len)
                + data.white_flowers_later.as_ref().map_or(0, Vec::len)
                + data.white_flowers_thirteen.as_ref().map_or(0, Vec::len)
                + data.white_flowers_many.as_ref().map_or(0, Vec::len)
                + data.white_flowers_bulk.as_ref().map_or(0, Vec::len)
                + data.white_flowers_remaining.as_ref().map_or(0, Vec::len)
                + data.white_flowers_mixed.as_ref().map_or(0, Vec::len)
                + data.white_flowers_final.as_ref().map_or(0, Vec::len)
                + data.star_cross_observed.as_ref().map_or(0, Vec::len)
        {
            return Err("Thumb literal source count mismatch".into());
        }
        for (literal, source) in data
            .literals
            .iter()
            .chain(&data.additional)
            .chain(&data.observed)
            .chain(&data.home_anchored)
            .chain(&data.home_observed)
            .chain(&data.red_spring_observed)
            .chain(&data.red_spring_anchored)
            .chain(&data.post_indexed_observed)
            .chain(&data.post_indexed_anchored)
            .chain(&data.bank_observed)
            .chain(&data.blue_screw_observed)
            .chain(&data.soil_evidence_observed)
            .chain(&data.soil_146_observed)
            .chain(&data.firaga_observed)
            .chain(&data.firaga_2d2e_observed)
            .chain(&data.firaga_eb83_ebc_observed)
            .chain(&data.firaga_code_observed)
            .chain(&data.firaga_mixed_observed)
            .chain(&data.firaga_residual_observed)
            .chain(data.white_flowers_observed.iter().flatten())
            .chain(data.white_flowers_additional.iter().flatten())
            .chain(data.white_flowers_later.iter().flatten())
            .chain(data.white_flowers_thirteen.iter().flatten())
            .chain(data.white_flowers_many.iter().flatten())
            .chain(data.white_flowers_bulk.iter().flatten())
            .chain(data.white_flowers_remaining.iter().flatten())
            .chain(data.white_flowers_mixed.iter().flatten())
            .chain(data.white_flowers_final.iter().flatten())
            .chain(data.star_cross_observed.iter().flatten())
            .zip(sources)
        {
            if source.offset != literal.source_offset
                || source.length != 4
                || source.role != model::SOURCE_ROLE
                || source.sha256 != ffta_rom::digest(&literal.value.to_le_bytes())
            {
                return Err("Thumb literal source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            literals: std::array::from_fn(|_| AtomicU64::new(0)),
            additional_bytes: AtomicU64::new(0),
            additional: std::array::from_fn(|_| AtomicU64::new(0)),
            observed_bytes: AtomicU64::new(0),
            observed: std::array::from_fn(|_| AtomicU64::new(0)),
            home_anchored_bytes: AtomicU64::new(0),
            home_anchored: AtomicU64::new(0),
            home_observed_bytes: AtomicU64::new(0),
            home_observed: AtomicU64::new(0),
            red_spring_observed_bytes: AtomicU64::new(0),
            red_spring_observed: std::array::from_fn(|_| AtomicU64::new(0)),
            red_spring_anchored_bytes: AtomicU64::new(0),
            red_spring_anchored: AtomicU64::new(0),
            post_indexed_observed_bytes: AtomicU64::new(0),
            post_indexed_observed: AtomicU64::new(0),
            post_indexed_anchored_bytes: AtomicU64::new(0),
            post_indexed_anchored: AtomicU64::new(0),
            bank_observed_bytes: AtomicU64::new(0),
            bank_observed: std::array::from_fn(|_| AtomicU64::new(0)),
            blue_screw_observed_bytes: AtomicU64::new(0),
            blue_screw_observed: AtomicU64::new(0),
            soil_evidence_observed_bytes: AtomicU64::new(0),
            soil_evidence_observed: std::array::from_fn(|_| AtomicU64::new(0)),
            soil_146_observed_bytes: AtomicU64::new(0),
            soil_146_observed: AtomicU64::new(0),
            firaga_observed_bytes: AtomicU64::new(0),
            firaga_observed: AtomicU64::new(0),
            firaga_2d2e_observed_bytes: AtomicU64::new(0),
            firaga_2d2e_observed: AtomicU64::new(0),
            firaga_eb83_ebc_observed_bytes: AtomicU64::new(0),
            firaga_eb83_ebc_observed: AtomicU64::new(0),
            firaga_code_observed_bytes: AtomicU64::new(0),
            firaga_code_observed: AtomicU64::new(0),
            firaga_mixed_observed_bytes: AtomicU64::new(0),
            firaga_mixed_observed: AtomicU64::new(0),
            firaga_residual_observed_bytes: AtomicU64::new(0),
            firaga_residual_observed: AtomicU64::new(0),
            white_flowers_observed_bytes: AtomicU64::new(0),
            white_flowers_observed: AtomicU64::new(0),
            white_flowers_additional_bytes: AtomicU64::new(0),
            white_flowers_additional: AtomicU64::new(0),
            white_flowers_later_bytes: AtomicU64::new(0),
            white_flowers_later: AtomicU64::new(0),
            white_flowers_thirteen_bytes: AtomicU64::new(0),
            white_flowers_thirteen: AtomicU64::new(0),
            white_flowers_many_bytes: AtomicU64::new(0),
            white_flowers_many: std::array::from_fn(|_| AtomicU64::new(0)),
            white_flowers_bulk_bytes: AtomicU64::new(0),
            white_flowers_bulk: std::array::from_fn(|_| AtomicU64::new(0)),
            white_flowers_remaining_bytes: AtomicU64::new(0),
            white_flowers_remaining: std::array::from_fn(|_| AtomicU64::new(0)),
            white_flowers_mixed_bytes: AtomicU64::new(0),
            white_flowers_mixed: AtomicU64::new(0),
            white_flowers_final_bytes: AtomicU64::new(0),
            white_flowers_final: AtomicU64::new(0),
            star_cross_observed_bytes: AtomicU64::new(0),
            star_cross_observed: AtomicU64::new(0),
        })
    }

    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        let cells: [_; N] = std::array::from_fn(|i| self.data.byte_at((index + i) as u32));
        if cells.iter().all(Option::is_none) {
            return prior;
        }
        let mut count = 0;
        let mut literals = [0u64; 6];
        let mut additional_count = 0;
        let mut additional = [0u64; 2];
        let mut observed_count = 0;
        let mut observed = [0u64; 7];
        let mut home_count = 0;
        let mut home = 0u64;
        let mut home_observed_count = 0;
        let mut home_observed = 0u64;
        let mut red_spring_observed_count = 0;
        let mut red_spring_observed = [0u64; 4];
        let mut red_spring_anchored_count = 0;
        let mut red_spring_anchored = 0u64;
        let mut post_indexed_observed_count = 0;
        let mut post_indexed_observed = 0u64;
        let mut post_indexed_anchored_count = 0;
        let mut post_indexed_anchored = 0u64;
        let mut bank_observed_count = 0;
        let mut bank_observed = [0u64; 2];
        let mut blue_screw_observed_count = 0;
        let mut blue_screw_observed = 0u64;
        let mut soil_evidence_observed_count = 0;
        let mut soil_evidence_observed = [0u64; 3];
        let mut soil_146_observed_count = 0;
        let mut soil_146_observed = 0u64;
        let mut firaga_observed_count = 0;
        let mut firaga_observed = 0u64;
        let mut firaga_2d2e_observed_count = 0;
        let mut firaga_2d2e_observed = 0u64;
        let mut firaga_eb83_ebc_observed_count = 0;
        let mut firaga_eb83_ebc_observed = 0u64;
        let mut firaga_code_observed_count = 0;
        let mut firaga_code_observed = 0u64;
        let mut firaga_mixed_observed_count = 0;
        let mut firaga_mixed_observed = 0u64;
        let mut firaga_residual_observed_count = 0;
        let mut firaga_residual_observed = 0u64;
        let mut white_flowers_observed_count = 0;
        let mut white_flowers_observed = 0u64;
        let mut white_flowers_additional_count = 0;
        let mut white_flowers_additional = 0u64;
        let mut white_flowers_later_count = 0;
        let mut white_flowers_later = 0u64;
        let mut white_flowers_thirteen_count = 0;
        let mut white_flowers_thirteen = 0u64;
        let mut white_flowers_many_count = 0;
        let mut white_flowers_many = [0u64; 2];
        let mut white_flowers_bulk_count = 0;
        let mut white_flowers_bulk = [0u64; 2];
        let mut white_flowers_remaining_count = 0;
        let mut white_flowers_remaining = [0u64; 3];
        let mut white_flowers_mixed_count = 0;
        let mut white_flowers_mixed = 0u64;
        let mut white_flowers_final_count = 0;
        let mut white_flowers_final = 0u64;
        let mut star_cross_observed_count = 0;
        let mut star_cross_observed = 0u64;
        let result = std::array::from_fn(|i| match cells[i] {
            Some((byte, word)) => {
                count += 1;
                if word < model::LITERAL_COUNT {
                    literals[word / 64] |= 1u64 << (word % 64);
                } else if word < model::LITERAL_COUNT + model::ADDITIONAL_COUNT {
                    let word = word - model::LITERAL_COUNT;
                    additional_count += 1;
                    additional[word / 64] |= 1u64 << (word % 64);
                } else if word
                    < model::LITERAL_COUNT + model::ADDITIONAL_COUNT + model::OBSERVED_COUNT
                {
                    let word = word - model::LITERAL_COUNT - model::ADDITIONAL_COUNT;
                    observed_count += 1;
                    observed[word / 64] |= 1u64 << (word % 64);
                } else if word
                    < model::LITERAL_COUNT
                        + model::ADDITIONAL_COUNT
                        + model::OBSERVED_COUNT
                        + model::HOME_ANCHORED_COUNT
                {
                    let word = word
                        - model::LITERAL_COUNT
                        - model::ADDITIONAL_COUNT
                        - model::OBSERVED_COUNT;
                    home_count += 1;
                    home |= 1u64 << word;
                } else if word
                    < model::LITERAL_COUNT
                        + model::ADDITIONAL_COUNT
                        + model::OBSERVED_COUNT
                        + model::HOME_ANCHORED_COUNT
                        + model::HOME_OBSERVED_COUNT
                {
                    let word = word
                        - model::LITERAL_COUNT
                        - model::ADDITIONAL_COUNT
                        - model::OBSERVED_COUNT
                        - model::HOME_ANCHORED_COUNT;
                    home_observed_count += 1;
                    home_observed |= 1u64 << word;
                } else if word
                    < model::LITERAL_COUNT
                        + model::ADDITIONAL_COUNT
                        + model::OBSERVED_COUNT
                        + model::HOME_ANCHORED_COUNT
                        + model::HOME_OBSERVED_COUNT
                        + model::RED_SPRING_OBSERVED_COUNT
                {
                    let word = word
                        - model::LITERAL_COUNT
                        - model::ADDITIONAL_COUNT
                        - model::OBSERVED_COUNT
                        - model::HOME_ANCHORED_COUNT
                        - model::HOME_OBSERVED_COUNT;
                    red_spring_observed_count += 1;
                    red_spring_observed[word / 64] |= 1u64 << (word % 64);
                } else if word < model::LITERAL_COUNT + model::ADDITIONAL_COUNT + model::OBSERVED_COUNT + model::HOME_ANCHORED_COUNT + model::HOME_OBSERVED_COUNT + model::RED_SPRING_OBSERVED_COUNT + model::RED_SPRING_ANCHORED_COUNT {
                    let word = word - model::LITERAL_COUNT - model::ADDITIONAL_COUNT - model::OBSERVED_COUNT - model::HOME_ANCHORED_COUNT - model::HOME_OBSERVED_COUNT - model::RED_SPRING_OBSERVED_COUNT;
                    red_spring_anchored_count += 1;
                    red_spring_anchored |= 1u64 << word;
                } else if word < model::LITERAL_COUNT + model::ADDITIONAL_COUNT + model::OBSERVED_COUNT + model::HOME_ANCHORED_COUNT + model::HOME_OBSERVED_COUNT + model::RED_SPRING_OBSERVED_COUNT + model::RED_SPRING_ANCHORED_COUNT + model::POST_INDEXED_OBSERVED_COUNT {
                    let word = word - model::LITERAL_COUNT - model::ADDITIONAL_COUNT - model::OBSERVED_COUNT - model::HOME_ANCHORED_COUNT - model::HOME_OBSERVED_COUNT - model::RED_SPRING_OBSERVED_COUNT - model::RED_SPRING_ANCHORED_COUNT;
                    post_indexed_observed_count += 1;
                    post_indexed_observed |= 1u64 << word;
                } else if word < model::LITERAL_COUNT + model::ADDITIONAL_COUNT + model::OBSERVED_COUNT + model::HOME_ANCHORED_COUNT + model::HOME_OBSERVED_COUNT + model::RED_SPRING_OBSERVED_COUNT + model::RED_SPRING_ANCHORED_COUNT + model::POST_INDEXED_OBSERVED_COUNT + model::POST_INDEXED_ANCHORED_COUNT {
                    let word = word - model::LITERAL_COUNT - model::ADDITIONAL_COUNT - model::OBSERVED_COUNT - model::HOME_ANCHORED_COUNT - model::HOME_OBSERVED_COUNT - model::RED_SPRING_OBSERVED_COUNT - model::RED_SPRING_ANCHORED_COUNT - model::POST_INDEXED_OBSERVED_COUNT;
                    post_indexed_anchored_count += 1;
                    post_indexed_anchored |= 1u64 << word;
                } else if word < model::LITERAL_COUNT + model::ADDITIONAL_COUNT + model::OBSERVED_COUNT + model::HOME_ANCHORED_COUNT + model::HOME_OBSERVED_COUNT + model::RED_SPRING_OBSERVED_COUNT + model::RED_SPRING_ANCHORED_COUNT + model::POST_INDEXED_OBSERVED_COUNT + model::POST_INDEXED_ANCHORED_COUNT + model::BANK_OBSERVED_COUNT {
                    let word = word - model::LITERAL_COUNT - model::ADDITIONAL_COUNT - model::OBSERVED_COUNT - model::HOME_ANCHORED_COUNT - model::HOME_OBSERVED_COUNT - model::RED_SPRING_OBSERVED_COUNT - model::RED_SPRING_ANCHORED_COUNT - model::POST_INDEXED_OBSERVED_COUNT - model::POST_INDEXED_ANCHORED_COUNT;
                    bank_observed_count += 1;
                    bank_observed[word / 64] |= 1u64 << (word % 64);
                } else if word < model::LITERAL_COUNT + model::ADDITIONAL_COUNT + model::OBSERVED_COUNT + model::HOME_ANCHORED_COUNT + model::HOME_OBSERVED_COUNT + model::RED_SPRING_OBSERVED_COUNT + model::RED_SPRING_ANCHORED_COUNT + model::POST_INDEXED_OBSERVED_COUNT + model::POST_INDEXED_ANCHORED_COUNT + model::BANK_OBSERVED_COUNT + model::BLUE_SCREW_OBSERVED_COUNT {
                    let word = word - model::LITERAL_COUNT - model::ADDITIONAL_COUNT - model::OBSERVED_COUNT - model::HOME_ANCHORED_COUNT - model::HOME_OBSERVED_COUNT - model::RED_SPRING_OBSERVED_COUNT - model::RED_SPRING_ANCHORED_COUNT - model::POST_INDEXED_OBSERVED_COUNT - model::POST_INDEXED_ANCHORED_COUNT - model::BANK_OBSERVED_COUNT;
                    blue_screw_observed_count += 1;
                    blue_screw_observed |= 1u64 << word;
                } else if word < model::LITERAL_COUNT + model::ADDITIONAL_COUNT + model::OBSERVED_COUNT + model::HOME_ANCHORED_COUNT + model::HOME_OBSERVED_COUNT + model::RED_SPRING_OBSERVED_COUNT + model::RED_SPRING_ANCHORED_COUNT + model::POST_INDEXED_OBSERVED_COUNT + model::POST_INDEXED_ANCHORED_COUNT + model::BANK_OBSERVED_COUNT + model::BLUE_SCREW_OBSERVED_COUNT + model::SOIL_EVIDENCE_OBSERVED_COUNT {
                    let word = word - model::LITERAL_COUNT - model::ADDITIONAL_COUNT - model::OBSERVED_COUNT - model::HOME_ANCHORED_COUNT - model::HOME_OBSERVED_COUNT - model::RED_SPRING_OBSERVED_COUNT - model::RED_SPRING_ANCHORED_COUNT - model::POST_INDEXED_OBSERVED_COUNT - model::POST_INDEXED_ANCHORED_COUNT - model::BANK_OBSERVED_COUNT - model::BLUE_SCREW_OBSERVED_COUNT;
                    soil_evidence_observed_count += 1;
                    soil_evidence_observed[word / 64] |= 1u64 << (word % 64);
                } else if word < model::LITERAL_COUNT + model::ADDITIONAL_COUNT + model::OBSERVED_COUNT + model::HOME_ANCHORED_COUNT + model::HOME_OBSERVED_COUNT + model::RED_SPRING_OBSERVED_COUNT + model::RED_SPRING_ANCHORED_COUNT + model::POST_INDEXED_OBSERVED_COUNT + model::POST_INDEXED_ANCHORED_COUNT + model::BANK_OBSERVED_COUNT + model::BLUE_SCREW_OBSERVED_COUNT + model::SOIL_EVIDENCE_OBSERVED_COUNT + model::SOIL_146_OBSERVED_COUNT {
                    let word = word - model::LITERAL_COUNT - model::ADDITIONAL_COUNT - model::OBSERVED_COUNT - model::HOME_ANCHORED_COUNT - model::HOME_OBSERVED_COUNT - model::RED_SPRING_OBSERVED_COUNT - model::RED_SPRING_ANCHORED_COUNT - model::POST_INDEXED_OBSERVED_COUNT - model::POST_INDEXED_ANCHORED_COUNT - model::BANK_OBSERVED_COUNT - model::BLUE_SCREW_OBSERVED_COUNT - model::SOIL_EVIDENCE_OBSERVED_COUNT;
                    soil_146_observed_count += 1;
                    soil_146_observed |= 1u64 << word;
                } else if word < model::LITERAL_COUNT + model::ADDITIONAL_COUNT + model::OBSERVED_COUNT + model::HOME_ANCHORED_COUNT + model::HOME_OBSERVED_COUNT + model::RED_SPRING_OBSERVED_COUNT + model::RED_SPRING_ANCHORED_COUNT + model::POST_INDEXED_OBSERVED_COUNT + model::POST_INDEXED_ANCHORED_COUNT + model::BANK_OBSERVED_COUNT + model::BLUE_SCREW_OBSERVED_COUNT + model::SOIL_EVIDENCE_OBSERVED_COUNT + model::SOIL_146_OBSERVED_COUNT + model::FIRAGA_OBSERVED_COUNT {
                    let word = word - model::LITERAL_COUNT - model::ADDITIONAL_COUNT - model::OBSERVED_COUNT - model::HOME_ANCHORED_COUNT - model::HOME_OBSERVED_COUNT - model::RED_SPRING_OBSERVED_COUNT - model::RED_SPRING_ANCHORED_COUNT - model::POST_INDEXED_OBSERVED_COUNT - model::POST_INDEXED_ANCHORED_COUNT - model::BANK_OBSERVED_COUNT - model::BLUE_SCREW_OBSERVED_COUNT - model::SOIL_EVIDENCE_OBSERVED_COUNT - model::SOIL_146_OBSERVED_COUNT;
                    firaga_observed_count += 1;
                    firaga_observed |= 1u64 << word;
                } else if word < model::LITERAL_COUNT + model::ADDITIONAL_COUNT + model::OBSERVED_COUNT + model::HOME_ANCHORED_COUNT + model::HOME_OBSERVED_COUNT + model::RED_SPRING_OBSERVED_COUNT + model::RED_SPRING_ANCHORED_COUNT + model::POST_INDEXED_OBSERVED_COUNT + model::POST_INDEXED_ANCHORED_COUNT + model::BANK_OBSERVED_COUNT + model::BLUE_SCREW_OBSERVED_COUNT + model::SOIL_EVIDENCE_OBSERVED_COUNT + model::SOIL_146_OBSERVED_COUNT + model::FIRAGA_OBSERVED_COUNT + model::FIRAGA_2D2E_OBSERVED_COUNT {
                    let word = word - model::LITERAL_COUNT - model::ADDITIONAL_COUNT - model::OBSERVED_COUNT - model::HOME_ANCHORED_COUNT - model::HOME_OBSERVED_COUNT - model::RED_SPRING_OBSERVED_COUNT - model::RED_SPRING_ANCHORED_COUNT - model::POST_INDEXED_OBSERVED_COUNT - model::POST_INDEXED_ANCHORED_COUNT - model::BANK_OBSERVED_COUNT - model::BLUE_SCREW_OBSERVED_COUNT - model::SOIL_EVIDENCE_OBSERVED_COUNT - model::SOIL_146_OBSERVED_COUNT - model::FIRAGA_OBSERVED_COUNT;
                    firaga_2d2e_observed_count += 1;
                    firaga_2d2e_observed |= 1u64 << word;
                } else if word < model::LITERAL_COUNT + model::ADDITIONAL_COUNT + model::OBSERVED_COUNT + model::HOME_ANCHORED_COUNT + model::HOME_OBSERVED_COUNT + model::RED_SPRING_OBSERVED_COUNT + model::RED_SPRING_ANCHORED_COUNT + model::POST_INDEXED_OBSERVED_COUNT + model::POST_INDEXED_ANCHORED_COUNT + model::BANK_OBSERVED_COUNT + model::BLUE_SCREW_OBSERVED_COUNT + model::SOIL_EVIDENCE_OBSERVED_COUNT + model::SOIL_146_OBSERVED_COUNT + model::FIRAGA_OBSERVED_COUNT + model::FIRAGA_2D2E_OBSERVED_COUNT + model::FIRAGA_EB83_EBC_OBSERVED_COUNT {
                    let word = word - model::LITERAL_COUNT - model::ADDITIONAL_COUNT - model::OBSERVED_COUNT - model::HOME_ANCHORED_COUNT - model::HOME_OBSERVED_COUNT - model::RED_SPRING_OBSERVED_COUNT - model::RED_SPRING_ANCHORED_COUNT - model::POST_INDEXED_OBSERVED_COUNT - model::POST_INDEXED_ANCHORED_COUNT - model::BANK_OBSERVED_COUNT - model::BLUE_SCREW_OBSERVED_COUNT - model::SOIL_EVIDENCE_OBSERVED_COUNT - model::SOIL_146_OBSERVED_COUNT - model::FIRAGA_OBSERVED_COUNT - model::FIRAGA_2D2E_OBSERVED_COUNT;
                    firaga_eb83_ebc_observed_count += 1;
                    firaga_eb83_ebc_observed |= 1u64 << word;
                } else if word < model::LITERAL_COUNT + model::ADDITIONAL_COUNT + model::OBSERVED_COUNT + model::HOME_ANCHORED_COUNT + model::HOME_OBSERVED_COUNT + model::RED_SPRING_OBSERVED_COUNT + model::RED_SPRING_ANCHORED_COUNT + model::POST_INDEXED_OBSERVED_COUNT + model::POST_INDEXED_ANCHORED_COUNT + model::BANK_OBSERVED_COUNT + model::BLUE_SCREW_OBSERVED_COUNT + model::SOIL_EVIDENCE_OBSERVED_COUNT + model::SOIL_146_OBSERVED_COUNT + model::FIRAGA_OBSERVED_COUNT + model::FIRAGA_2D2E_OBSERVED_COUNT + model::FIRAGA_EB83_EBC_OBSERVED_COUNT + model::FIRAGA_CODE_OBSERVED_COUNT {
                    let word = word - model::LITERAL_COUNT - model::ADDITIONAL_COUNT - model::OBSERVED_COUNT - model::HOME_ANCHORED_COUNT - model::HOME_OBSERVED_COUNT - model::RED_SPRING_OBSERVED_COUNT - model::RED_SPRING_ANCHORED_COUNT - model::POST_INDEXED_OBSERVED_COUNT - model::POST_INDEXED_ANCHORED_COUNT - model::BANK_OBSERVED_COUNT - model::BLUE_SCREW_OBSERVED_COUNT - model::SOIL_EVIDENCE_OBSERVED_COUNT - model::SOIL_146_OBSERVED_COUNT - model::FIRAGA_OBSERVED_COUNT - model::FIRAGA_2D2E_OBSERVED_COUNT - model::FIRAGA_EB83_EBC_OBSERVED_COUNT;
                    firaga_code_observed_count += 1;
                    firaga_code_observed |= 1u64 << word;
                } else if word < model::LITERAL_COUNT + model::ADDITIONAL_COUNT + model::OBSERVED_COUNT + model::HOME_ANCHORED_COUNT + model::HOME_OBSERVED_COUNT + model::RED_SPRING_OBSERVED_COUNT + model::RED_SPRING_ANCHORED_COUNT + model::POST_INDEXED_OBSERVED_COUNT + model::POST_INDEXED_ANCHORED_COUNT + model::BANK_OBSERVED_COUNT + model::BLUE_SCREW_OBSERVED_COUNT + model::SOIL_EVIDENCE_OBSERVED_COUNT + model::SOIL_146_OBSERVED_COUNT + model::FIRAGA_OBSERVED_COUNT + model::FIRAGA_2D2E_OBSERVED_COUNT + model::FIRAGA_EB83_EBC_OBSERVED_COUNT + model::FIRAGA_CODE_OBSERVED_COUNT + model::FIRAGA_MIXED_OBSERVED_COUNT {
                    let word = word - model::LITERAL_COUNT - model::ADDITIONAL_COUNT - model::OBSERVED_COUNT - model::HOME_ANCHORED_COUNT - model::HOME_OBSERVED_COUNT - model::RED_SPRING_OBSERVED_COUNT - model::RED_SPRING_ANCHORED_COUNT - model::POST_INDEXED_OBSERVED_COUNT - model::POST_INDEXED_ANCHORED_COUNT - model::BANK_OBSERVED_COUNT - model::BLUE_SCREW_OBSERVED_COUNT - model::SOIL_EVIDENCE_OBSERVED_COUNT - model::SOIL_146_OBSERVED_COUNT - model::FIRAGA_OBSERVED_COUNT - model::FIRAGA_2D2E_OBSERVED_COUNT - model::FIRAGA_EB83_EBC_OBSERVED_COUNT - model::FIRAGA_CODE_OBSERVED_COUNT;
                    firaga_mixed_observed_count += 1;
                    firaga_mixed_observed |= 1u64 << word;
                } else if word < model::WHITE_FLOWERS_BASE {
                    let word = word - model::LITERAL_COUNT - model::ADDITIONAL_COUNT - model::OBSERVED_COUNT - model::HOME_ANCHORED_COUNT - model::HOME_OBSERVED_COUNT - model::RED_SPRING_OBSERVED_COUNT - model::RED_SPRING_ANCHORED_COUNT - model::POST_INDEXED_OBSERVED_COUNT - model::POST_INDEXED_ANCHORED_COUNT - model::BANK_OBSERVED_COUNT - model::BLUE_SCREW_OBSERVED_COUNT - model::SOIL_EVIDENCE_OBSERVED_COUNT - model::SOIL_146_OBSERVED_COUNT - model::FIRAGA_OBSERVED_COUNT - model::FIRAGA_2D2E_OBSERVED_COUNT - model::FIRAGA_EB83_EBC_OBSERVED_COUNT - model::FIRAGA_CODE_OBSERVED_COUNT - model::FIRAGA_MIXED_OBSERVED_COUNT;
                    firaga_residual_observed_count += 1;
                    firaga_residual_observed |= 1u64 << word;
                } else if word < model::WHITE_FLOWERS_ADDITIONAL_BASE {
                    let word = word - model::WHITE_FLOWERS_BASE;
                    white_flowers_observed_count += 1;
                    white_flowers_observed |= 1u64 << word;
                } else if word < model::WHITE_FLOWERS_LATER_BASE {
                    let word = word - model::WHITE_FLOWERS_ADDITIONAL_BASE;
                    white_flowers_additional_count += 1;
                    white_flowers_additional |= 1u64 << word;
                } else if word < model::WHITE_FLOWERS_THIRTEEN_BASE {
                    let word = word - model::WHITE_FLOWERS_LATER_BASE;
                    white_flowers_later_count += 1;
                    white_flowers_later |= 1u64 << word;
                } else if word < model::WHITE_FLOWERS_MANY_BASE {
                    let word = word - model::WHITE_FLOWERS_THIRTEEN_BASE;
                    white_flowers_thirteen_count += 1;
                    white_flowers_thirteen |= 1u64 << word;
                } else if word < model::WHITE_FLOWERS_BULK_BASE {
                    let word = word - model::WHITE_FLOWERS_MANY_BASE;
                    white_flowers_many_count += 1;
                    white_flowers_many[word / 64] |= 1u64 << (word % 64);
                } else if word < model::WHITE_FLOWERS_REMAINING_BASE {
                    let word = word - model::WHITE_FLOWERS_BULK_BASE;
                    white_flowers_bulk_count += 1;
                    white_flowers_bulk[word / 64] |= 1u64 << (word % 64);
                } else if word < model::WHITE_FLOWERS_MIXED_BASE {
                    let word = word - model::WHITE_FLOWERS_REMAINING_BASE;
                    white_flowers_remaining_count += 1;
                    white_flowers_remaining[word / 64] |= 1u64 << (word % 64);
                } else if word < model::WHITE_FLOWERS_FINAL_BASE {
                    let word = word - model::WHITE_FLOWERS_MIXED_BASE;
                    white_flowers_mixed_count += 1;
                    white_flowers_mixed |= 1u64 << word;
                } else if word < model::STAR_CROSS_OBSERVED_BASE {
                    let word = word - model::WHITE_FLOWERS_FINAL_BASE;
                    white_flowers_final_count += 1;
                    white_flowers_final |= 1u64 << word;
                } else {
                    let word = word - model::STAR_CROSS_OBSERVED_BASE;
                    star_cross_observed_count += 1;
                    star_cross_observed |= 1u64 << word;
                }
                byte
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |value| value[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(count, Ordering::Relaxed);
        for (counter, mask) in self.literals.iter().zip(literals) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        self.additional_bytes
            .fetch_add(additional_count, Ordering::Relaxed);
        for (counter, mask) in self.additional.iter().zip(additional) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        self.observed_bytes
            .fetch_add(observed_count, Ordering::Relaxed);
        for (counter, mask) in self.observed.iter().zip(observed) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        self.home_anchored_bytes
            .fetch_add(home_count, Ordering::Relaxed);
        self.home_anchored.fetch_or(home, Ordering::Relaxed);
        self.home_observed_bytes
            .fetch_add(home_observed_count, Ordering::Relaxed);
        self.home_observed
            .fetch_or(home_observed, Ordering::Relaxed);
        self.red_spring_observed_bytes
            .fetch_add(red_spring_observed_count, Ordering::Relaxed);
        for (counter, mask) in self.red_spring_observed.iter().zip(red_spring_observed) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        self.red_spring_anchored_bytes
            .fetch_add(red_spring_anchored_count, Ordering::Relaxed);
        self.red_spring_anchored
            .fetch_or(red_spring_anchored, Ordering::Relaxed);
        self.post_indexed_observed_bytes.fetch_add(post_indexed_observed_count, Ordering::Relaxed);
        self.post_indexed_observed.fetch_or(post_indexed_observed, Ordering::Relaxed);
        self.post_indexed_anchored_bytes.fetch_add(post_indexed_anchored_count, Ordering::Relaxed);
        self.post_indexed_anchored.fetch_or(post_indexed_anchored, Ordering::Relaxed);
        self.bank_observed_bytes.fetch_add(bank_observed_count, Ordering::Relaxed);
        for (counter, mask) in self.bank_observed.iter().zip(bank_observed) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        self.blue_screw_observed_bytes.fetch_add(blue_screw_observed_count, Ordering::Relaxed);
        self.blue_screw_observed.fetch_or(blue_screw_observed, Ordering::Relaxed);
        self.soil_evidence_observed_bytes.fetch_add(soil_evidence_observed_count, Ordering::Relaxed);
        for (counter, mask) in self.soil_evidence_observed.iter().zip(soil_evidence_observed) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        self.soil_146_observed_bytes.fetch_add(soil_146_observed_count, Ordering::Relaxed);
        self.soil_146_observed.fetch_or(soil_146_observed, Ordering::Relaxed);
        self.firaga_observed_bytes.fetch_add(firaga_observed_count, Ordering::Relaxed);
        self.firaga_observed.fetch_or(firaga_observed, Ordering::Relaxed);
        self.firaga_2d2e_observed_bytes.fetch_add(firaga_2d2e_observed_count, Ordering::Relaxed);
        self.firaga_2d2e_observed.fetch_or(firaga_2d2e_observed, Ordering::Relaxed);
        self.firaga_eb83_ebc_observed_bytes.fetch_add(firaga_eb83_ebc_observed_count, Ordering::Relaxed);
        self.firaga_eb83_ebc_observed.fetch_or(firaga_eb83_ebc_observed, Ordering::Relaxed);
        self.firaga_code_observed_bytes.fetch_add(firaga_code_observed_count, Ordering::Relaxed);
        self.firaga_code_observed.fetch_or(firaga_code_observed, Ordering::Relaxed);
        self.firaga_mixed_observed_bytes.fetch_add(firaga_mixed_observed_count, Ordering::Relaxed);
        self.firaga_mixed_observed.fetch_or(firaga_mixed_observed, Ordering::Relaxed);
        self.firaga_residual_observed_bytes.fetch_add(firaga_residual_observed_count, Ordering::Relaxed);
        self.firaga_residual_observed.fetch_or(firaga_residual_observed, Ordering::Relaxed);
        self.white_flowers_observed_bytes.fetch_add(white_flowers_observed_count, Ordering::Relaxed);
        self.white_flowers_observed.fetch_or(white_flowers_observed, Ordering::Relaxed);
        self.white_flowers_additional_bytes.fetch_add(white_flowers_additional_count, Ordering::Relaxed);
        self.white_flowers_additional.fetch_or(white_flowers_additional, Ordering::Relaxed);
        self.white_flowers_later_bytes.fetch_add(white_flowers_later_count, Ordering::Relaxed);
        self.white_flowers_later.fetch_or(white_flowers_later, Ordering::Relaxed);
        self.white_flowers_thirteen_bytes.fetch_add(white_flowers_thirteen_count, Ordering::Relaxed);
        self.white_flowers_thirteen.fetch_or(white_flowers_thirteen, Ordering::Relaxed);
        self.white_flowers_many_bytes.fetch_add(white_flowers_many_count, Ordering::Relaxed);
        for (counter, mask) in self.white_flowers_many.iter().zip(white_flowers_many) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        self.white_flowers_bulk_bytes.fetch_add(white_flowers_bulk_count, Ordering::Relaxed);
        for (counter, mask) in self.white_flowers_bulk.iter().zip(white_flowers_bulk) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        self.white_flowers_remaining_bytes.fetch_add(white_flowers_remaining_count, Ordering::Relaxed);
        for (counter, mask) in self.white_flowers_remaining.iter().zip(white_flowers_remaining) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        self.white_flowers_mixed_bytes.fetch_add(white_flowers_mixed_count, Ordering::Relaxed);
        self.white_flowers_mixed.fetch_or(white_flowers_mixed, Ordering::Relaxed);
        self.white_flowers_final_bytes.fetch_add(white_flowers_final_count, Ordering::Relaxed);
        self.white_flowers_final.fetch_or(white_flowers_final, Ordering::Relaxed);
        self.star_cross_observed_bytes.fetch_add(star_cross_observed_count, Ordering::Relaxed);
        self.star_cross_observed.fetch_or(star_cross_observed, Ordering::Relaxed);
        Some(result)
    }

    pub(super) fn stats(&self) -> ThumbPcLiteralsStats {
        ThumbPcLiteralsStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.bytes.load(Ordering::Relaxed),
            literals_mask: self.literals.each_ref().map(|v| v.load(Ordering::Relaxed)),
            additional_source_bytes: self.additional_bytes.load(Ordering::Relaxed),
            additional_mask: self
                .additional
                .each_ref()
                .map(|v| v.load(Ordering::Relaxed)),
            observed_source_bytes: self.observed_bytes.load(Ordering::Relaxed),
            observed_mask: self.observed.each_ref().map(|v| v.load(Ordering::Relaxed)),
            home_anchored_bytes: self.home_anchored_bytes.load(Ordering::Relaxed),
            home_anchored_mask: self.home_anchored.load(Ordering::Relaxed),
            home_observed_bytes: self.home_observed_bytes.load(Ordering::Relaxed),
            home_observed_mask: self.home_observed.load(Ordering::Relaxed),
            red_spring_observed_bytes: self.red_spring_observed_bytes.load(Ordering::Relaxed),
            red_spring_observed_mask: self
                .red_spring_observed
                .each_ref()
                .map(|v| v.load(Ordering::Relaxed)),
            red_spring_anchored_bytes: self.red_spring_anchored_bytes.load(Ordering::Relaxed),
            red_spring_anchored_mask: self.red_spring_anchored.load(Ordering::Relaxed),
            post_indexed_observed_bytes: self.post_indexed_observed_bytes.load(Ordering::Relaxed),
            post_indexed_observed_mask: self.post_indexed_observed.load(Ordering::Relaxed),
            post_indexed_anchored_bytes: self.post_indexed_anchored_bytes.load(Ordering::Relaxed),
            post_indexed_anchored_mask: self.post_indexed_anchored.load(Ordering::Relaxed),
            bank_observed_bytes: self.bank_observed_bytes.load(Ordering::Relaxed),
            bank_observed_mask: self.bank_observed.each_ref().map(|v| v.load(Ordering::Relaxed)),
            blue_screw_observed_bytes: self.blue_screw_observed_bytes.load(Ordering::Relaxed),
            blue_screw_observed_mask: self.blue_screw_observed.load(Ordering::Relaxed),
            soil_evidence_observed_bytes: self.soil_evidence_observed_bytes.load(Ordering::Relaxed),
            soil_evidence_observed_mask: self.soil_evidence_observed.each_ref().map(|v| v.load(Ordering::Relaxed)),
            soil_146_observed_bytes: self.soil_146_observed_bytes.load(Ordering::Relaxed),
            soil_146_observed_mask: self.soil_146_observed.load(Ordering::Relaxed),
            firaga_observed_bytes: self.firaga_observed_bytes.load(Ordering::Relaxed),
            firaga_observed_mask: self.firaga_observed.load(Ordering::Relaxed),
            firaga_2d2e_observed_bytes: self.firaga_2d2e_observed_bytes.load(Ordering::Relaxed),
            firaga_2d2e_observed_mask: self.firaga_2d2e_observed.load(Ordering::Relaxed),
            firaga_eb83_ebc_observed_bytes: self.firaga_eb83_ebc_observed_bytes.load(Ordering::Relaxed),
            firaga_eb83_ebc_observed_mask: self.firaga_eb83_ebc_observed.load(Ordering::Relaxed),
            firaga_code_observed_bytes: self.firaga_code_observed_bytes.load(Ordering::Relaxed),
            firaga_code_observed_mask: self.firaga_code_observed.load(Ordering::Relaxed),
            firaga_mixed_observed_bytes: self.firaga_mixed_observed_bytes.load(Ordering::Relaxed),
            firaga_mixed_observed_mask: self.firaga_mixed_observed.load(Ordering::Relaxed),
            firaga_residual_observed_bytes: self.firaga_residual_observed_bytes.load(Ordering::Relaxed),
            firaga_residual_observed_mask: self.firaga_residual_observed.load(Ordering::Relaxed),
            white_flowers_observed_bytes: self.white_flowers_observed_bytes.load(Ordering::Relaxed),
            white_flowers_observed_mask: self.white_flowers_observed.load(Ordering::Relaxed),
            white_flowers_additional_bytes: self.white_flowers_additional_bytes.load(Ordering::Relaxed),
            white_flowers_additional_mask: self.white_flowers_additional.load(Ordering::Relaxed),
            white_flowers_later_bytes: self.white_flowers_later_bytes.load(Ordering::Relaxed),
            white_flowers_later_mask: self.white_flowers_later.load(Ordering::Relaxed),
            white_flowers_thirteen_bytes: self.white_flowers_thirteen_bytes.load(Ordering::Relaxed),
            white_flowers_thirteen_mask: self.white_flowers_thirteen.load(Ordering::Relaxed),
            white_flowers_many_bytes: self.white_flowers_many_bytes.load(Ordering::Relaxed),
            white_flowers_many_mask: std::array::from_fn(|i| self.white_flowers_many[i].load(Ordering::Relaxed)),
            white_flowers_bulk_bytes: self.white_flowers_bulk_bytes.load(Ordering::Relaxed),
            white_flowers_bulk_mask: std::array::from_fn(|i| self.white_flowers_bulk[i].load(Ordering::Relaxed)),
            white_flowers_remaining_bytes: self.white_flowers_remaining_bytes.load(Ordering::Relaxed),
            white_flowers_remaining_mask: std::array::from_fn(|i| self.white_flowers_remaining[i].load(Ordering::Relaxed)),
            white_flowers_mixed_bytes: self.white_flowers_mixed_bytes.load(Ordering::Relaxed),
            white_flowers_mixed_mask: self.white_flowers_mixed.load(Ordering::Relaxed),
            white_flowers_final_bytes: self.white_flowers_final_bytes.load(Ordering::Relaxed),
            white_flowers_final_mask: self.white_flowers_final.load(Ordering::Relaxed),
            star_cross_observed_bytes: self.star_cross_observed_bytes.load(Ordering::Relaxed),
            star_cross_observed_mask: self.star_cross_observed.load(Ordering::Relaxed),
        }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.literals.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.additional.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.observed.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.home_anchored.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.home_observed.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.red_spring_observed.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.red_spring_anchored.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.post_indexed_observed.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.post_indexed_anchored.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.bank_observed.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.blue_screw_observed.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.soil_evidence_observed.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.soil_146_observed.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.firaga_observed.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.firaga_2d2e_observed.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.firaga_eb83_ebc_observed.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.firaga_code_observed.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.firaga_mixed_observed.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.firaga_residual_observed.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.white_flowers_observed.as_ref().map_or(0, |words| words.capacity() * std::mem::size_of::<model::LiteralWord>())
            + self.data.white_flowers_additional.as_ref().map_or(0, |words| words.capacity() * std::mem::size_of::<model::LiteralWord>())
            + self.data.white_flowers_later.as_ref().map_or(0, |words| words.capacity() * std::mem::size_of::<model::LiteralWord>())
            + self.data.white_flowers_thirteen.as_ref().map_or(0, |words| words.capacity() * std::mem::size_of::<model::LiteralWord>())
            + self.data.white_flowers_many.as_ref().map_or(0, |words| words.capacity() * std::mem::size_of::<model::LiteralWord>())
            + self.data.white_flowers_bulk.as_ref().map_or(0, |words| words.capacity() * std::mem::size_of::<model::LiteralWord>())
            + self.data.white_flowers_remaining.as_ref().map_or(0, |words| words.capacity() * std::mem::size_of::<model::LiteralWord>())
            + self.data.white_flowers_mixed.as_ref().map_or(0, |words| words.capacity() * std::mem::size_of::<model::LiteralWord>())
            + self.data.white_flowers_final.as_ref().map_or(0, |words| words.capacity() * std::mem::size_of::<model::LiteralWord>())
            + self.data.star_cross_observed.as_ref().map_or(0, |words| words.capacity() * std::mem::size_of::<model::LiteralWord>())
    }
}
