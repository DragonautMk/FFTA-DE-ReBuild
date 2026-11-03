//! Raw UI palette source contract and original initialization/selector/copy proof.
use super::*;
use gba_core::{
    rom_data::ui_palettes::{Catalog, Palettes, REFERENCES},
    Bus, Machine,
};
use std::collections::BTreeSet;
pub(super) const STACK: u32 = 0x03007e00;
pub(super) const RETURN: u32 = 0x08000300;
pub(super) fn inventory() -> serde_json::Value {
    serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_UI_PALETTE_INVENTORY").unwrap()).unwrap(),
    )
    .unwrap()
}
pub(super) fn initialize(m: &mut Machine, visited: &mut BTreeSet<(u32, bool)>) -> usize {
    m.cpu.regs[13] = STACK;
    m.cpu.cpsr = 0xbf;
    m.cpu.regs[15] = 0x080027ee;
    let mut count = 0;
    while m.cpu.regs[15] != 0x080027fe {
        assert!(count < 100);
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
        count += 1;
    }
    assert_eq!(&m.bus.iwram[0x5e78..0x5f28], &m.bus.rom[0xd96838..0xd968e8]);
    count
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Case {
    pub reference: usize,
    pub mode: Option<u8>,
}
pub(super) fn cases() -> Vec<Case> {
    let mut out = Vec::new();
    for (i, r) in REFERENCES.iter().enumerate() {
        if r.call == 0x4fb80 || r.call == 0x12f2bc {
            continue;
        }
        out.push(Case {
            reference: i,
            mode: None,
        });
    }
    for call in [0x4fb80, 0x12f2bc] {
        for mode in [0, 1, 2, 3, 0xfe] {
            let source = match (call, mode & 3) {
                (0x4fb80, 1) => 0x411310,
                (0x4fb80, 2) => 0x410830,
                (0x4fb80, _) => 0x40fcb0,
                (_, 1) => 0x4113b0,
                (_, 2) => 0x4108d0,
                _ => 0x40fd50,
            };
            let reference = REFERENCES
                .iter()
                .position(|r| r.call == call && r.source_offset == source)
                .unwrap();
            out.push(Case {
                reference,
                mode: Some(mode),
            });
        }
    }
    out
}
pub(super) fn setup(m: &mut Machine, case: Case, inv: &serde_json::Value) {
    let r = &REFERENCES[case.reference];
    let seed = &inv["references"][case.reference];
    for i in 0..13 {
        m.cpu.regs[i] = seed["seed_registers"][i].as_u64().unwrap_or(0) as u32;
    }
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.cpsr = 0xbf;
    m.cpu.regs[15] = 0x08000000 + r.reader;
    if let Some(mode) = case.mode {
        m.bus.write8(0x02002fc2, mode);
        m.cpu.regs[15] = if r.call == 0x4fb80 {
            0x0804fafe
        } else {
            0x0812f27e
        };
    }
}
pub(super) fn run(
    m: &mut Machine,
    case: Case,
    inv: &serde_json::Value,
    visited: &mut BTreeSet<(u32, bool)>,
) -> usize {
    setup(m, case, inv);
    let r = &REFERENCES[case.reference];
    let stop = 0x08000000 + r.call + 4;
    let mut count = 0;
    let mut reached = false;
    while m.cpu.regs[15] != stop {
        assert!(
            count < 2000,
            "UI palette original path {case:?} at{:08x}",
            m.cpu.regs[15]
        );
        if m.cpu.regs[15] == 0x08000000 + r.call {
            assert_eq!(
                &m.cpu.regs[..4],
                &[
                    r.destination,
                    0x08000000 + r.source_offset,
                    r.length,
                    0x03005ee9
                ]
            );
            reached = true;
        }
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
        count += 1;
    }
    assert!(reached);
    count
}
#[test]
#[ignore = "EUR raw UI palette original source/initializer/copy/Color-Mode proof; Wine only, Linux/USA pending"]
fn european_ui_palettes_match_original_sources_and_copy_paths() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_UI_PALETTE_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let inv = inventory();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let palettes = &catalog.decoded;
    assert_eq!(
        (
            palettes.banks.len(),
            REFERENCES.len(),
            catalog.sources.len()
        ),
        (21, 23, 21)
    );
    assert_eq!(catalog.sources.iter().map(|s| s.length).sum::<u32>(), 2656);
    for (i, b) in palettes.banks.iter().enumerate() {
        assert_eq!(inv["banks"][i]["source_offset"], b.source_offset);
        assert_eq!(inv["banks"][i]["source_length"], b.colors.len() * 2);
        assert_eq!(
            inv["banks"][i]["sha256"],
            gba_core::ffta_rom::digest(&b.source_bytes())
        );
    }
    let encoded = serde_json::to_vec_pretty(palettes).unwrap();
    let parsed: Palettes = serde_json::from_slice(&encoded).unwrap();
    parsed.verify_source(&rom).unwrap();
    assert_eq!(serde_json::to_vec_pretty(&parsed).unwrap(), encoded);
    for kind in 0..6 {
        let mut bad = palettes.clone();
        match kind {
            0 => bad.banks[0].colors.clear(),
            1 => {
                bad.banks[0].colors.pop();
            }
            2 => bad.banks[0].source_offset |= 1,
            3 => bad.banks[0].source_offset = u32::MAX - 1,
            4 => bad.banks.swap(0, 1),
            _ => bad.banks[1].source_offset = bad.banks[0].source_offset,
        };
        assert!(bad.validate().is_err());
    }
    let mut edited = palettes.clone();
    edited.banks[0].colors[0] ^= 0x8000;
    edited.validate().unwrap();
    assert!(edited.verify_source(&rom).is_err());
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    let mut unknown: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    unknown["banks"][0]["unknown"] = 0.into();
    assert!(serde_json::from_value::<Palettes>(unknown).is_err());
    let mut m = crate::make_machine(rom.clone(), None);
    let mut visited = BTreeSet::new();
    let mut instructions = initialize(&mut m, &mut visited);
    // Direct original callback with both word-copy and byte-tail paths; high-bit
    // colors remain unchanged. These synthetic bytes are original test content.
    for alignment in [0, 1, 2, 3] {
        let dest = 0x02030010 + alignment;
        let src = 0x02032010 + alignment;
        let length = 35;
        for i in 0..64 {
            m.bus.write8(0x02030000 + i, 0x5a);
            m.bus.write8(0x02032000 + i, (i * 37) as u8);
        }
        let expected: Vec<_> = (0..length).map(|i| m.bus.read8(src + i)).collect();
        m.cpu.regs[..4].copy_from_slice(&[dest, src, length, 0x03005ee9]);
        m.cpu.regs[13] = STACK;
        m.cpu.regs[14] = RETURN | 1;
        m.cpu.regs[15] = 0x08145c14;
        m.cpu.cpsr = 0xbf;
        for _ in 0..1000 {
            m.bus.frame_ready = false;
            m.step();
            instructions += 1;
            if m.cpu.regs[15] == RETURN {
                break;
            }
        }
        assert_eq!(m.cpu.regs[15], RETURN);
        assert!((0..64).all(|i| m.bus.read8(0x02030000 + i)
            == if (dest..dest + length).contains(&(0x02030000 + i)) {
                expected[(0x02030000 + i - dest) as usize]
            } else {
                0x5a
            }));
    }
    let cases = cases();
    let mut outputs = 0;
    for case in &cases {
        for i in 0..1056 {
            m.bus.write8(0x03003850 + i, 0x5a);
        }
        setup(&mut m, *case, &inv);
        let r = &REFERENCES[case.reference];
        let mut before = None;
        let stop = 0x08000000 + r.call + 4;
        for _ in 0..2000 {
            if m.cpu.regs[15] == 0x08000000 + r.call {
                assert_eq!(
                    &m.cpu.regs[..4],
                    &[
                        r.destination,
                        0x08000000 + r.source_offset,
                        r.length,
                        0x03005ee9
                    ]
                );
                before = Some(m.bus.iwram.clone());
            }
            m.bus.frame_ready = false;
            m.step();
            instructions += 1;
            if m.cpu.regs[15] == stop {
                break;
            }
        }
        assert_eq!(m.cpu.regs[15], stop);
        let mut expected = before.expect("actual original copy call reached");
        let at = r.destination as usize & 0x7fff;
        expected[at..at + r.length as usize]
            .copy_from_slice(&rom[r.source_offset as usize..(r.source_offset + r.length) as usize]);
        assert_eq!(&m.bus.iwram[0x3850..0x3c70], &expected[0x3850..0x3c70]);
        outputs += 1;
    }
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("ui-palettes.json"), encoded).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"banks":21,"references":23,"ranges":21,"source_bytes":2656,"words":1328,"original_initializer_bytes":176,"original_copy_paths":outputs,"color_mode_selector_cases":10,"unaligned_callback_cases":4,"instructions":instructions,"strict_json_source_and_bit15_preserved":true,"runtime":"offline only; managed/consumption pending","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
