//! Original language switches, seven new producers and shared split-copy tails.
use super::*;
use gba_core::{
    rom_data::{
        obj_graphics,
        ui_graphics::{Catalog, Graphics, LOCALIZED_BANK_ROOTS, SELECTOR_ROOTS},
    },
    Bus, Machine,
};
use std::collections::BTreeSet;
pub(super) const SCRATCH: u32 = 0x02020010;
const STACK: u32 = 0x03007c00;
#[derive(Clone, Copy, Debug)]
pub(super) enum Case {
    Selector { group: usize, language: u8 },
    SharedTail { language: u8 },
}
pub(super) fn cases() -> Vec<Case> {
    (0..2)
        .flat_map(|group| {
            [0, 1, 2, 3, 4, 5, 255].map(|language| Case::Selector { group, language })
        })
        .chain([
            Case::SharedTail { language: 3 },
            Case::SharedTail { language: 4 },
        ])
        .collect()
}
pub(super) fn shared() -> obj_graphics::Graphics {
    serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_SHARED_OBJ_GRAPHICS").unwrap()).unwrap(),
    )
    .unwrap()
}
fn full(group: usize, language: u8) -> bool {
    (group == 0 && (1..=4).contains(&language)) || (group == 1 && language < 3)
}
pub(super) fn stop(case: Case) -> u32 {
    match case {
        Case::SharedTail { language } => {
            if language == 4 {
                0x08143dd6
            } else {
                0x08143d66
            }
        }
        Case::Selector { group: 0, language } => {
            if full(0, language) {
                0x0808edac
            } else {
                0x0808ed9c
            }
        }
        Case::Selector { group: 1, language } => {
            if language > 4 {
                0x08143dfe
            } else if full(1, language) {
                0x08143d66
            } else if language == 4 {
                0x08143db4
            } else {
                0x08143d44
            }
        }
        _ => unreachable!(),
    }
}
pub(super) fn setup(m: &mut Machine, case: Case, obj: &obj_graphics::Graphics) {
    m.cpu.regs.fill(0);
    m.cpu.regs[7] = SCRATCH;
    m.cpu.regs[10] = SCRATCH;
    m.cpu.regs[13] = STACK;
    m.cpu.cpsr = 0xbf;
    for i in 0..32800 {
        m.bus.write8(SCRATCH - 16 + i, 0x5a);
    }
    m.bus.vram.fill(0x5a);
    match case {
        Case::Selector { group, language } => {
            m.bus.write8(0x02001f6e, language);
            m.cpu.regs[15] = 0x08000000 + SELECTOR_ROOTS[group].entry;
        }
        Case::SharedTail { language } => {
            let at = if language == 4 { 0xb80140 } else { 0xb7f874 };
            let bank = obj.banks.iter().find(|b| b.source_offset == at).unwrap();
            for (i, v) in bank.decoded_bytes().into_iter().enumerate() {
                m.bus.write8(SCRATCH + i as u32, v);
            }
            m.cpu.regs[..3].copy_from_slice(&[SCRATCH + 5056, 0x06000000, 0x540]);
            m.cpu.regs[15] = if language == 4 {
                0x08143dd2
            } else {
                0x08143d62
            };
        }
    }
}
pub(super) fn run(
    m: &mut Machine,
    case: Case,
    graphics: &Graphics,
    obj: &obj_graphics::Graphics,
    visited: &mut BTreeSet<(u32, bool)>,
) -> usize {
    setup(m, case, obj);
    let end = stop(case);
    let mut n = 0;
    let selected = match case {
        Case::Selector { group, language } => graphics.selectors[group].select(language),
        _ => None,
    };
    let mut terminal = false;
    let mut saw_selector = false;
    let mut uploads = 0;
    while m.cpu.regs[15] != end {
        assert!(
            n < 2_000_000,
            "localized custom UI path {case:?} at{:08x}",
            m.cpu.regs[15]
        );
        if let Case::Selector { group, language } = case {
            let call = if group == 0 {
                0x0808ed9c
            } else if language == 4 {
                0x08143db4
            } else {
                0x08143d44
            };
            if m.cpu.regs[15] == call {
                assert_eq!(m.cpu.regs[0], SCRATCH);
                assert_eq!(m.cpu.regs[1], 0x08000000 + selected.unwrap().bank_offset);
                saw_selector = true;
            }
            if full(group, language) {
                let p = selected.unwrap();
                let b = graphics
                    .banks
                    .iter()
                    .find(|b| b.source_offset == p.bank_offset)
                    .unwrap();
                if m.cpu.regs[15] == 0x080058de {
                    assert_eq!(
                        m.cpu.regs[7],
                        0x08000000 + b.source_offset + b.encoded_source.len() as u32
                    );
                    assert_eq!(m.cpu.regs[9], b.words.len() as u32 * 4);
                    terminal = true;
                }
                if m.cpu.regs[15] == call + 4 {
                    assert_eq!(m.cpu.regs[0], b.words.len() as u32 * 4);
                }
                if group == 0 && m.cpu.regs[15] == 0x0808eda8 {
                    assert_eq!(&m.cpu.regs[..3], &[SCRATCH, 0x0600ad40, 0x110]);
                    uploads += 1;
                }
                if group == 1 && m.cpu.regs[15] == 0x08143d50 {
                    assert_eq!(&m.cpu.regs[..3], &[SCRATCH, 0x06016440, 0x4f0]);
                    uploads += 1;
                }
                if group == 1 && m.cpu.regs[15] == 0x08143d62 {
                    assert_eq!(&m.cpu.regs[..3], &[SCRATCH + 5056, 0x06000000, 0x540]);
                    uploads += 1;
                }
            }
        }
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    let mut expected = vec![0x5a; 0x18000];
    match case {
        Case::Selector { group, language } => {
            if selected.is_some() && !full(group, language) {
                assert_eq!(m.cpu.regs[0], SCRATCH);
                assert_eq!(m.cpu.regs[1], 0x08000000 + selected.unwrap().bank_offset);
            }
            if full(group, language) {
                assert!(saw_selector && terminal);
                assert_eq!(uploads, if group == 0 { 1 } else { 2 });
                let b = graphics
                    .banks
                    .iter()
                    .find(|b| b.source_offset == selected.unwrap().bank_offset)
                    .unwrap();
                let decoded = b.decoded_bytes();
                assert!((0..decoded.len()).all(|i| m.bus.read8(SCRATCH + i as u32) == decoded[i]));
                assert!((0..16).all(|i| m.bus.read8(SCRATCH - 16 + i) == 0x5a));
                assert!((decoded.len()..32784).all(|i| m.bus.read8(SCRATCH + i as u32) == 0x5a));
                if group == 0 {
                    expected[0xad40..0xad40 + 544].copy_from_slice(&decoded[..544]);
                } else {
                    expected[0x16440..0x16440 + 5056].copy_from_slice(&decoded[..5056]);
                    expected[..2688].copy_from_slice(&decoded[5056..]);
                }
            }
        }
        Case::SharedTail { language } => {
            let at = if language == 4 { 0xb80140 } else { 0xb7f874 };
            let decoded = obj
                .banks
                .iter()
                .find(|b| b.source_offset == at)
                .unwrap()
                .decoded_bytes();
            expected[..2688].copy_from_slice(&decoded[5056..]);
        }
    }
    assert_eq!(m.bus.vram.as_slice(), expected);
    n
}
#[test]
#[ignore = "EUR two language switches/seven new graphics/shared split tails; Wine only, Linux/USA pending"]
fn european_localized_ui_graphics_match_original_selectors_producers_and_split_copies() {
    let root =
        PathBuf::from(std::env::var_os("FFTA_STATE_TEST_UI_LOCALIZED_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let graphics = &catalog.decoded;
    let obj = shared();
    graphics.validate_shared_obj(&obj).unwrap();
    let old: Graphics = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_PRIOR_UI_GRAPHICS").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        (
            old.banks.len(),
            graphics.banks.len(),
            graphics.selectors.len(),
            catalog.sources.len()
        ),
        (9, 16, 2, 26)
    );
    assert!(old.banks.iter().all(|b| graphics.banks.contains(b)));
    assert_eq!(catalog.sources.iter().map(|s| s.length).sum::<u32>(), 71474);
    let inv: serde_json::Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_UI_LOCALIZED_INVENTORY").unwrap()).unwrap(),
    )
    .unwrap();
    for (i, &at) in LOCALIZED_BANK_ROOTS.iter().enumerate() {
        let b = graphics
            .banks
            .iter()
            .find(|b| b.source_offset == at)
            .unwrap();
        assert_eq!(inv["new_banks"][i]["source_offset"], at);
        assert_eq!(inv["new_banks"][i]["source_length"], b.encoded_source.len());
        assert_eq!(
            inv["new_banks"][i]["decoded_sha256"],
            gba_core::ffta_rom::digest(&b.decoded_bytes())
        );
    }
    let bytes = serde_json::to_vec_pretty(graphics).unwrap();
    let parsed: Graphics = serde_json::from_slice(&bytes).unwrap();
    parsed.verify_source(&rom).unwrap();
    assert_eq!(serde_json::to_vec_pretty(&parsed).unwrap(), bytes);
    for kind in 0..6 {
        let mut bad = graphics.clone();
        match kind {
            0 => bad.selectors[0].pointers[0].bank_offset = 0,
            1 => bad.selectors[0].pointers[0].source_offset += 2,
            2 => bad.selectors[0].fallback_variant = Some(5),
            3 => bad.selectors[1].id = bad.selectors[0].id.clone(),
            4 => {
                bad.shared_obj_banks.pop();
            }
            _ => {
                bad.selectors[1].pointers[0].source_offset =
                    bad.selectors[0].pointers[0].source_offset
            }
        };
        assert!(bad.validate().is_err());
    }
    let mut swapped = graphics.clone();
    swapped.selectors[0].pointers.swap(0, 1);
    swapped.validate().unwrap();
    assert!(swapped.verify_source(&rom).is_err());
    let mut wrong_fallback = graphics.clone();
    wrong_fallback.selectors[0].fallback_variant = None;
    wrong_fallback.validate().unwrap();
    assert!(wrong_fallback.verify_source(&rom).is_err());
    let mut missing = obj.clone();
    missing.banks.retain(|b| b.source_offset != 0xb7f874);
    assert!(graphics.validate_shared_obj(&missing).is_err());
    let mut unknown: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    unknown["selectors"][0]["unknown"] = 0.into();
    assert!(serde_json::from_value::<Graphics>(unknown).is_err());
    let mut m = crate::make_machine(rom, None);
    let mut instructions = 0;
    let cases = cases();
    for &case in &cases {
        instructions += run(&mut m, case, graphics, &obj, &mut BTreeSet::new());
    }
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("ui-graphics.json"), bytes).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"banks":16,"old_bank_records_unchanged":9,"new_banks":7,"selectors":2,"pointer_literals":10,"shared_obj_banks":2,"source_ranges":26,"source_bytes":71474,"new_ranges":17,"new_source_bytes":14436,"new_decoded_bytes":35008,"original_selector_paths":14,"new_decode_upload_paths":7,"isolated_shared_bg_tails":2,"instructions":instructions,"fallback_english_and_skip_out_of_range_preserved":true,"strict_json_source_and_shared_owner_guards":true,"runtime":"offline only; managed/consumption pending","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
