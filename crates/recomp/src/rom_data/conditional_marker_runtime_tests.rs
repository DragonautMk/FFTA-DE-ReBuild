//! Focused EUR publication and original-reader checks for conditional OBJ markers.
use super::*;
use gba_core::{rom_data::obj_lists::conditional_markers::ROOTS, Bus, Machine};
use std::sync::Arc;

const CONTEXT: u32 = 0x0202_0000;
const STACK: u32 = 0x0300_7c00;
const RETURN: u32 = 0x0800_0300;

pub(super) fn assert_previous_resource_metadata(
    current: &gba_core::rom_data::Resource,
    previous: &gba_core::rom_data::Resource,
) {
    if current.id != OBJ_STATIC_LISTS_ID {
        assert_eq!(current, previous);
        return;
    }
    let mut projected = current.clone();
    for (offset, digest) in ROOTS {
        let source = projected.sources.iter().find(|source| source.offset == offset).unwrap();
        assert_eq!((source.length, source.sha256.as_str()), (8, digest));
    }
    projected.sources.retain(|source| !ROOTS.iter().any(|(offset, _)| *offset == source.offset));
    projected.sha256 = previous.sha256.clone();
    assert_eq!(&projected, previous);
}

pub(super) fn assert_previous_resource_files(
    current: &gba_core::rom_data::Resource,
    previous: &gba_core::rom_data::Resource,
    current_dir: &Path,
    previous_dir: &Path,
) {
    assert_previous_resource_metadata(current, previous);
    let current_bytes = fs::read(current_dir.join(&current.path)).unwrap();
    let previous_bytes = fs::read(previous_dir.join(&previous.path)).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&current_bytes), current.sha256);
    assert_eq!(gba_core::ffta_rom::digest(&previous_bytes), previous.sha256);
    if current.id == OBJ_STATIC_LISTS_ID {
        let mut projected = parse_obj_lists(&current_bytes).unwrap();
        projected.lists.retain(|list| !ROOTS.iter().any(|(offset, _)| *offset == list.source_offset));
        assert_eq!(projected, parse_obj_lists(&previous_bytes).unwrap());
    } else {
        assert_eq!(current_bytes, previous_bytes);
    }
}

fn setup(machine: &mut Machine, offset: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.bus.write32(0x0200_f444, CONTEXT);
    machine.bus.write8(CONTEXT + 0xe, 1);
    machine.bus.write8(0x0200_3c32, 1);
    machine.bus.write16(CONTEXT + 0x47c, 100);
    machine.bus.write16(CONTEXT + 0x47e, 80);
    machine.bus.write32(CONTEXT + 0x480, 0x0800_0000 + offset);
    machine.bus.iwram[0x2c58..0x2dd8].fill(0x5a);
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = 0x0802_f3c0;
    machine.cpu.cpsr = 0xbf;
}

fn run(machine: &mut Machine) {
    for _ in 0..20_000 {
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == RETURN {
            return;
        }
    }
    panic!("original conditional marker consumer did not return");
}

