//! Explicitly seeded EUR action-13 continuation; this is not ordinary gameplay.
use super::*;
use gba_core::{Bus, Machine};

#[test]
#[ignore = "exact EUR ROM and e164 dataset; Wine only, Linux/USA pending"]
fn european_script39_action13_seeded_callee() {
    const UNIT: u32 = 0x0200_2fc4;
    const MANAGER: u32 = 0x0202_4000;
    const LIST: u32 = 0x0202_4100;
    const NODE: u32 = 0x0202_4200;
    const ACTOR: u32 = 0x0202_4600;
    const CONTEXT: u32 = 0x0202_6000;
    const WORK: u32 = 0x0202_6100;
    const BODY: u32 = 0x0202_7000;
    const AUX: u32 = 0x0202_7100;
    const EQUIPMENT: u32 = 0x0202_7200;
    const SPRITE_MANAGER: u32 = 0x0202_7300;
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
    let mut comparisons = 0;
    let mut property_bytes = 0;
    let mut body_index_bytes = 0;
    let mut equipment_index_bytes = 0;
    for (at, selector, sprite, body_index) in [
        (0xb96a1e, 20u8, 102u16, 0x382708usize),
        (0xba0841, 22u8, 103u16, 0x3828d0usize),
    ] {
        let instruction = scripts
            .programs
            .iter()
            .flat_map(|p| &p.instructions)
            .find(|i| i.source_offset == at)
            .unwrap();
        let p = &instruction.parameters;
        assert_eq!(instruction.command, 39);
        assert_eq!(p.len(), 4);
        assert_eq!((p[0], u16::from_le_bytes([p[1], p[2]])), (selector, 9));
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &dataset).unwrap();
        let owner = managed.bus.rom_resources().unwrap().clone();
        let configure = |m: &mut Machine| {
            m.bus.ewram.fill(0);
            m.bus.iwram.fill(0);
            m.bus.ffta_sprites.enabled = false;
            m.bus.reg_ie = 0;
            m.bus.ime = false;
            m.bus.write8(UNIT + 4, selector);
            m.bus.write32(0x0200_f4b0, MANAGER);
            m.bus.write32(MANAGER + 0x10, LIST);
            m.bus.write32(LIST + 4, NODE);
            m.bus.write32(NODE, ACTOR);
            m.bus.write32(ACTOR, UNIT);
            m.bus.write8(ACTOR + 0x1f, 2);
            m.bus.write16(CONTEXT + 0x14, u16::from(selector));
            m.bus.write16(CONTEXT + 0x16, 9);
            m.bus.write16(CONTEXT + 0x18, u16::from(p[3]));
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
                if machine.cpu.regs[15] == 0x0809_ba84 {
                    break;
                }
                machine.step();
            }
            assert_eq!(machine.cpu.regs[15], 0x0809_ba84);
            assert_eq!(&machine.cpu.regs[..4], &[ACTOR, 13, 2, 0]);
            // The task has already cleared the equipment slots. Seed both
            // continuations identically rather than claiming allocator coverage.
            assert_eq!(machine.bus.read32(ACTOR + 0x4c), 0);
            assert_eq!(machine.bus.read32(ACTOR + 0x50), 0);
            machine.bus.write32(ACTOR + 0x30, 0);
            machine.bus.write16(ACTOR + 0x34, sprite);
            machine.bus.write8(ACTOR + 0x20, 2);
            machine.bus.write32(ACTOR + 0x40, SPRITE_MANAGER);
            machine.bus.write32(ACTOR + 0x44, BODY);
            machine.bus.write32(ACTOR + 0x48, AUX);
            machine.bus.write32(ACTOR + 0x4c, EQUIPMENT);
        }
        let poisoned = [body_index, 0x3868d8, 0x3868dc];
        for &offset in &poisoned {
            managed.bus.rom[offset..offset + 4].fill(0xcd);
        }
        let before_property = owner.unit_obj_property_words_stats().unwrap().source_bytes;
        let before_body = owner.unit_obj_normal_battle_stats().unwrap().index_bytes;
        let before_equipment = owner.unit_obj_equipment_stats().unwrap().index_bytes;
        for machine in [&mut raw, &mut managed] {
            for _ in 0..20000 {
                if machine.cpu.regs[15] == 0x0812_a940 {
                    break;
                }
                machine.step();
            }
            assert_eq!(machine.cpu.regs[15], 0x0812_a940, "selector {selector}");
            assert_eq!(machine.bus.read32(SPRITE_MANAGER), BODY);
            assert_eq!(machine.bus.read32(ACTOR + 0x44), BODY);
            assert_eq!(machine.bus.read16(ACTOR + 0x36), 48);
            assert_eq!(
                (machine.bus.read16(BODY + 6), machine.bus.read16(BODY + 8)),
                (sprite, 50)
            );
            assert_eq!(machine.bus.read32(BODY), 0x65);
            assert_eq!(machine.bus.read32(ACTOR + 0x4c), EQUIPMENT);
            assert_eq!(
                (
                    machine.bus.read16(EQUIPMENT + 6),
                    machine.bus.read16(EQUIPMENT + 8)
                ),
                (185, 6)
            );
            assert_eq!(machine.bus.read32(EQUIPMENT), 0x40065);
            assert_eq!(machine.bus.read8(EQUIPMENT + 0x1d), 1);
            assert_eq!(machine.bus.read32(ACTOR + 0x50), 0);
        }
        let property =
            owner.unit_obj_property_words_stats().unwrap().source_bytes - before_property;
        let body = owner.unit_obj_normal_battle_stats().unwrap().index_bytes - before_body;
        let equipment = owner.unit_obj_equipment_stats().unwrap().index_bytes - before_equipment;
        assert!(property > 0 && body >= 4 && equipment >= 8);
        property_bytes += property;
        body_index_bytes += body;
        equipment_index_bytes += equipment;
        for &offset in &poisoned {
            managed.bus.rom[offset..offset + 4].copy_from_slice(&rom[offset..offset + 4]);
        }
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
        comparisons += 1;
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"EUR original action13 callee continuation with identical objects seeded after task cleanup",
        "controlled_callee_comparisons":comparisons, "property_source_bytes":property_bytes,
        "body_index_bytes":body_index_bytes, "equipment_index_bytes":equipment_index_bytes,
        "selected_index_rom_bytes_poisoned":true, "raw_managed_full_state_and_guest_clock_equal":true,
        "uninterrupted_allocator_path":"pending", "actor_overrides":"pending", "ordinary_reachability":"pending",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
