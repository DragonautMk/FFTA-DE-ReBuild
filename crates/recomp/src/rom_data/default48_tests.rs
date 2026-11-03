//! Original constructor copy/common patch prefix; allocation and live-unit tails excluded.
use super::*;
use gba_core::{
    rom_data::default48::{Catalog, Template, ALTERNATE_BASE, END_OFFSET, SOURCE_OFFSET},
    Bus, Machine,
};
use std::collections::BTreeSet;
pub(super) const STACK: u32 = 0x03007e00;
pub(super) const SCRATCH: u32 = STACK - 0x54;
pub(super) const RETURN: u32 = 0x08000300;
pub(super) const PATCH_END: u32 = 0x08127eb0;
#[derive(Clone, Copy, Debug)]
pub(super) struct Case {
    pub args: [u32; 4],
    pub flag: u32,
    pub selector: u32,
    pub mode: u32,
}
impl Case {
    pub(super) fn stop(self) -> u32 {
        if matches!(self.mode as u8, 1 | 2) {
            PATCH_END
        } else {
            RETURN
        }
    }
}
pub(super) fn cases() -> Vec<Case> {
    let mut out: Vec<_> = (0..256)
        .map(|id| Case {
            args: [id, 0x100ab, 0xffff00cd, 0x100ef],
            flag: [0, 1, 2, 3, 255][id as usize % 5],
            selector: [0, 1, 2, 255][id as usize % 4],
            mode: 1,
        })
        .collect();
    for id in [0, 19, 20, 23, 24, 127, 128, 255] {
        out.push(Case {
            args: [0xffff0000 | id, 256, 511, 0x80000080],
            flag: 0x103,
            selector: 0x102,
            mode: 0x102,
        });
    }
    out.push(Case {
        args: [0x10000, 0, 0, 0],
        flag: 0x101,
        selector: 0x100,
        mode: 0x101,
    });
    for mode in [0, 3, 255, 0x10000, 0x10003, 0xffffff00] {
        out.push(Case {
            args: [0, 1, 2, 3],
            flag: 1,
            selector: 2,
            mode,
        });
    }
    out
}
pub(super) fn setup(m: &mut Machine, case: Case) {
    for i in 0..13 {
        m.cpu.regs[i] = 0x11220000 + i as u32;
    }
    for at in STACK - 0x100..STACK + 32 {
        m.bus.write8(at, 0x5a);
    }
    for (i, v) in [case.flag, case.selector, case.mode]
        .into_iter()
        .enumerate()
    {
        m.bus.write32(STACK + i as u32 * 4, v);
    }
    m.cpu.regs[..4].copy_from_slice(&case.args);
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = 0x08127dc4;
    m.cpu.cpsr = 0xbf;
}
pub(super) fn expected(m: &Machine, case: Case, data: &Template) -> [u8; 48] {
    if Template::default_id(case.args[0]) {
        return data.patched(
            case.args[0],
            case.selector,
            case.args[1..4].try_into().unwrap(),
            case.flag,
        );
    }
    let at = ALTERNATE_BASE as usize + (usize::from(case.args[0] as u8) - 20) * 48;
    let mut raw: [u8; 48] = m.bus.rom[at..at + 48].try_into().unwrap();
    raw[0] = case.args[0] as u8;
    if case.selector as u8 > 1 {
        raw[1] = case.selector as u8;
    }
    for (byte, argument) in raw[5..8].iter_mut().zip(&case.args[1..]) {
        *byte = *argument as u8;
    }
    let mask = match case.flag as u8 {
        1 => 0x8000,
        2 => 0x2000,
        3 => 0x1000,
        _ => 0,
    };
    let flags = u16::from_le_bytes(raw[42..44].try_into().unwrap()) | mask;
    raw[42..44].copy_from_slice(&flags.to_le_bytes());
    raw
}
pub(super) fn check(m: &mut Machine, case: Case, data: &Template) {
    assert_eq!(m.cpu.regs[15], case.stop());
    assert_eq!(m.cpu.regs[11], 0x1122000b);
    if case.stop() == PATCH_END {
        assert_eq!(m.cpu.regs[13], SCRATCH);
        assert_eq!(
            &m.bus.iwram[(SCRATCH & 0x7fff) as usize..(SCRATCH & 0x7fff) as usize + 48],
            &expected(m, case, data),
            "{case:?}"
        );
        assert_eq!(m.bus.read32(SCRATCH + 48), case.args[3] as u8 as u32);
        assert_eq!(m.cpu.regs[7], case.args[0] as u8 as u32);
        assert_eq!(m.cpu.regs[8], case.selector as u8 as u32);
        assert_eq!(m.cpu.regs[9], case.args[1] as u8 as u32);
        assert_eq!(m.cpu.regs[10], case.args[2] as u8 as u32);
    } else {
        assert_eq!(m.cpu.regs[0], 0);
        assert_eq!(m.cpu.regs[13], STACK);
        for i in 4..13 {
            assert_eq!(m.cpu.regs[i], 0x11220000 + i as u32);
        }
    }
    for (i, v) in [case.flag, case.selector, case.mode]
        .into_iter()
        .enumerate()
    {
        assert_eq!(m.bus.read32(STACK + i as u32 * 4), v);
    }
    for at in STACK - 0x100..STACK - 0x80 {
        assert_eq!(m.bus.read8(at), 0x5a);
    }
    for at in STACK + 12..STACK + 32 {
        assert_eq!(m.bus.read8(at), 0x5a);
    }
}
pub(super) fn run(
    m: &mut Machine,
    case: Case,
    data: &Template,
    visited: &mut BTreeSet<(u32, bool)>,
) -> usize {
    setup(m, case);
    let mut n = 0;
    while m.cpu.regs[15] != case.stop() {
        assert!(n < 2000, "default48 {case:?} at{:08x}", m.cpu.regs[15]);
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    check(m, case, data);
    n
}
#[test]
#[ignore = "private EUR default48 inventory and original constructor-prefix proof; reduced Wine only, Linux/USA pending"]
fn european_default48_matches_original_constructor_copy_patch_and_mode_gate() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DEFAULT48_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let independent: Template = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_DEFAULT48_FIELDS").unwrap()).unwrap(),
    )
    .unwrap();
    let inv: serde_json::Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_DEFAULT48_INVENTORY").unwrap()).unwrap(),
    )
    .unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    assert_eq!(data, &independent);
    assert_eq!(
        data.source_bytes(),
        rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
    );
    assert_eq!(
        inv["source_sha256"],
        gba_core::ffta_rom::digest(&data.source_bytes())
    );
    assert_eq!(
        (
            catalog.sources.len(),
            catalog.sources[0].offset,
            catalog.sources[0].length
        ),
        (1, SOURCE_OFFSET, 48)
    );
    let bytes = serde_json::to_vec_pretty(data).unwrap();
    let parsed: Template = serde_json::from_slice(&bytes).unwrap();
    parsed.verify_source(&rom).unwrap();
    assert_eq!(serde_json::to_vec_pretty(&parsed).unwrap(), bytes);
    for kind in 0..5 {
        let mut bad = serde_json::to_value(data).unwrap();
        match kind {
            0 => bad["unknown"] = 0.into(),
            1 => bad["byte00"] = 256.into(),
            2 => bad["flags2a"] = 65536.into(),
            3 => {
                bad["halfwords08_28"].as_array_mut().unwrap().pop();
            }
            _ => bad["bytes02_04"][0] = (-1).into(),
        };
        assert!(serde_json::from_value::<Template>(bad).is_err());
    }
    let mut bad = data.clone();
    bad.source_offset += 48;
    assert!(bad.validate().is_err());
    for kind in 0..4 {
        let mut bad = data.clone();
        match kind {
            0 => bad.byte00 ^= 1,
            1 => bad.bytes02_04[1] ^= 1,
            2 => bad.flags2a = 1,
            _ => bad.halfwords2c_2e[1] = 0xffff,
        };
        bad.validate().unwrap();
        assert!(bad.verify_source(&rom).is_err());
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    let mut m = crate::make_machine(rom, None);
    let mut visited = BTreeSet::new();
    let initialization = super::ui_palette_tests::initialize(&mut m, &mut visited);
    let ewram = m.bus.ewram.clone();
    let iwram = m.bus.iwram.clone();
    let mut instructions = 0;
    let cases = cases();
    for &case in &cases {
        instructions += run(&mut m, case, data, &mut visited);
    }
    let mut synthetic = data.clone();
    synthetic.byte00 = 0xa5;
    synthetic.byte01 = 0x80;
    synthetic.bytes02_04 = [0xfe, 0x80, 0xff];
    synthetic.arguments05_07 = [1, 2, 3];
    synthetic.halfwords08_28 = std::array::from_fn(|i| 0x8000 + i as u16 * 257);
    synthetic.flags2a = 0x0555;
    synthetic.halfwords2c_2e = [0xdead, 0xbeef];
    assert!(synthetic.verify_source(&m.bus.rom).is_err());
    m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
        .copy_from_slice(&synthetic.source_bytes());
    let mut synthetic_cases = 0;
    for id in [0, 19, 24, 127, 128, 255] {
        for flag in [0, 1, 2, 3, 255] {
            let case = Case {
                args: [id, 0x1ff, 0x100, 0x180],
                flag,
                selector: if flag % 2 == 0 { 1 } else { 0x102 },
                mode: 2,
            };
            instructions += run(&mut m, case, &synthetic, &mut visited);
            synthetic_cases += 1;
        }
    }
    m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize].copy_from_slice(&data.source_bytes());
    assert_eq!(m.bus.ewram, ewram);
    for (i, (&actual, &old)) in m.bus.iwram.iter().zip(&iwram).enumerate() {
        if !(0x7d00..0x7e20).contains(&i) {
            assert_eq!(actual, old, "IWRAM{i:x}");
        }
    }
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("default48.json"), bytes).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"source_bytes":48,"original_cases":cases.len(),"synthetic_cases":synthetic_cases,"instructions":instructions,"initialization_instructions":initialization,"distinct_instructions":visited.len(),"all256_ids_default_vs_four_alternate_sources":true,"u8_inputs_mode1_2_gate_signed_id_selector_gt1_live_arguments_flag_or":true,"strict_json_fixed_arrays_source_coherent_edit_guards":true,"scratch_frame_incoming_arguments_callee_invalid_return_outer_ram_guards":true,"original_iwram_copy_helper_initialized_by_original_rom_code":true,"allocation_live_unit_tail_after_patch":"not claimed or replaced","following240bytes":"unowned, no inferred records","managed_runtime":"pending; EUR e28-d27 unchanged","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
