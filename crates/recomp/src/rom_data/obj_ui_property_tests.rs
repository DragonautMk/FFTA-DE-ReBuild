//! Reduced EUR managed, original/native, host and lifecycle acceptance.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
const NEW: [u32; 1] = [0x3950a4];
fn machine_source(rom: &[u8], list: &gba_core::rom_data::obj_lists::List) -> Vec<u8> {
    rom[list.source_offset as usize..list.source_offset as usize + list.source_len()].to_vec()
}
const RETURN: u32 = 0x08000300;
const STACK: u32 = 0x03007c00;
const BUFFER: u32 = 0x02030010;
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_UI_PROPERTY_OBJ_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 64] {
    [
        MANIFEST_FILE,
        OBJ_DIMENSIONS_FILE,
        BG_DESCRIPTORS_FILE,
        BG_BASE_PALETTES_FILE,
        BG_ALTERNATE_PALETTES_FILE,
        BG_PACKET_PROGRAMS_FILE,
        BG_MAIN_GRAPHICS_FILE,
        BG_TRANSFER_PROGRAMS_FILE,
        SHORT_TEXTS_FILE,
        TEXT_CATALOGS_FILE,
        FONT_BANK_FILE,
        NAME_KEYBOARD_FILE,
        OBJ_STATIC_LISTS_FILE,
        OBJ_ANIMATION_BANKS_FILE,
        OBJ_CUSTOM_GRAPHICS_FILE,
        UI_RAW_PALETTES_FILE,
        UI_CUSTOM_GRAPHICS_FILE,
        ITEM_RECORDS_FILE,
        ITEM_LINKS_FILE,
        METADATA16_FILE,
        METADATA52_FILE,
        METADATA4_FILE,
        METADATA2_FILE,
        DEFAULT48_FILE,
        CONSTRUCTOR_ALTERNATES_FILE,
        METADATA14_FILE,
        METADATA12_FILE,
        GLOBAL_PAIR_FLAGS_FILE,
        EVALUATION_PARAMETERS_FILE,
        PROGRESSION_REWARDS_FILE,
        PROGRESSION_GATES_FILE,
        PROGRESSION_SELECTION_GATES_FILE,
        AGGREGATE_FLAG_KEYS_FILE,
        TRADE_PARAMETERS_FILE,
        ITEM_EFFECT_PARAMETERS_FILE,
        AUDIO_DIRECTORY_FILE,
        AUDIO_TONES_FILE,
        AUDIO_SEQUENCES_FILE,
        AUDIO_DRIVER_FILE,
        PALETTE_EFFECT_DISPATCH_FILE,
        PALETTE_PROGRAMS_FILE,
        BLEND_PROGRAMS_FILE,
        TASK_DESCRIPTORS_FILE,
        EFFECT_EVENT_DIRECTORIES_FILE,
        SCRIPT_COMMANDS_FILE,
        SCRIPT_ARCHIVE_FILE,
        SCRIPT_TEXT_FILE,
        SCRIPT_COORDINATES_FILE,
        AUXILIARY_COMMANDS_FILE,
        AUXILIARY_ARCHIVE_FILE,
        MAP_RASTER_PALETTES_FILE,
        UI_CHOICE_LISTS_FILE,
        UNIT_OBJ_INITIAL_POSE_FILE,
        COLOR_MODE_PALETTES_FILE,
        UNIT_OBJ_EQUIPMENT_FILE,
        UNIT_OBJ_NORMAL_BATTLE_FILE,
        UNIT_OBJ_ALTERNATE_BATTLE_FILE,
        UNIT_OBJ_SCRIPT_SPECIAL_NORMAL_FILE,
        UNIT_OBJ_SCRIPT_SPECIAL_ALTERNATE_FILE,
        UNIT_OBJ_SCRIPT_OVERRIDE_FILE,
        UNIT_OBJ_SCRIPT_REGISTRY_SPECIAL_FILE,
        UNIT_OBJ_SCRIPT_REGISTRY_GENERIC_FILE,
        UNIT_OBJ_PROPERTY_WORDS_FILE,
        UNIT_OBJ_BYTE_PROPERTY_WORDS_FILE,
    ]
}