#[test]
#[ignore = "private exact EUR ROM and e128 predecessor; Windows executable under Wine"]
fn european_conditional_markers_publication_and_managed_original_reads() {
    let root = PathBuf::from(std::env::var_os("FFTA_MARKER_RUNTIME_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old_manifest: Manifest =
        toml::from_str(&fs::read_to_string(old_dir.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (
            old_manifest.extractor_revision,
            old_manifest.decoder_revision,
            old_manifest.resources.len()
        ),
        (128, 127, 100)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (129, 128, 100)
    );
    assert_eq!(source.manifest.coverage, old_manifest.coverage);
    let directory = root.join("offline dataset");
    extract(&directory, &mut source).unwrap();
    verify(&directory, &source).unwrap();
    assert_eq!(fs::read_dir(&directory).unwrap().count(), 101);
    let old_obj = old_manifest
        .resources
        .iter()
        .find(|r| r.id == OBJ_STATIC_LISTS_ID)
        .unwrap();
    let new_obj = source
        .manifest
        .resources
        .iter()
        .find(|r| r.id == OBJ_STATIC_LISTS_ID)
        .unwrap();
    let mut old_projection = new_obj.clone();
    old_projection
        .sources
        .retain(|s| !ROOTS.iter().any(|(offset, _)| *offset == s.offset));
    old_projection.sha256 = old_obj.sha256.clone();
    assert_eq!(&old_projection, old_obj);
    for previous in &old_manifest.resources {
        let old_bytes = fs::read(old_dir.join(&previous.path)).unwrap();
        assert_eq!(gba_core::ffta_rom::digest(&old_bytes), previous.sha256);
        let current = source
            .manifest
            .resources
            .iter()
            .find(|r| r.id == previous.id)
            .unwrap();
        if previous.id != OBJ_STATIC_LISTS_ID {
            assert_eq!(current, previous);
            assert_eq!(fs::read(directory.join(&current.path)).unwrap(), old_bytes);
        }
    }
    let old_lists =
        parse_obj_lists(&fs::read(old_dir.join(OBJ_STATIC_LISTS_FILE)).unwrap()).unwrap();
    let bytes = fs::read(directory.join(OBJ_STATIC_LISTS_FILE)).unwrap();
    let all_lists = parse_obj_lists(&bytes).unwrap();
    let mut previous_lists = all_lists.clone();
    previous_lists
        .lists
        .retain(|l| !ROOTS.iter().any(|(offset, _)| *offset == l.source_offset));
    assert_eq!(previous_lists, old_lists);
    for (offset, digest) in ROOTS {
        let list = all_lists
            .lists
            .iter()
            .find(|l| l.source_offset == offset)
            .unwrap();
        assert_eq!(list.objects.len(), 1);
        let span = new_obj.sources.iter().find(|s| s.offset == offset).unwrap();
        assert_eq!((span.length, span.sha256.as_str()), (8, digest));
        assert_eq!(
            list.source_bytes().unwrap(),
            rom[offset as usize..offset as usize + 8]
        );
    }

    let managed_root = root.join("managed");
    let created = preparation::prepare(&managed_root, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(created.action, "created");
    assert!(created
        .directory
        .parent()
        .unwrap()
        .ends_with("v1/e129-d128"));
    assert_eq!(
        preparation::prepare(&managed_root, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(created.directory.join(OBJ_STATIC_LISTS_FILE), b"partial").unwrap();
    assert_eq!(
        preparation::prepare(&managed_root, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "repaired"
    );

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut decoded = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut decoded, &directory).unwrap();
    let owner = decoded.bus.rom_resources().unwrap().clone();
    for (offset, _) in ROOTS {
        setup(&mut raw, offset);
        setup(&mut decoded, offset);
        let old_count = owner.obj_list_stats().unwrap().source_bytes;
        decoded.bus.rom[offset as usize..offset as usize + 8].fill(0xcd);
        run(&mut raw);
        run(&mut decoded);
        assert!(owner.obj_list_stats().unwrap().source_bytes > old_count);
        assert_eq!(owner.obj_list_source_used(offset), Some((true, false)));
        decoded.bus.rom[offset as usize..offset as usize + 8]
            .copy_from_slice(&rom[offset as usize..offset as usize + 8]);
        assert_eq!(
            raw.save_state().unwrap(),
            decoded.save_state().unwrap(),
            "full state/clock for marker {offset:x}"
        );
    }
    for (offset, _) in ROOTS {
        decoded.bus.rom[offset as usize..offset as usize + 8].fill(0xcd);
    }
    for (offset, _) in ROOTS {
        for at in offset - 3..offset + 11 {
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!(decoded.bus.read8(mirror + at), rom[at as usize]);
                assert_eq!(
                    decoded.bus.read16(mirror + at),
                    u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap())
                );
                assert_eq!(
                    decoded.bus.read32(mirror + at),
                    u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
                );
            }
        }
    }
    for (offset, _) in ROOTS {
        decoded.bus.rom[offset as usize..offset as usize + 8]
            .copy_from_slice(&rom[offset as usize..offset as usize + 8]);
    }
    let state = decoded.save_state().unwrap();
    let manifest = fs::read(directory.join(MANIFEST_FILE)).unwrap();
    let mut altered = all_lists.clone();
    altered
        .lists
        .iter_mut()
        .find(|l| l.source_offset == ROOTS[0].0)
        .unwrap()
        .objects[0]
        .tile ^= 1;
    let bad_bytes = encode_obj_lists(&altered).unwrap();
    let mut bad_manifest = source.manifest.clone();
    bad_manifest
        .resources
        .iter_mut()
        .find(|r| r.id == OBJ_STATIC_LISTS_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    fs::write(directory.join(OBJ_STATIC_LISTS_FILE), bad_bytes).unwrap();
    fs::write(
        directory.join(MANIFEST_FILE),
        toml::to_string_pretty(&bad_manifest).unwrap(),
    )
    .unwrap();
    assert!(attach_from_directory(&mut decoded, &directory).is_err());
    assert!(Arc::ptr_eq(decoded.bus.rom_resources().unwrap(), &owner));
    assert_eq!(decoded.save_state().unwrap(), state);
    fs::write(directory.join(OBJ_STATIC_LISTS_FILE), bytes).unwrap();
    fs::write(directory.join(MANIFEST_FILE), manifest).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[1,129,128],"resources":100,"files":101,"prior_other_resources_unchanged":99,"old_obj_lists_and_sources_preserved":true,"new_lists":2,"original_slot_consumer_interpreter_full_state_clock_pairs":2,"mixed_boundary_widths_mirrors":true,"create_reuse_repair":true,"coherent_tamper_transaction":true,"ordinary_scene":false,"translated_native":"pending","performance_measurements":false})).unwrap()).unwrap();
}

#[test]
#[ignore = "private exact EUR ROM and TinyCC; Windows executable under Wine"]
fn european_conditional_markers_original_translated_managed_reads() {
    use gba_core::capi::RT_API;
    use recomp_core::analyze::{Analysis, Block, View};
    use std::collections::BTreeSet;

    let root = PathBuf::from(std::env::var_os("FFTA_MARKER_NATIVE_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (129, 128)
    );
    let directory = root.join("offline dataset");
    extract(&directory, &mut source).unwrap();

    // Trace only the instructions actually reached by the two selected roots.
    let mut trace = crate::make_machine(rom.clone(), None);
    let initial = trace.save_state().unwrap();
    let mut visited = BTreeSet::new();
    for (offset, _) in ROOTS {
        trace.load_state(&initial).unwrap();
        setup(&mut trace, offset);
        for _ in 0..20_000 {
            visited.insert((trace.cpu.regs[15], trace.cpu.thumb()));
            trace.bus.frame_ready = false;
            trace.step();
            if trace.cpu.regs[15] == RETURN {
                break;
            }
        }
        assert_eq!(trace.cpu.regs[15], RETURN);
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
            let mut instrs = vec![first];
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
    let object = root.join("marker-native.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().expect("TinyCC required"),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut native = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut native, &directory).unwrap();
    let owner = native.bus.rom_resources().unwrap().clone();
    let mut total_bytes = 0;
    for (offset, _) in ROOTS {
        raw.load_state(&initial).unwrap();
        native.load_state(&initial).unwrap();
        setup(&mut raw, offset);
        setup(&mut native, offset);
        let before = owner.obj_list_stats().unwrap().source_bytes;
        native.bus.rom[offset as usize..offset as usize + 8].fill(0xcd);
        run(&mut raw);
        let mut dispatches = 0;
        while native.cpu.regs[15] != RETURN {
            assert!(dispatches < 20_000, "translated consumer did not return");
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                .expect("traced original instruction compiled")(
                &RT_API,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatches += 1;
        }
        let used = owner.obj_list_stats().unwrap().source_bytes - before;
        assert!(
            used > 0,
            "translated path did not read the selected managed source"
        );
        total_bytes += used;
        assert_eq!(owner.obj_list_source_used(offset), Some((true, false)));
        native.bus.rom[offset as usize..offset as usize + 8]
            .copy_from_slice(&rom[offset as usize..offset as usize + 8]);
        assert_eq!(
            raw.save_state().unwrap(),
            native.save_state().unwrap(),
            "full state/clock for translated marker {offset:x}"
        );
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[1,129,128],"original_slot_consumer_translated_full_state_clock_pairs":2,"translated_managed_source_bytes":total_bytes,"traced_instruction_addresses":visited.len(),"both_exact_roots_observed":true,"ordinary_scene":false,"performance_measurements":false})).unwrap()).unwrap();
}
