//! Original global pair wrapper and live44-bit RAM-mask filter.
use super::*;
use gba_core::{
    rom_data::global_pair_flags::{Catalog, GlobalPairFlags, END_OFFSET, SOURCE_OFFSET},
    Bus, Machine,
};
use std::collections::BTreeSet;
pub(super) const RETURN: u32 = 0x08000300;
pub(super) const STACK: u32 = 0x03007e00;
const MASK: u32 = 0x02024000;
#[derive(Clone, Copy, Debug)]
pub(super) enum Case {
    Direct { slot: u32, mode: u32, ignored: u32 },
    Mask { live: [u8; 8], ignored: u32 },
}
pub(super) fn setup(m: &mut Machine, c: Case) {
    for i in 0..13 {
        m.cpu.regs[i] = 0x11220000 + i as u32;
    }
    m.bus.ewram[0x23ff0..0x24110].fill(0x3c);
    m.bus.iwram[0x7d00..0x7e20].fill(0x5a);
    match c {
        Case::Direct {
            slot,
            mode,
            ignored,
        } => {
            m.cpu.regs[..3].copy_from_slice(&[ignored, slot, mode]);
            m.cpu.regs[15] = 0x0813779c;
        }
        Case::Mask { live, ignored } => {
            for (i, v) in live.into_iter().enumerate() {
                m.bus.write8(MASK + i as u32, v);
            }
            m.cpu.regs[0] = MASK;
            m.cpu.regs[1] = ignored;
            m.cpu.regs[15] = 0x081377b4;
        }
    }
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.cpsr = 0xbf;
    m.bus.reg_ie = 0;
    m.bus.ime = false;
}
pub(super) fn direct_address(slot: u32, mode: u32) -> (u32, u32) {
    let bit = slot
        .wrapping_mul(2)
        .wrapping_add(u32::from(mode as u8 != 0));
    (
        (0x08000000 + SOURCE_OFFSET).wrapping_add(((bit as i32) >> 3) as u32),
        bit & 7,
    )
}
fn expected(c: Case, data: &GlobalPairFlags) -> (bool, Vec<u32>) {
    match c {
        Case::Direct { slot, mode, .. } => (
            data.flag(slot, mode).unwrap(),
            vec![direct_address(slot, mode).0],
        ),
        Case::Mask { live, .. } => {
            let mut reads = vec![];
            for slot in 0..44 {
                if live[slot / 8] & (1 << (slot % 8)) != 0 {
                    reads.push(0x08000000 + SOURCE_OFFSET + (slot / 4) as u32);
                    if !data.flag(slot as u32, 0).unwrap() {
                        return (false, reads);
                    }
                }
            }
            (true, reads)
        }
    }
}
pub(super) fn execute(
    m: &mut Machine,
    c: Case,
    answer: bool,
    reads: &[u32],
    visited: &mut BTreeSet<(u32, bool)>,
) -> usize {
    setup(m, c);
    let ram = m.bus.ewram[0x23ff0..0x24110].to_vec();
    let mut actual = vec![];
    let mut n = 0;
    while m.cpu.regs[15] != RETURN {
        assert!(n < 5000, "global pair reader {c:?}");
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        if m.cpu.regs[15] == 0x08135a34 {
            actual.push(m.cpu.regs[0]);
        }
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    assert_eq!(m.cpu.regs[0], u32::from(answer), "global pair answer {c:?}");
    assert_eq!(actual, reads, "global pair source addresses {c:?}");
    assert_eq!(m.cpu.regs[13], STACK);
    for i in 4..12 {
        assert_eq!(m.cpu.regs[i], 0x11220000 + i as u32, "callee r{i} {c:?}");
    }
    assert_eq!(&m.bus.ewram[0x23ff0..0x24110], &ram);
    assert!(m.bus.iwram[0x7d00..0x7de0].iter().all(|&v| v == 0x5a));
    assert!(m.bus.iwram[0x7e00..0x7e20].iter().all(|&v| v == 0x5a));
    n
}
pub(super) fn run(
    m: &mut Machine,
    c: Case,
    data: &GlobalPairFlags,
    visited: &mut BTreeSet<(u32, bool)>,
) -> usize {
    let (answer, reads) = expected(c, data);
    execute(m, c, answer, &reads, visited)
}
pub(super) fn cases(data: &GlobalPairFlags) -> Vec<Case> {
    let mut out = vec![];
    for slot in 0..44 {
        for mode in [0, 1] {
            out.push(Case::Direct {
                slot,
                mode,
                ignored: 0xaabb0000 + slot,
            });
        }
    }
    for slot in [0, 43] {
        for mode in [0x100, 0x101, 0xff, 0xffff0000] {
            out.push(Case::Direct {
                slot,
                mode,
                ignored: u32::MAX,
            });
        }
    }
    for live in [
        [0; 8],
        [255; 8],
        [0, 0, 0, 0, 0, 0xf0, 0xff, 0xff],
        [0x81, 0x12, 0x44, 0x08, 0x22, 0x04, 0x80, 0xff],
    ] {
        out.push(Case::Mask {
            live,
            ignored: 0xdeadbeef,
        });
    }
    for answer in [false, true] {
        if let Some(slot) = (0..44).find(|&s| data.flag(s, 0) == Some(answer)) {
            let mut live = [0; 8];
            live[slot as usize / 8] = 1 << (slot % 8);
            out.push(Case::Mask { live, ignored: 0 });
        }
    }
    out
}
#[test]
#[ignore = "private EUR11B global packed pair source proof; one reduced Wine oracle, Linux/USA pending"]
fn european_global_pair_flags_match_original_wrapper_and_live_mask_filter() {
    let root =
        PathBuf::from(std::env::var_os("FFTA_STATE_TEST_GLOBAL_PAIR_FLAGS_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let independent: GlobalPairFlags = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_GLOBAL_PAIR_FLAGS_RECORDS").unwrap()).unwrap(),
    )
    .unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    assert_eq!(data, &independent);
    data.validate().unwrap();
    data.verify_source(&rom).unwrap();
    assert_eq!(
        data.source_bytes().as_slice(),
        &rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
    );
    assert_eq!((catalog.sources.len(), catalog.sources[0].length), (1, 11));
    let bytes = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        &serde_json::from_slice::<GlobalPairFlags>(&bytes).unwrap(),
        data
    );
    let shape = serde_json::to_value(data).unwrap();
    for case in 0..4 {
        let mut v = shape.clone();
        match case {
            0 => v["extra"] = true.into(),
            1 => v["groups"][0] = serde_json::json!([0, 0, 0]),
            2 => v["groups"] = serde_json::json!([]),
            _ => {
                v.as_object_mut().unwrap().remove("source_offset");
            }
        }
        assert!(serde_json::from_value::<GlobalPairFlags>(v).is_err());
    }
    let mut bad = data.clone();
    bad.source_offset += 1;
    assert!(bad.validate().is_err());
    bad = data.clone();
    bad.groups[0][0] = 4;
    assert!(bad.validate().is_err());
    bad = data.clone();
    bad.groups[10][3] ^= 1;
    bad.validate().unwrap();
    assert!(bad.verify_source(&rom).is_err());
    assert!(data.flag(44, 0).is_none());
    assert!(data.flag(u32::MAX, 0).is_none());
    let mut m = crate::make_machine(rom.clone(), None);
    let mut visited = BTreeSet::new();
    let real = cases(data);
    let mut instructions = 0;
    for &c in &real {
        instructions += run(&mut m, c, data, &mut visited);
    }
    let mut raw = 0;
    for slot in [44, 55, 56, u32::MAX, 0x80000000] {
        for mode in [0, 0x101] {
            let (a, bit) = direct_address(slot, mode);
            let answer = ((rom[(a - 0x08000000) as usize] >> bit) & 1) != 0;
            instructions += execute(
                &mut m,
                Case::Direct {
                    slot,
                    mode,
                    ignored: 0x11223344,
                },
                answer,
                &[a],
                &mut visited,
            );
            raw += 1;
        }
    }
    let mut synthetic = 0;
    let mut modified = data.clone();
    modified.groups = std::array::from_fn(|i| std::array::from_fn(|j| ((i * 3 + j) % 4) as u8));
    modified.validate().unwrap();
    assert!(modified.verify_source(&rom).is_err());
    m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
        .copy_from_slice(&modified.source_bytes());
    assert!(Catalog::from_rom(&m.bus.rom).is_err());
    for slot in 0..44 {
        for mode in [0x100, 0xff] {
            instructions += run(
                &mut m,
                Case::Direct {
                    slot,
                    mode,
                    ignored: 0xffffffff,
                },
                &modified,
                &mut visited,
            );
            synthetic += 1;
        }
    }
    for pair in [0, 1, 3] {
        modified.groups = [[pair; 4]; 11];
        m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
            .copy_from_slice(&modified.source_bytes());
        instructions += run(
            &mut m,
            Case::Mask {
                live: [255; 8],
                ignored: 0xabbaabba,
            },
            &modified,
            &mut visited,
        );
        synthetic += 1;
    }
    m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
        .copy_from_slice(&rom[SOURCE_OFFSET as usize..END_OFFSET as usize]);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("global-pair-flags.json"), &bytes).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"source_bytes":11,"pairs":44,"real_cases":real.len(),"raw_outside_wrapping_cases":raw,"synthetic_cases":synthetic,"instructions_coverage":instructions,"pcs":visited.len(),"original_wrapper_and_live_mask_earlyexit":true,"bothbits_u8mode_ignored_r0_signed_slot_wrap":true,"strict_json_groups_pairrange_offsets_source":true,"exact_load_addresses_callee_stack_ram_guards":true,"padding3bytes":"unowned","highmode_ordinary_use":"unproved; direct wrapper proven","synthetic":"unauthenticated/unpublished","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
