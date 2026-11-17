//! EUR opcode-39 registry pose publication and original-reader checks.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

#[test]
#[ignore = "exact EUR ROM, e162 dataset and TinyCC; Wine only, Linux/USA pending"]
fn european_script39_registry_poses_publication_and_readers() {
    const RETURN: u32 = 0x0800_0300;
    const CASES: [(u32, u32, u32); 6] = [
        (0, 4, 1),
        (0, 4, 2),
        (0, 5, 1),
        (0, 5, 2),
        (0, 68, 0),
        (0, 69, 0),
    ];
    let root = PathBuf::from(std::env::var_os("FFTA_SCRIPT39_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_SCRIPT39_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest =
        toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (162, 161));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (163, 162)
    );
    let prepared =
        preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(
        preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    assert_eq!(source.manifest.resources.len(), old.resources.len() + 1);
    for prior in &old.resources {
        let current = source
            .manifest
            .resources
            .iter()
            .find(|r| r.id == prior.id)
            .unwrap();
        assert_eq!(current, prior);
        assert_eq!(
            fs::read(prepared.directory.join(&current.path)).unwrap(),
            fs::read(previous.join(&prior.path)).unwrap()
        );
    }
    let catalog = source.script39_registry_poses.as_ref().unwrap();
    let spans = catalog.decoded.source_spans().unwrap();
    let entry = source
        .manifest
        .resources
        .iter()
        .find(|r| r.id == SCRIPT39_REGISTRY_POSES_ID)
        .unwrap();
    assert_eq!(entry.sources, catalog.sources);
    assert_eq!(spans.iter().map(|s| s.1.len()).sum::<usize>(), 91712);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let poison = |m: &mut Machine| {
        for (at, bytes, _) in &spans {
            m.bus.rom[*at as usize..*at as usize + bytes.len()].fill(0xcd);
        }
    };
    let restore = |m: &mut Machine| {
        for (at, bytes, _) in &spans {
            m.bus.rom[*at as usize..*at as usize + bytes.len()]
                .copy_from_slice(&rom[*at as usize..*at as usize + bytes.len()]);
        }
    };
    poison(&mut managed);
    let before = owner.script39_registry_pose_stats().unwrap();
    for (at, bytes, _) in &spans {
        for (i, expected) in bytes.iter().enumerate() {
            assert_eq!(managed.bus.read8(0x0c00_0000 + *at + i as u32), *expected);
        }
    }
    let after = owner.script39_registry_pose_stats().unwrap();
    assert_eq!(after.source_bytes - before.source_bytes, 91712);
    assert_eq!(after.index_bytes - before.index_bytes, 27840);
    assert_eq!(after.sequence_bytes - before.sequence_bytes, 11520);
    assert_eq!(after.pixel_bytes - before.pixel_bytes, 52352);
    let api_ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for edge in [0x3714b0usize, 0x374000, 0x89b740, 0xa52280] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.wrapping_add_signed(delta);
            assert_eq!(
                (RT_API.read16)(api_ptr, 0x0a00_0000 + at as u32),
                u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()))
            );
            assert_eq!(
                (RT_API.read32)(api_ptr, 0x0800_0000 + at as u32),
                u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
            );
        }
    }
    let old_owner = owner.unit_obj_normal_battle_stats().unwrap().source_bytes;
    managed.bus.rom[0x3948dc..0x3948e0].fill(0xcd);
    assert_eq!(
        managed.bus.read32(0x0839_48dc),
        u32::from_le_bytes(rom[0x3948dc..0x3948e0].try_into().unwrap())
    );
    assert_eq!(
        owner.unit_obj_normal_battle_stats().unwrap().source_bytes - old_owner,
        4
    );
    managed.bus.rom[0x3948dc..0x3948e0].copy_from_slice(&rom[0x3948dc..0x3948e0]);
    restore(&mut managed);

    let configure = |m: &mut Machine, (sprite, animation, part): (u32, u32, u32)| {
        m.bus.ewram.fill(0x5a);
        m.bus.iwram.fill(0);
        m.bus.ffta_sprites.enabled = false;
        m.bus.reg_ie = 0;
        m.bus.ime = false;
        m.bus.frame_ready = false;
        m.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
        m.cpu.regs[..4].copy_from_slice(&[0x0201_0000, sprite, animation, part]);
        m.cpu.regs[13] = 0x0300_7e00;
        m.cpu.regs[14] = RETURN | 1;
        m.cpu.regs[15] = 0x0802_2e20;
        m.cpu.cpsr = 0xbf;
    };
    let mut trace = crate::make_machine(rom.clone(), None);
    let initial = trace.save_state().unwrap();
    let mut visited = BTreeSet::new();
    for &case in &CASES {
        trace.load_state(&initial).unwrap();
        configure(&mut trace, case);
        for _ in 0..2000 {
            if trace.cpu.regs[15] == RETURN {
                break;
            }
            visited.insert((trace.cpu.regs[15], trace.cpu.thumb()));
            trace.bus.frame_ready = false;
            trace.step();
        }
        assert_eq!(trace.cpu.regs[15], RETURN, "original reset {case:?}");
    }
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: Some(&trace.bus.iwram),
        bios: None,
    };
    let blocks = visited
        .iter()
        .map(|&(address, thumb)| {
            let first = if thumb {
                armv4t::decode_thumb(view.read16(address), address)
            } else {
                armv4t::decode_arm(view.read32(address), address)
            };
            Block {
                addr: address,
                thumb,
                instrs: vec![first],
            }
        })
        .collect();
    let object = root.join("script39-registry-native.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut comparisons = 0;
    for native in [false, true] {
        for &case in &CASES {
            let mut raw = crate::make_machine(rom.clone(), None);
            raw.load_state(&initial).unwrap();
            managed.load_state(&initial).unwrap();
            configure(&mut raw, case);
            configure(&mut managed, case);
            poison(&mut managed);
            let before = owner.script39_registry_pose_stats().unwrap();
            for _ in 0..2000 {
                if raw.cpu.regs[15] == RETURN {
                    break;
                }
                raw.bus.frame_ready = false;
                raw.step();
            }
            assert_eq!(raw.cpu.regs[15], RETURN, "raw {case:?}");
            let mut steps = 0;
            while managed.cpu.regs[15] != RETURN {
                managed.bus.frame_ready = false;
                if native {
                    table
                        .get(managed.cpu.regs[15] | u32::from(managed.cpu.thumb()))
                        .expect("compiled reset instruction")(
                        &RT_API,
                        &mut managed as *mut Machine as *mut std::ffi::c_void,
                    );
                } else {
                    managed.step();
                }
                steps += 1;
                assert!(
                    steps < 2000,
                    "managed {case:?} PC {:08x}",
                    managed.cpu.regs[15]
                );
            }
            let after = owner.script39_registry_pose_stats().unwrap();
            assert!(
                after.index_bytes - before.index_bytes >= 4,
                "new index read for {case:?}"
            );
            restore(&mut managed);
            assert_eq!(
                raw.save_state().unwrap(),
                managed.save_state().unwrap(),
                "case {case:?}, compiled {native}"
            );
            comparisons += 1;
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"EUR opcode39 registry pose publication, typed reads and controlled original reset readers",
        "revision":[163,162], "previous_resources_preserved":old.resources.len(), "source_bytes":91712,
        "index_bytes":27840, "sequence_bytes":11520, "pixel_bytes":52352,
        "prior_normal_root_owner_preserved":true, "source_rom_poisoned":true,
        "original_reader_comparisons":comparisons, "full_state_clock_equal":true,
        "compiled_dispatch":"instruction blocks may use interpreter fallback", "ordinary_reachability":"pending",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
