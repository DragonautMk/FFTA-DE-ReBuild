//! Controlled EUR opcode-39 special-branch selection, not ordinary gameplay.
use super::*;
use gba_core::{Bus, Machine};

#[test]
#[ignore = "exact EUR ROM and e164 dataset; Wine only, Linux/USA pending"]
fn european_script39_action13_task_branch() {
    const UNIT: u32 = 0x0200_2fc4;
    const OTHER_UNIT: u32 = 0x0202_1100;
    const MANAGER: u32 = 0x0202_4000;
    const LIST: u32 = 0x0202_4100;
    const FIRST: u32 = 0x0202_4200;
    const SECOND: u32 = 0x0202_4220;
    const OTHER_ACTOR: u32 = 0x0202_4400;
    const ACTOR: u32 = 0x0202_4600;
    const CONTEXT: u32 = 0x0202_6000;
    const WORK: u32 = 0x0202_6100;
    let root = PathBuf::from(std::env::var_os("FFTA_ACTION13_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_ACTION13_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let manifest: Manifest =
        toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (164, 163)
    );
    let scripts: ScriptArchive =
        serde_json::from_slice(&fs::read(dataset.join(SCRIPT_ARCHIVE_FILE)).unwrap()).unwrap();
    scripts.verify_source(&rom).unwrap();
    let mut cases = Vec::new();
    for (at, selector) in [(0xb96a1e, 20), (0xba0841, 22)] {
        let instruction = scripts
            .programs
            .iter()
            .flat_map(|p| &p.instructions)
            .find(|i| i.source_offset == at)
            .unwrap();
        assert_eq!(instruction.command, 39);
        let p = &instruction.parameters;
        assert_eq!(p.len(), 4);
        assert_eq!((p[0], u16::from_le_bytes([p[1], p[2]])), (selector, 9));
        cases.push((selector, 9u16, p[3], 0x0809_ba84u32, 13u32));
    }
    // A controlled adjacent group must select the default task branch.
    cases.push((20, 8, 0, 0x0809_d9c4, 32));
    let mut comparisons = 0;
    for (selector, group, flag, expected_pc, expected_r1) in cases {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &dataset).unwrap();
        let configure = |m: &mut Machine| {
            m.bus.ewram.fill(0);
            m.bus.iwram.fill(0);
            m.bus.ffta_sprites.enabled = false;
            m.bus.reg_ie = 0;
            m.bus.ime = false;
            m.bus.write8(UNIT + 4, selector);
            m.bus.write32(0x0200_f4b0, MANAGER);
            m.bus.write32(MANAGER + 0x10, LIST);
            m.bus.write32(LIST + 4, FIRST);
            m.bus.write32(FIRST, OTHER_ACTOR);
            m.bus.write32(FIRST + 8, SECOND);
            m.bus.write32(SECOND, ACTOR);
            m.bus.write32(OTHER_ACTOR, OTHER_UNIT);
            m.bus.write32(ACTOR, UNIT);
            m.bus.write8(ACTOR + 0x1f, 2);
            m.bus.write16(CONTEXT + 0x14, u16::from(selector));
            m.bus.write16(CONTEXT + 0x16, group);
            m.bus.write16(CONTEXT + 0x18, u16::from(flag));
            m.cpu.regs = [0; 16];
            m.cpu.regs[1] = CONTEXT;
            m.cpu.regs[2] = WORK;
            m.cpu.regs[13] = 0x0300_7c00;
            m.cpu.regs[14] = 0x0800_0301;
            m.cpu.regs[15] = 0x0812_a8c4;
            m.cpu.cpsr = 0xbf;
        };
        for machine in [&mut raw, &mut managed] {
            configure(machine);
            for _ in 0..2000 {
                if machine.cpu.regs[15] == expected_pc {
                    break;
                }
                machine.step();
            }
            assert_eq!(
                machine.cpu.regs[15], expected_pc,
                "selector {selector}, group {group}"
            );
            assert_eq!(&machine.cpu.regs[..4], &[ACTOR, expected_r1, 2, 0]);
            assert_eq!(machine.bus.read32(WORK + 4), ACTOR);
        }
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
        comparisons += 1;
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"EUR original opcode39 task prefix to action13/default callee entry",
        "script_instruction_witnesses":2, "controlled_branch_comparisons":comparisons,
        "raw_managed_full_state_and_guest_clock_equal":true,
        "callee_execution":"pending", "actor_overrides":"pending", "ordinary_reachability":"pending",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
