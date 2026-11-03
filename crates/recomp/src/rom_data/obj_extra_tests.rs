//! Reduced EUR managed, original/native, host and lifecycle acceptance.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
const NEW: [u32; 7] = [
    0x394f34, 0x3950fa, 0x395102, 0x399628, 0x3adfc4, 0x3adfd2, 0x3adfda,
];
fn machine_source(rom: &[u8], list: &gba_core::rom_data::obj_lists::List) -> Vec<u8> {
    rom[list.source_offset as usize..list.source_offset as usize + list.source_len()].to_vec()
}
const RETURN: u32 = 0x08000300;
const STACK: u32 = 0x03007c00;
const BUFFER: u32 = 0x02030010;
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_OBJ_EXTRA_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 14] {
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
#[ignore = "exact EUR ROM/e15 dataset/new extra roots and absent new root; Linux/USA pending"]
fn european_extra_obj_lists_managed_publication_authentication_repair_and_move() {
    let root = root();
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old_files: Vec<_> = names()[..14]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old_files[0], "prior e15").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (15, 14));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (16, 15)
    );
    assert_eq!(source.manifest.resources.len(), 13);
    assert_eq!(source.manifest.resources[11].sources.len(), 73);
    let offline = dataset();
    extract(&offline, &mut source).unwrap();
    assert_eq!(fs::read_dir(&offline).unwrap().count(), 14);
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(offline.join(n)).unwrap())
        .collect();
    for i in 1..14 {
        if i != 12 {
            assert_eq!(files[i], old_files[i]);
        }
    }
    let old = parse_obj_lists(&old_files[12]).unwrap();
    let current = parse_obj_lists(&files[12]).unwrap();
    assert_eq!(old.lists.len(), 66);
    assert_eq!(current.lists.len(), 73);
    assert_ne!(files[12], old_files[12]);
    for list in &old.lists {
        assert!(current.lists.contains(list));
    }
    let extra: Vec<_> = current
        .lists
        .iter()
        .filter(|l| NEW.contains(&l.source_offset))
        .collect();
    assert_eq!(extra.len(), 7);
    assert_eq!(extra.iter().map(|l| l.objects.len()).sum::<usize>(), 13);
    assert_eq!(extra.iter().map(|l| l.source_len()).sum::<usize>(), 92);
    assert_eq!(gba_core::rom_data::obj_lists::REFERENCES.len(), 123);
    for list in extra {
        assert_eq!(list.source_bytes().unwrap(), machine_source(&rom, list));
    }
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
        .join("v1/e15-d14/generation-retained");
    fs::create_dir_all(&retained).unwrap();
    for (n, b) in names()[..14].iter().zip(&old_files) {
        fs::write(retained.join(n), b).unwrap();
    }
    fs::write(
        retained.parent().unwrap().join("current"),
        b"generation-retained\n",
    )
    .unwrap();
    let first = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(first.action, "created");
    assert!(first.directory.parent().unwrap().ends_with("v1/e16-d15"));
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
            1 => manifest.resources[11].path = "../outside.json".into(),
            2 => {
                manifest.resources.remove(11);
                manifest.coverage.remove(11);
            }
            3 => manifest.resources[11].sources[0].offset += 1,
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
    let mut whitespace = files[12].clone();
    whitespace.extend_from_slice(b"\n \n");
    let mut manifest = source.manifest.clone();
    manifest.resources[11].sha256 = gba_core::ffta_rom::digest(&whitespace);
    fs::write(offline.join(OBJ_STATIC_LISTS_FILE), whitespace).unwrap();
    fs::write(
        offline.join(MANIFEST_FILE),
        toml::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    attach_from_directory(&mut machine, &offline).unwrap();
    assert_ne!(
        owner.dataset_sha256(),
        machine.bus.rom_resources().unwrap().dataset_sha256()
    );
    fs::File::options()
        .write(true)
        .open(offline.join(OBJ_STATIC_LISTS_FILE))
        .unwrap()
        .set_len(MAX_OBJ_STATIC_LISTS_FILE_BYTES + 1)
        .unwrap();
    assert!(attach_from_directory(&mut machine, &offline).is_err());
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
    for (n, b) in names()[..14].iter().zip(&old_files) {
        assert_eq!(
            fs::read(moved.join(&retained_relative).join(n)).unwrap(),
            *b
        );
        assert_eq!(fs::read(prior_dir.join(n)).unwrap(), *b);
    }
    report(
        "managed-result.json",
        serde_json::json!({"files":14,"resources":13,"revision":[1,16,15],"lists":73,"new_lists":7,"new_objects":13,"new_source_bytes":92,
        "prior_e15_and_other_decoded_files_unchanged":true,"offline_json_exact":true,"publication_failure_create_reuse_repair_move":true,
        "coherent_edit_path_coverage_provenance_dependency_revision_truncation_and_input_guard_rejected":true,
        "whole_set_identity":true,"activation_transactional":true,"linux":"pending","usa":"pending","performance_measurements":false}),
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
#[ignore = "exact EUR ROM/current OBJ dataset/TinyCC; reduced Wine, Linux/USA pending"]
fn european_extra_obj_lists_original_native_host_provenance_and_bus_edges() {
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
    // All seven new ranges, mirrors, widths and neighbouring prior-list bytes.
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
        m.bus.write32(0x040000d4, 0x08394f34);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 7);
    }
    ranges(&mut decoded, sources, None);
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    ranges(&mut decoded, sources, Some(&rom));
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    for m in [&mut legacy, &mut decoded] {
        m.cpu.regs[..3].copy_from_slice(&[0x0a394f34, 0x02032400, 7]);
        m.cpu.regs[15] = 0x02000000;
        m.cpu.cpsr = 0xbf;
        m.bus.write16(0x02000000, 0xdf0b);
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
        "compiled_source_bytes":compiled,"poisoned_ranges":7,"states_and_clocks_equal":true,"all_edges_widths_mirrors_real_bios_bus":true,
        "dma_cpuset_equal":true,"host_decode_publication_checks":host_checks,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}

#[test]
#[ignore = "exact EUR ROM/new dataset; unavailable temporary copies only, Linux/USA pending"]
fn european_extra_obj_lists_owner_survives_slots_undo_backup_restart_and_missing_files() {
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
