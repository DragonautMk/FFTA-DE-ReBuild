//! Typed score/threshold bytes; original live producers, arithmetic and RNG remain.
use super::*;
use crate::rom_data::evaluation_parameters as model;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EvaluationParametersStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub divisor_bytes: u64,
    pub weight_bytes: u64,
    pub bonus_bytes: u64,
    pub score_opaque_bytes: u64,
    pub threshold_bytes: u64,
    pub threshold_opaque_bytes: u64,
    pub threshold_mask: u8,
}
pub(super) struct EvaluationParametersResources {
    data: model::EvaluationParameters,
    counters: [AtomicU64; 8],
    threshold_mask: AtomicU64,
}
impl EvaluationParametersResources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x6e7
    }
    pub(super) fn new(
        data: model::EvaluationParameters,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        data.validate()?;
        if sources.len() != 2 {
            return Err("Evaluation parameters require both source bindings".into());
        }
        let score = data.score_bytes();
        let thresholds = data.threshold_bytes();
        for (source, (offset, bytes, role)) in sources.iter().zip([
            (
                model::SCORE_OFFSET,
                score.as_slice(),
                "evaluation-parameters/score",
            ),
            (
                model::THRESHOLD_SOURCE,
                thresholds.as_slice(),
                "evaluation-parameters/thresholds",
            ),
        ]) {
            if source.offset != offset
                || source.length as usize != bytes.len()
                || source.role != role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("Evaluation parameter source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            threshold_mask: AtomicU64::new(0),
        })
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        if ![
            (model::SCORE_OFFSET, model::SCORE_END),
            (model::THRESHOLD_SOURCE, model::END_OFFSET),
        ]
        .iter()
        .any(|&(a, z)| index < z as usize && index.saturating_add(N) > a as usize)
        {
            return prior;
        }
        let mut counts = [0u64; 8];
        let mut mask = 0;
        let bytes = std::array::from_fn(|i| {
            let at = index + i;
            let owned = if (model::SCORE_OFFSET as usize..model::SCORE_END as usize).contains(&at) {
                let offset = at - model::SCORE_OFFSET as usize;
                let record = offset / 2;
                if record < 7 {
                    let term = &self.data.terms[record];
                    Some(if offset % 2 == 0 {
                        (term.divisor, 2)
                    } else {
                        (term.weight, 3)
                    })
                } else {
                    let bonus = &self.data.bonuses[record - 7];
                    Some(if offset % 2 == 0 {
                        (bonus.byte00, 5)
                    } else {
                        (bonus.weight, 4)
                    })
                }
            } else if (model::THRESHOLD_SOURCE as usize..model::END_OFFSET as usize).contains(&at) {
                let offset = at - model::THRESHOLD_SOURCE as usize;
                let record = offset / 4;
                let field = offset % 4;
                mask |= 1 << record;
                let threshold = &self.data.thresholds[record];
                Some(if field == 0 {
                    (threshold.threshold, 6)
                } else {
                    (threshold.bytes01_03[field - 1], 7)
                })
            } else {
                None
            };
            if let Some((value, field)) = owned {
                counts[1] += 1;
                counts[field] += 1;
                value
            } else {
                prior.as_ref().map_or_else(|| rom[at], |v| v[i])
            }
        });
        counts[0] = 1;
        for (counter, count) in self.counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        self.threshold_mask.fetch_or(mask, Ordering::Relaxed);
        Some(bytes)
    }
    pub(super) fn stats(&self) -> EvaluationParametersStats {
        let [bus_reads, source_bytes, divisor_bytes, weight_bytes, bonus_bytes, score_opaque_bytes, threshold_bytes, threshold_opaque_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        EvaluationParametersStats {
            bus_reads,
            source_bytes,
            divisor_bytes,
            weight_bytes,
            bonus_bytes,
            score_opaque_bytes,
            threshold_bytes,
            threshold_opaque_bytes,
            threshold_mask: self.threshold_mask.load(Ordering::Relaxed) as u8,
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
