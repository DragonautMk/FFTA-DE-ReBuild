//! Original live score prefix, exact arithmetic fragments and RNG threshold prefix.
use super::*;
use gba_core::{
    rom_data::evaluation_parameters::{
        Catalog, EvaluationParameters, END_OFFSET, SCORE_END, SCORE_OFFSET, THRESHOLD_SOURCE,
    },
    Bus, Machine,
};
use std::collections::BTreeSet;
pub(super) const STACK: u32 = 0x03007e00;
const UNIT: u32 = 0x02024000;
const CONTEXT: u32 = 0x02025000;
const RNG: u32 = 0x030034b0;
const LOADS: [u32; 7] = [
    0x080d2db4, 0x080d2dc8, 0x080d2dd8, 0x080d2df0, 0x080d2e08, 0x080d2e20, 0x080d2e38,
];
const ENDS: [u32; 7] = [
    0x080d2dc4, 0x080d2dd4, 0x080d2de4, 0x080d2dfc, 0x080d2e14, 0x080d2e2c, 0x080d2e44,
];
#[derive(Clone, Copy, Debug)]
pub(super) enum Case {
    Score {
        job: u8,
        level: u8,
        hp: u16,
        mp: u16,
        stats: [u16; 4],
    },
    Term {
        index: usize,
        input: u32,
        acc: u32,
    },
    Bonus {
        index: usize,
        predicate: u32,
        acc: u32,
    },
    Threshold {
        class: u32,
        seed: u32,
    },
    ThresholdSuffix {
        class: u32,
        draw: u32,
    },
}
impl Case {
    pub(super) fn stop(self) -> u32 {
        match self {
            Self::Score { .. } => 0x080d2e6a,
            Self::Term { index, .. } => ENDS[index],
            Self::Bonus { index, .. } => {
                if index == 0 {
                    0x080d2e58
                } else {
                    0x080d2e6a
                }
            }
            _ => 0x080d486c,
        }
    }
}
fn seed_for(draw: u32) -> u32 {
    draw.wrapping_shl(16)
        .wrapping_sub(0x3039)
        .wrapping_mul(0xeeb9eb65)
}
pub(super) fn cases(data: &EvaluationParameters) -> Vec<Case> {
    let mut out = vec![];
    for job in [1, 2, 3, 8, 20, 43] {
        for high in [false, true] {
            out.push(Case::Score {
                job,
                level: if high { 255 } else { 12 },
                hp: if high { 65535 } else { 120 },
                mp: if high { 32768 } else { 40 },
                stats: if high {
                    [0x8000, 0x7fff, 0xffff, 0x1234]
                } else {
                    [120, 90, 70, 60]
                },
            });
        }
    }
    for index in 0..7 {
        for input in [0, 1, 255, 65535, 0x7fffffff, 0x80000000, 0xffffffff] {
            out.push(Case::Term {
                index,
                input,
                acc: 0xfffffff0,
            });
        }
    }
    for index in 0..2 {
        for predicate in [0, 1, 2, 0xff, 0x100, 0x101] {
            out.push(Case::Bonus {
                index,
                predicate,
                acc: 0xfffffffe,
            });
        }
    }
    for t in &data.thresholds {
        let mut draws: BTreeSet<u32> = [
            0,
            1,
            254,
            255,
            256,
            32767,
            u32::from(t.threshold),
            u32::from(t.threshold).saturating_sub(1),
        ]
        .into_iter()
        .collect();
        for draw in std::mem::take(&mut draws) {
            out.push(Case::Threshold {
                class: t.id as u32,
                seed: seed_for(draw),
            });
        }
        for draw in [0x8000, 0xffff, 0xffff0000, 0x1234ffff] {
            out.push(Case::ThresholdSuffix {
                class: t.id as u32,
                draw,
            });
        }
    }
    out.push(Case::Threshold {
        class: 0,
        seed: 0xdeadbeef,
    });
    out
}
pub(super) fn setup(m: &mut Machine, c: Case) {
    for i in 0..13 {
        m.cpu.regs[i] = 0x11220000 + i as u32;
    }
    m.bus.ewram[0x23ff0..0x24120].fill(0x3c);
    m.bus.ewram[0x24ff0..0x25070].fill(0x3c);
    m.bus.ewram[0x24000..0x24108].fill(0);
    m.bus.iwram[0x7c00..0x7e20].fill(0x5a);
    m.bus.write32(RNG, 0xaabbccdd);
    m.bus.reg_ie = 0;
    m.bus.ime = false;
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.cpsr = 0xbf;
    m.cpu.regs[15] = match c {
        Case::Score {
            job,
            level,
            hp,
            mp,
            stats,
        } => {
            m.bus.write16(UNIT, 1);
            m.bus.write8(UNIT + 4, 1);
            m.bus.write8(UNIT + 5, job);
            m.bus.write8(UNIT + 7, job);
            m.bus.write8(UNIT + 9, level);
            m.bus.write16(UNIT + 0x1a, hp);
            m.bus.write16(UNIT + 0x1e, mp);
            for (i, v) in stats.into_iter().enumerate() {
                m.bus.write16(UNIT + 0x20 + (i * 2) as u32, v);
            }
            m.cpu.regs[6] = UNIT;
            0x080d2db0
        }
        Case::Term { index, input, acc } => {
            m.cpu.regs[4] = 0x08000000 + SCORE_OFFSET + (index * 2) as u32;
            m.cpu.regs[0] = input;
            m.cpu.regs[5] = acc;
            LOADS[index]
        }
        Case::Bonus {
            index,
            predicate,
            acc,
        } => {
            m.cpu.regs[4] = 0x08000000 + SCORE_OFFSET + 14 + (index * 2) as u32;
            m.cpu.regs[0] = predicate;
            m.cpu.regs[5] = acc;
            if index == 0 {
                0x080d2e4c
            } else {
                0x080d2e60
            }
        }
        Case::Threshold { class, seed } => {
            m.cpu.regs[4] = class;
            m.cpu.regs[9] = CONTEXT;
            m.bus.write32(RNG, seed);
            0x080d4840
        }
        Case::ThresholdSuffix { class, draw } => {
            m.cpu.regs[4] = class;
            m.cpu.regs[5] = 0x08000000 + data_threshold_offset(class);
            m.cpu.regs[9] = CONTEXT;
            m.cpu.regs[0] = draw;
            0x080d484e
        }
    };
}
fn data_threshold_offset(class: u32) -> u32 {
    gba_core::rom_data::evaluation_parameters::THRESHOLD_BIAS + class * 4
}
pub(super) fn run(
    m: &mut Machine,
    c: Case,
    data: &EvaluationParameters,
    visited: &mut BTreeSet<(u32, bool)>,
) -> usize {
    setup(m, c);
    let ram = m.bus.ewram[0x23ff0..0x24120].to_vec();
    let context = m.bus.ewram[0x24ff0..0x25070].to_vec();
    let preserved = m.cpu.regs;
    let mut inputs = [0u32; 7];
    let mut predicates = [u32::MAX; 2];
    let mut rng_draw = None;
    let mut threshold_loads = 0;
    let mut score_reads = 0;
    let mut n = 0;
    while m.cpu.regs[15] != c.stop() {
        assert!(n < 30000, "evaluation {c:?} at{:08x}", m.cpu.regs[15]);
        let pc = m.cpu.regs[15];
        visited.insert((pc, m.cpu.thumb()));
        if let Some(i) = LOADS.iter().position(|&at| at == pc) {
            assert_eq!(m.cpu.regs[4], 0x08000000 + SCORE_OFFSET + (i * 2) as u32);
            inputs[i] = m.cpu.regs[0];
            score_reads += 2;
        }
        if pc == 0x080d2e4c {
            predicates[0] = m.cpu.regs[0];
        }
        if pc == 0x080d2e60 {
            predicates[1] = m.cpu.regs[0];
        }
        if pc == 0x080d2e54 {
            assert_eq!(m.cpu.regs[4], 0x08000000 + SCORE_OFFSET + 14);
            score_reads += 1;
        }
        if pc == 0x080d2e66 {
            assert_eq!(m.cpu.regs[4], 0x08000000 + SCORE_OFFSET + 16);
            score_reads += 1;
        }
        if pc == 0x080d484e {
            rng_draw = Some(m.cpu.regs[0]);
        }
        if pc == 0x080d4864 {
            let class = match c {
                Case::Threshold { class, .. } | Case::ThresholdSuffix { class, .. } => class,
                _ => unreachable!(),
            };
            assert_eq!(
                m.cpu.regs[5],
                0x08000000 + data.threshold(class).unwrap().source_offset
            );
            threshold_loads += 1;
        }
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    match c {
        Case::Score { .. } => {
            assert_eq!(
                m.cpu.regs[5],
                data.score(inputs, predicates).unwrap(),
                "live score {c:?}, inputs={inputs:?}, predicates={predicates:?}"
            );
            assert!(score_reads >= 14);
            assert!(predicates.iter().all(|&v| v != u32::MAX));
            assert_eq!(m.cpu.regs[4], 0x08000000 + SCORE_OFFSET + 16);
        }
        Case::Term { index, input, acc } => {
            let product = data.term_product(index, input).unwrap();
            assert_eq!(
                m.cpu.regs[5],
                if index == 0 {
                    product
                } else {
                    acc.wrapping_add(product)
                },
                "term {c:?}"
            );
            assert_eq!(score_reads, 2);
        }
        Case::Bonus {
            index,
            predicate,
            acc,
        } => {
            let take = predicate as u8 == if index == 0 { 1 } else { 0 };
            assert_eq!(
                m.cpu.regs[5],
                acc.wrapping_add(if take {
                    data.bonuses[index].weight as u32
                } else {
                    0
                })
            );
            assert_eq!(score_reads, u32::from(take));
        }
        Case::Threshold { class, seed } => {
            if class == 0 {
                assert_eq!(threshold_loads, 0);
                assert_eq!(rng_draw, None);
                assert_eq!(m.bus.read32(RNG), seed);
                assert_eq!(m.cpu.regs[4], 0);
            } else {
                let next = seed.wrapping_mul(0x41c64e6d).wrapping_add(0x3039);
                let draw = (next >> 16) & 0x7fff;
                assert_eq!(rng_draw, Some(draw));
                assert_eq!(m.bus.read32(RNG), next);
                assert_eq!(threshold_loads, 1);
                assert_eq!(
                    m.cpu.regs[4],
                    if data.threshold(class).unwrap().accepts_random(draw) {
                        class
                    } else {
                        0
                    }
                );
            }
        }
        Case::ThresholdSuffix { class, draw } => {
            assert_eq!(threshold_loads, 1);
            assert_eq!(rng_draw, Some(draw));
            assert_eq!(
                m.cpu.regs[4],
                if data.threshold(class).unwrap().accepts_random(draw) {
                    class
                } else {
                    0
                }
            );
        }
    }
    if !matches!(c, Case::Threshold { .. }) {
        assert_eq!(m.bus.read32(RNG), 0xaabbccdd);
    }
    assert_eq!(m.cpu.regs[13], STACK);
    for i in 6..12 {
        assert_eq!(m.cpu.regs[i], preserved[i], "preservedr{i} {c:?}");
    }
    assert_eq!(&m.bus.ewram[0x23ff0..0x24120], &ram);
    assert_eq!(&m.bus.ewram[0x24ff0..0x25070], &context);
    assert!(m.bus.iwram[0x7c00..0x7d80].iter().all(|&v| v == 0x5a));
    assert!(m.bus.iwram[0x7e00..0x7e20].iter().all(|&v| v == 0x5a));
    n
}
fn source(m: &mut Machine, data: &EvaluationParameters) {
    m.bus.rom[SCORE_OFFSET as usize..SCORE_END as usize].copy_from_slice(&data.score_bytes());
    m.bus.rom[THRESHOLD_SOURCE as usize..END_OFFSET as usize]
        .copy_from_slice(&data.threshold_bytes());
}
#[test]
#[ignore = "private EUR38B score/threshold source proof; one reduced Wine oracle, Linux/USA pending"]
fn european_evaluation_parameters_match_original_live_score_arithmetic_and_rng_thresholds() {
    let root = PathBuf::from(
        std::env::var_os("FFTA_STATE_TEST_EVALUATION_PARAMETERS_ORIGINAL_ROOT").unwrap(),
    );
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let independent: EvaluationParameters = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_EVALUATION_PARAMETERS_RECORDS").unwrap())
            .unwrap(),
    )
    .unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    assert_eq!(data, &independent);
    data.validate().unwrap();
    data.verify_source(&rom).unwrap();
    assert_eq!(
        (
            catalog.sources.len(),
            catalog.sources[0].length,
            catalog.sources[1].length
        ),
        (2, 18, 20)
    );
    assert_eq!(
        &data.score_bytes(),
        &rom[SCORE_OFFSET as usize..SCORE_END as usize]
    );
    assert_eq!(
        &data.threshold_bytes(),
        &rom[THRESHOLD_SOURCE as usize..END_OFFSET as usize]
    );
    let bytes = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        &serde_json::from_slice::<EvaluationParameters>(&bytes).unwrap(),
        data
    );
    let shape = serde_json::to_value(data).unwrap();
    for c in 0..6 {
        let mut v = shape.clone();
        match c {
            0 => v["extra"] = true.into(),
            1 => v["terms"][0]["extra"] = true.into(),
            2 => {
                v["terms"].as_array_mut().unwrap().pop();
            }
            3 => v["thresholds"][0]["bytes01_03"] = serde_json::json!([0, 0]),
            4 => {
                v["thresholds"].as_array_mut().unwrap().pop();
            }
            _ => {
                v.as_object_mut().unwrap().remove("bonuses");
            }
        }
        assert!(serde_json::from_value::<EvaluationParameters>(v).is_err());
    }
    for c in 0..7 {
        let mut v = data.clone();
        match c {
            0 => v.score_offset += 1,
            1 => v.threshold_address_bias += 4,
            2 => v.terms[2].id = 0,
            3 => v.bonuses[0].source_offset += 1,
            4 => v.thresholds[0].id = 0,
            5 => v.thresholds.swap(0, 4),
            _ => v.thresholds[4].bytes01_03[2] ^= 1,
        }
        if c < 6 {
            assert!(v.validate().is_err());
        } else {
            v.validate().unwrap();
            assert!(v.verify_source(&rom).is_err());
        }
    }
    assert!(data.threshold(0).is_none());
    assert!(data.threshold(6).is_none());
    assert!(data.threshold(0x10000001).is_none());
    let mut zero = data.clone();
    zero.terms[0].divisor = 0;
    zero.validate().unwrap();
    assert_eq!(zero.term_product(0, 123), None);
    assert!(zero.verify_source(&rom).is_err());
    let mut m = crate::make_machine(rom.clone(), None);
    let real = cases(data);
    let mut visited = BTreeSet::new();
    let mut instructions = 0;
    for &c in &real {
        instructions += run(&mut m, c, data, &mut visited);
    }
    let mut modified = data.clone();
    for (i, t) in modified.terms.iter_mut().enumerate() {
        t.divisor = if i % 2 == 0 { 1 } else { 255 };
        t.weight = 255;
    }
    for b in &mut modified.bonuses {
        b.byte00 = 0xff;
        b.weight = 255;
    }
    for (i, t) in modified.thresholds.iter_mut().enumerate() {
        t.threshold = if i % 2 == 0 { 0 } else { 255 };
        t.bytes01_03 = [0xff, 0x80, 0x55];
    }
    modified.validate().unwrap();
    assert!(modified.verify_source(&rom).is_err());
    source(&mut m, &modified);
    assert!(Catalog::from_rom(&m.bus.rom).is_err());
    let mut synthetic = 0;
    for index in 0..7 {
        for input in [0xffffffff, 0x80000000, 0x12345678] {
            instructions += run(
                &mut m,
                Case::Term {
                    index,
                    input,
                    acc: 0xfffffffd,
                },
                &modified,
                &mut visited,
            );
            synthetic += 1;
        }
    }
    for index in 0..2 {
        for predicate in [0x100, 0x101, 0xff] {
            instructions += run(
                &mut m,
                Case::Bonus {
                    index,
                    predicate,
                    acc: 0xfffffffe,
                },
                &modified,
                &mut visited,
            );
            synthetic += 1;
        }
    }
    for class in 1..=5 {
        for draw in [0, 254, 255, 0x8000, 0xffff] {
            instructions += run(
                &mut m,
                Case::ThresholdSuffix { class, draw },
                &modified,
                &mut visited,
            );
            synthetic += 1;
        }
    }
    source(&mut m, data);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("evaluation-parameters.json"), &bytes).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"source_bytes":38,"source_ranges":2,"terms":7,"bonuses":2,"thresholds":5,"real_cases":real.len(),"live_score_prefixes":real.iter().filter(|c|matches!(c,Case::Score{..})).count(),"synthetic_cases":synthetic,"instructions_coverage":instructions,"pcs":visited.len(),"original_live_getters_unsigned_first_u8_then_signed_six_wrapping_arithmetic":true,"conditional_predicate_low8_equal1_equal0":true,"original_lcg_state_and_class0_no_rng_no_source":true,"signed_low16_mod256_threshold_prefix_negative_synthetic_only":true,"fixed_json_shape_ids_order_offsets_opaque_full_roundtrip":true,"stack_register_ram_context_guards":true,"zero_divisor":"diagnosticNone only; original guest unchanged, not executed here","unowned_score_gap_and_class0_bytes":18,"whole_function_and_semantic_names":"unclaimed","synthetic":"unauthenticated/unpublished","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