fn report(name: &str, value: serde_json::Value) {
    fs::write(
        root().join(name),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "private EUR e69 newUIlist model/managed/lifecycle; Wine only, Linux/USA pending"]
fn european_ui_property_obj_lists_managed_publication_authentication_repair_and_move() {
    let root = root();
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old_files: Vec<_> = names()[..64]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old_files[0], "prior e68").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (68, 67));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (69, 68)
    );
    assert_eq!(source.manifest.resources.len(), 63);
    assert_eq!(source.manifest.resources[11].sources.len(), 74);
    let offline = dataset();
    extract(&offline, &mut source).unwrap();
    assert_eq!(fs::read_dir(&offline).unwrap().count(), 64);
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(offline.join(n)).unwrap())
        .collect();
    for i in 1..64 {
        if i != 12 {
            assert_eq!(files[i], old_files[i]);
        }
    }
    assert_eq!(source.manifest.coverage, prior.coverage);
    for i in 0..63 {
        if i != 11 {
            assert_eq!(source.manifest.resources[i], prior.resources[i]);
        }
    }
    assert_eq!(
        source.manifest.resources[11].dependencies,
        vec![gba_core::rom_data::OBJ_DIMENSIONS_ID.to_string()]
    );
    assert_eq!(
        source.manifest.resources[11]
            .sources
            .iter()
            .filter(|s| s.offset != NEW[0])
            .collect::<Vec<_>>(),
        prior.resources[11].sources.iter().collect::<Vec<_>>()
    );
    let old = parse_obj_lists(&old_files[12]).unwrap();
    let current = parse_obj_lists(&files[12]).unwrap();
    assert_eq!(old.lists.len(), 73);
    assert_eq!(current.lists.len(), 74);
    assert_ne!(files[12], old_files[12]);
    for list in &old.lists {
        assert!(current.lists.contains(list));
    }
    let extra: Vec<_> = current
        .lists
        .iter()
        .filter(|l| NEW.contains(&l.source_offset))
        .collect();
    assert_eq!(extra.len(), 1);
    assert_eq!(extra.iter().map(|l| l.objects.len()).sum::<usize>(), 2);
    assert_eq!(extra.iter().map(|l| l.source_len()).sum::<usize>(), 14);
    assert_eq!(gba_core::rom_data::obj_lists::REFERENCES.len(), 126);
    for list in extra {
        assert_eq!(list.source_bytes().unwrap(), machine_source(&rom, list));
    }
    let plan: serde_json::Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_UI_PROPERTY_OBJ_PLAN").unwrap()).unwrap(),
    )
    .unwrap();
    let new = current
        .lists
        .iter()
        .find(|l| l.source_offset == NEW[0])
        .unwrap();
    assert_eq!(serde_json::to_value(new).unwrap(), plan["new_list"]);
    assert_eq!(new.source_len(), 14);
    assert_eq!(
        gba_core::ffta_rom::digest(&new.source_bytes().unwrap()),
        plan["new_source_sha256"].as_str().unwrap()
    );
    assert_eq!(
        serde_json::to_value(
            current
                .lists
                .iter()
                .find(|l| l.source_offset == 0x3950b2)
                .unwrap()
        )
        .unwrap(),
        plan["shared_list"]
    );
    let encoded = encode_obj_lists(&current).unwrap();
    assert_eq!(parse_obj_lists(&encoded).unwrap(), current);
    let interrupted = root.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    let mut writes = 0;
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| {
        writes += 1;
        if writes == 12 {
            Err("injected OBJ publication failure".into())
        } else {
            write_synced(p, b)
        }
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    assert!(verify(&interrupted, &source).is_err());
    let managed = root.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e68-d67/generation-retained");
    fs::create_dir_all(&retained).unwrap();
    for (n, b) in names()[..64].iter().zip(&old_files) {
        fs::write(retained.join(n), b).unwrap();
    }
    fs::write(
        retained.parent().unwrap().join("current"),
        b"generation-retained\n",
    )
    .unwrap();
    let first = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(first.action, "created");
    assert!(first.directory.parent().unwrap().ends_with("v1/e69-d68"));
    let current = first.directory.parent().unwrap().join("current");
    let current_bytes = fs::read(&current).unwrap();
    let reused = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(reused.action, "reused");
    assert_eq!(fs::read(&current).unwrap(), current_bytes);
    fs::write(first.directory.join(OBJ_STATIC_LISTS_FILE), b"partial").unwrap();
    let repaired = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(repaired.action, "repaired");
    fs::remove_file(repaired.directory.join(OBJ_STATIC_LISTS_FILE)).unwrap();
    let repaired = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(repaired.action, "repaired");
    let mut machine = crate::make_machine(rom, None);
    let state = machine.save_state().unwrap();
    attach_from_directory(&mut machine, &offline).unwrap();
    let owner = machine.bus.rom_resources().unwrap().clone();
    for case in 0..8 {
        let mut manifest = source.manifest.clone();
        let mut bytes = files[12].clone();
        match case {
            0 => {
                let mut value = parse_obj_lists(&bytes).unwrap();
                value
                    .lists
                    .iter_mut()
                    .find(|l| l.source_offset == NEW[0])
                    .unwrap()
                    .objects[0]
                    .tile ^= 1;
                bytes = encode_obj_lists(&value).unwrap();
                manifest.resources[11].sha256 = gba_core::ffta_rom::digest(&bytes);
            }
            1 => manifest.resources[11]
                .sources
                .retain(|s| s.offset != NEW[0]),
            2 => {
                manifest.resources[11]
                    .sources
                    .iter_mut()
                    .find(|s| s.offset == NEW[0])
                    .unwrap()
                    .length -= 2
            }
            3 => {
                manifest.resources[11]
                    .sources
                    .iter_mut()
                    .find(|s| s.offset == NEW[0])
                    .unwrap()
                    .offset += 2
            }
            4 => manifest.resources[11].dependencies.clear(),
            5 => {
                bytes.pop();
                manifest.resources[11].sha256 = gba_core::ffta_rom::digest(&bytes);
            }
            6 => manifest.decoder_revision -= 1,
            _ => {
                let mut value = parse_obj_lists(&bytes).unwrap();
                let at = value
                    .lists
                    .iter()
                    .position(|l| l.source_offset == NEW[0])
                    .unwrap();
                value.lists.remove(at);
                bytes = encode_obj_lists(&value).unwrap();
                manifest.resources[11].sha256 = gba_core::ffta_rom::digest(&bytes);
            }
        }
        fs::write(offline.join(OBJ_STATIC_LISTS_FILE), bytes).unwrap();
        fs::write(
            offline.join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(attach_from_directory(&mut machine, &offline).is_err());
        assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
        assert_eq!(machine.save_state().unwrap(), state);
    }
    fs::write(offline.join(OBJ_STATIC_LISTS_FILE), &files[12]).unwrap();
    fs::write(offline.join(MANIFEST_FILE), &files[0]).unwrap();
    let relative = repaired
        .directory
        .strip_prefix(&managed)
        .unwrap()
        .to_path_buf();
    let retained_relative = retained.strip_prefix(&managed).unwrap().to_path_buf();
    let moved = root.join("moved resources");
    fs::rename(managed, &moved).unwrap();
    let relocated = preparation::prepare(&moved, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(relocated.action, "reused");
    assert_eq!(relocated.directory, moved.join(relative));
    for (n, b) in names()[..64].iter().zip(&old_files) {
        assert_eq!(
            fs::read(moved.join(&retained_relative).join(n)).unwrap(),
            *b
        );
        assert_eq!(fs::read(prior_dir.join(n)).unwrap(), *b);
    }
    owner_lifecycle();
    report(
        "managed-result.json",
        serde_json::json!({"files":64,"resources":63,"revision":[1,69,68],"references":126,"lists":74,"new_lists":1,"new_objects":2,"new_source_bytes":14,
        "prior_e68_and_62_other_decoded_files_unchanged":true,"offline_json_exact":true,"publication_failure_create_reuse_repair_move":true,
        "coherent_newlist_edit_missing_source_extent_offset_dependency_revision_truncation_and_omission_rejected":true,
        "strict_source_JSON_roundtrip_and_shared_B2":true,"activation_transactional":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}

#[derive(Clone, Copy)]
struct Call {
    offset: u32,
    first: bool,
    flags: u32,
}
fn setup(m: &mut Machine, call: Call) {
    m.cpu.regs[..4].copy_from_slice(&[
        0x08000000 + call.offset,
        100,
        70,
        if call.first { call.flags } else { 7 },
    ]);
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = if call.first { 0x08001500 } else { 0x0800170c };
    m.cpu.cpsr = 0xbf;
    let args = if call.first {
        vec![7, 2, 3, BUFFER, 128, 0]
    } else {
        vec![2, 3, BUFFER, 128, 0]
    };
    for (i, value) in args.into_iter().enumerate() {
        m.bus.write32(STACK + i as u32 * 4, value);
    }
    for at in 0..128 * 8 {
        m.bus.write8(BUFFER + at, 0x5a);
    }
}
fn interpreted(m: &mut Machine, call: Call, visited: &mut BTreeSet<(u32, bool)>) -> usize {
    setup(m, call);
    for steps in 1..=20_000 {
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
        if m.cpu.regs[15] == RETURN {
            assert_eq!(m.cpu.regs[13], STACK);
            return steps;
        }
    }
    panic!("original OBJ call did not return");
}
fn ranges(m: &mut Machine, sources: &[gba_core::rom_data::SourceRange], rom: Option<&[u8]>) {
    for s in sources {
        let at = s.offset as usize;
        let end = at + s.length as usize;
        if let Some(rom) = rom {
            m.bus.rom[at..end].copy_from_slice(&rom[at..end]);
        } else {
            m.bus.rom[at..end].fill(0xcd);
        }
    }
}
thread_local! { static NATIVE_BYTES: Cell<u64> = const { Cell::new(0) }; }
fn used(m: *mut std::ffi::c_void) -> u64 {
    unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .obj_list_stats()
            .unwrap()
            .source_bytes
    }
}
extern "C" fn read8(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let before = used(m);
    let r = (RT_API.read8)(m, a);
    NATIVE_BYTES.with(|n| n.set(n.get() + used(m) - before));
    r
}
extern "C" fn read16(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let before = used(m);
    let r = (RT_API.read16)(m, a);
    NATIVE_BYTES.with(|n| n.set(n.get() + used(m) - before));
    r
}
extern "C" fn read32(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let before = used(m);
    let r = (RT_API.read32)(m, a);
    NATIVE_BYTES.with(|n| n.set(n.get() + used(m) - before));
    r
}

#[test]
#[ignore = "private EUR e69 newUIlist native/host/bus; Wine only, Linux/USA pending"]
fn european_ui_property_obj_lists_original_native_host_provenance_and_bus_edges() {
    let output = root().join("native");
    assert!(!output.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let extra_sources: Vec<_> = files.manifest.resources[11]
        .sources
        .iter()
        .filter(|s| NEW.contains(&s.offset))
        .cloned()
        .collect();
    let sources = &extra_sources;
    let calls: Vec<_> = NEW
        .into_iter()
        .flat_map(|offset| {
            [
                Call {
                    offset,
                    first: true,
                    flags: 0x30000400,
                },
                Call {
                    offset,
                    first: false,
                    flags: 0,
                },
            ]
        })
        .collect();
    let mut tracer = crate::make_machine(rom.clone(), None);
    tracer.bus.ffta_sprites.enabled = false;
    let start = tracer.save_state().unwrap();
    let mut visited = BTreeSet::new();
    let mut instructions = 0;
    for &call in &calls {
        instructions += interpreted(&mut tracer, call, &mut visited);
    }
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks = visited
        .iter()
        .map(|&(address, thumb)| {
            let mut instrs = vec![if thumb {
                armv4t::decode_thumb(view.read16(address), address)
            } else {
                armv4t::decode_arm(view.read32(address), address)
            }];
            if thumb {
                let next = armv4t::decode_thumb(view.read16(address + 2), address + 2);
                if armv4t::fuse_thumb_bl(&instrs[0], &next).is_some() {
                    instrs.push(next);
                }
            }
            Block {
                addr: address,
                thumb,
                instrs,
            }
        })
        .collect();
    fs::create_dir_all(&output).unwrap();
    let object = output.join("obj-lists.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut legacy = crate::make_machine(rom.clone(), None);
    let mut decoded = crate::make_machine(rom.clone(), None);
    let mut native = crate::make_machine(rom.clone(), None);
    for m in [&mut decoded, &mut native] {
        attach_from_directory(m, &dataset()).unwrap();
    }
    for m in [&mut legacy, &mut decoded, &mut native] {
        m.load_state(&start).unwrap();
    }
    let api = gba_core::capi::RtApi {
        read8,
        read16,
        read32,
        ..RT_API
    };
    NATIVE_BYTES.with(|n| n.set(0));
    let mut dispatches = 0;
    for &call in &calls {
        interpreted(&mut legacy, call, &mut BTreeSet::new());
        for m in [&mut decoded, &mut native] {
            ranges(m, sources, None);
        }
        interpreted(&mut decoded, call, &mut BTreeSet::new());
        setup(&mut native, call);
        while native.cpu.regs[15] != RETURN {
            assert!(dispatches < 100_000);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                .expect("original instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatches += 1;
        }
        for m in [&mut decoded, &mut native] {
            ranges(m, sources, Some(&rom));
            assert_eq!(
                legacy.save_state().unwrap(),
                m.save_state().unwrap(),
                "all state/clock OBJ{:x}",
                call.offset
            );
        }
    }
    let compiled = NATIVE_BYTES.with(Cell::get);
    assert!(compiled > 0);
    assert!(NEW.iter().all(|&at| native
        .bus
        .rom_resources()
        .unwrap()
        .obj_list_source_used(at)
        .unwrap()
        .0));
    // New14B range, both boundaries, widths/mirrors and neighbouring existing lists.
    ranges(&mut decoded, sources, None);
    for real_bios in [false, true] {
        decoded.bus.real_bios = real_bios;
        for source in sources {
            for at in source.offset - 3..source.offset + source.length + 3 {
                for mirror in [0x08000000, 0x0a000000, 0x0c000000] {
                    assert_eq!(decoded.bus.read8(mirror + at), rom[at as usize]);
                    assert_eq!(
                        decoded.bus.read16(mirror + at),
                        u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap())
                    );
                    assert_eq!(
                        (RT_API.read32)(
                            &mut decoded as *mut Machine as *mut std::ffi::c_void,
                            mirror + at
                        ),
                        u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
                    );
                }
            }
        }
    }
    ranges(&mut decoded, sources, Some(&rom));
    decoded.bus.real_bios = false;
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.bus.write32(0x040000d4, 0x083950a4);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 8);
    }
    ranges(&mut decoded, sources, None);
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    ranges(&mut decoded, sources, Some(&rom));
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    for m in [&mut legacy, &mut decoded] {
        m.cpu.regs[..3].copy_from_slice(&[0x0a3950a4, 0x02032400, 8]);
        m.cpu.regs[15] = 0x02000000;
        m.cpu.cpsr = 0xbf;
        m.bus.write16(0x02000000, 0xdf0b);
    }
    ranges(&mut decoded, sources, None);
    legacy.step();
    decoded.step();
    ranges(&mut decoded, sources, Some(&rom));
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    // A word spanning the new list and the existing B2list uses both typed records.
    let mut mixed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut mixed, &dataset()).unwrap();
    ranges(&mut mixed, sources, None);
    assert_eq!(
        mixed
            .bus
            .rom_resources()
            .unwrap()
            .obj_list_source_used(NEW[0]),
        Some((false, false))
    );
    assert_eq!(
        mixed
            .bus
            .rom_resources()
            .unwrap()
            .obj_list_source_used(0x3950b2),
        Some((false, false))
    );
    assert_eq!(
        (RT_API.read32)(
            &mut mixed as *mut Machine as *mut std::ffi::c_void,
            0x083950b0
        ),
        u32::from_le_bytes(rom[0x3950b0..0x3950b4].try_into().unwrap())
    );
    assert_eq!(
        mixed
            .bus
            .rom_resources()
            .unwrap()
            .obj_list_source_used(NEW[0]),
        Some((true, false))
    );
    assert_eq!(
        mixed
            .bus
            .rom_resources()
            .unwrap()
            .obj_list_source_used(0x3950b2),
        Some((true, false))
    );
    // FastSet covers4prior+14new+14B2bytes, without widening the new source owner.
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.cpu.regs[..3].copy_from_slice(&[0x0a3950a0, 0x02032400, 8]);
        m.cpu.regs[15] = 0x02000000;
        m.cpu.cpsr = 0xbf;
        m.bus.write16(0x02000000, 0xdf0c);
    }
    ranges(&mut decoded, sources, None);
    legacy.step();
    decoded.step();
    ranges(&mut decoded, sources, Some(&rom));
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    // Exercise actual host list decoding/publication with all raw lists poisoned.
    gba_core::mem::set_extra(120);
    let mut host_checks = 0;
    for &call in &calls {
        let mut old = crate::make_machine(rom.clone(), None);
        let mut new = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut new, &dataset()).unwrap();
        ranges(&mut new, sources, None);
        for m in [&mut old, &mut new] {
            setup(m, call);
            m.bus.ffta_sprites.enabled = true;
            m.ffta_sprite_entry();
        }
        // Original instructions supply only the central output to publication.
        let mut reference = crate::make_machine(rom.clone(), None);
        interpreted(&mut reference, call, &mut BTreeSet::new());
        let oam: Vec<_> = (0..1024)
            .map(|at| reference.bus.read8(BUFFER + at))
            .collect();
        for m in [&mut old, &mut new] {
            m.bus
                .ffta_sprites
                .publish(BUFFER, 0x07000000, 1024, true, &oam);
        }
        assert!(old.bus.ffta_sprites.active.is_some());
        assert_eq!(
            format!("{:?}", old.bus.ffta_sprites.active),
            format!("{:?}", new.bus.ffta_sprites.active)
        );
        assert!(
            new.bus
                .rom_resources()
                .unwrap()
                .obj_list_stats()
                .unwrap()
                .host_reads
                > 0
        );
        assert_eq!(
            new.bus
                .rom_resources()
                .unwrap()
                .obj_list_source_used(call.offset),
            Some((false, true))
        );
        host_checks += 1;
    }
    gba_core::mem::set_extra(0);
    report(
        "native-result.json",
        serde_json::json!({"calls":calls.len(),"original_instructions":instructions,"native_dispatches":dispatches,
        "compiled_source_bytes":compiled,"poisoned_ranges":1,"states_and_clocks_equal":true,"all_edges_widths_mirrors_real_bios_bus":true,
        "dma_cpuset_fastset_equal":true,"mixed_API_new2_B2count2":true,"DMA_CpuSet_new14_B2count2":true,"FastSet_prior4_new14_B2all14":true,"host_decode_publication_checks":host_checks,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}

fn owner_lifecycle() {
    let temp = tempfile::tempdir().unwrap();
    let copy = temp.path().join("dataset");
    fs::create_dir(&copy).unwrap();
    for n in names() {
        fs::copy(dataset().join(n), copy.join(n)).unwrap();
    }
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let mut m = crate::make_machine(rom.clone(), None);
    let boot = m.save_state().unwrap();
    attach_from_directory(&mut m, &copy).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    fs::rename(&copy, temp.path().join("unavailable")).unwrap();
    assert!(attach_from_directory(&mut m, &copy).is_err());
    assert_eq!(m.save_state().unwrap(), boot);
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    let save = temp.path().join("test.sav");
    let mut slots = crate::save_state::Slots::new(&save, &m.bus.rom);
    slots.save(&m, 1, &vec![[0; 4]; 240 * 160], false).unwrap();
    m.cpu.regs[4] = 99;
    let changed = m.save_state().unwrap();
    slots.load(&mut m, 1).unwrap();
    assert_eq!(m.cpu.regs[4], 0);
    slots.undo(&mut m).unwrap();
    assert_eq!(m.save_state().unwrap(), changed);
    let len = m.bus.save_data().unwrap().len();
    fs::write(&save, vec![7; len]).unwrap();
    let backups = crate::save_backups::Backups::new(&save);
    backups.capture_startup().unwrap();
    backups
        .restore(backups.choices(len)[0].generation.unwrap(), &mut m, &boot)
        .unwrap();
    m.bus.load_save_data(&vec![9; len]);
    crate::save_backups::restart(&mut m, &boot, &save).unwrap();
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    assert_eq!(fs::read(&save).unwrap(), vec![9; len]);
    let files = read_dataset(&dataset()).unwrap();
    let extra_sources: Vec<_> = files.manifest.resources[11]
        .sources
        .iter()
        .filter(|s| NEW.contains(&s.offset))
        .cloned()
        .collect();
    let sources = &extra_sources;
    ranges(&mut m, sources, None);
    for s in sources {
        assert_eq!(m.bus.read8(0x08000000 + s.offset), rom[s.offset as usize]);
        assert_eq!(
            m.bus.read8(0x08000000 + s.offset + 2),
            rom[s.offset as usize + 2]
        );
    }
    let stats = owner.obj_list_stats().unwrap();
    assert!(stats.count_bytes > 0 && stats.attribute_bytes > 0);
    report(
        "lifecycle-result.json",
        serde_json::json!({"temporary_files_unavailable":true,"activation_transactional":true,
        "load_undo_backup_restart_owner_retained":true,"count_and_attribute_use_positive":true,"linux":"pending","usa":"pending"}),
    );
}
