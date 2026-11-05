//! EUR publication and original initializer checks for the layered OBJ resource.
use super::*;
use gba_core::{Bus, Machine};
use std::sync::Arc;

const TASK: u32 = 0x0201_0000;
const STACK: u32 = 0x0300_7c00;
const RETURN: u32 = 0x0800_0300;

fn setup(machine: &mut Machine, descriptor: &gba_core::rom_data::layered_obj_animations::Descriptor) {
    machine.bus.ffta_sprites.enabled = false;
    machine.bus.ewram.fill(0xa5);
    machine.bus.iwram.fill(0xc3);
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = 0x0814_2590;
    machine.cpu.cpsr = 0xbf;
    machine.cpu.regs[..4].copy_from_slice(&[
        TASK, 0x0800_0000 + descriptor.object_root, 0x0800_0000 + descriptor.sequence_root,
        u32::from_le_bytes(descriptor.opaque[4..8].try_into().unwrap()),
    ]);
    machine.bus.write32(STACK, u32::from(descriptor.opaque[14]));
}

fn run(machine: &mut Machine) {
    for _ in 0..100_000 {
        if machine.cpu.regs[15] == RETURN { return; }
        machine.bus.frame_ready = false;
        machine.step();
    }
    panic!("original layered OBJ initializer did not return");
}

#[test]
#[ignore = "private EUR ROM and authenticated E129 dataset; Windows under Wine"]
fn european_layered_obj_publication_and_managed_original_reads() {
    let root = PathBuf::from(std::env::var_os("FFTA_LAYERED_RUNTIME_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Manifest = toml::from_str(&fs::read_to_string(old_dir.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (129, 128, 100));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (130, 129, 101));
    let directory = root.join("offline dataset");
    extract(&directory, &mut source).unwrap();
    verify(&directory, &source).unwrap();
    assert_eq!(fs::read_dir(&directory).unwrap().count(), 102);
    assert_eq!(&source.manifest.coverage[..100], old.coverage.as_slice());
    for (current, previous) in source.manifest.resources[..100].iter().zip(&old.resources) {
        assert_eq!(current, previous);
        let bytes = fs::read(directory.join(&current.path)).unwrap();
        assert_eq!(gba_core::ffta_rom::digest(&bytes), previous.sha256);
        assert_eq!(bytes, fs::read(old_dir.join(&previous.path)).unwrap());
    }
    let managed_root = root.join("managed");
    let created = preparation::prepare(&managed_root, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(created.action, "created");
    assert!(created.directory.parent().unwrap().ends_with("v1/e130-d129"));
    assert_eq!(preparation::prepare(&managed_root, &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    fs::write(created.directory.join(LAYERED_OBJ_ANIMATIONS_FILE), b"partial").unwrap();
    assert_eq!(preparation::prepare(&managed_root, &mut source, &mut |_| Ok(())).unwrap().action, "repaired");

    let model = &source.layered_obj_animations.as_ref().unwrap().decoded;
    let spans = model.source_spans().unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut decoded = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut decoded, &directory).unwrap();
    let owner = decoded.bus.rom_resources().unwrap().clone();
    for (offset, bytes, _) in &spans {
        decoded.bus.rom[*offset as usize..*offset as usize + bytes.len()].fill(0xcd);
    }
    for (offset, bytes, _) in &spans {
        for (i, byte) in bytes.iter().enumerate() {
            assert_eq!(decoded.bus.read8(0x0800_0000 + *offset + i as u32), *byte);
        }
    }
    for descriptor in &model.descriptors {
        setup(&mut raw, descriptor);
        setup(&mut decoded, descriptor);
        run(&mut raw);
        run(&mut decoded);
        for (offset, bytes, _) in &spans {
            decoded.bus.rom[*offset as usize..*offset as usize + bytes.len()].copy_from_slice(bytes);
        }
        assert_eq!(raw.save_state().unwrap(), decoded.save_state().unwrap());
        for (offset, bytes, _) in &spans {
            decoded.bus.rom[*offset as usize..*offset as usize + bytes.len()].fill(0xcd);
        }
    }
    let stats = owner.layered_obj_animations_stats().unwrap();
    assert!(stats.descriptor_bytes > 0 && stats.fields.source_bytes > 0);
    for (offset, bytes, _) in &spans {
        decoded.bus.rom[*offset as usize..*offset as usize + bytes.len()].copy_from_slice(bytes);
    }
    let state = decoded.save_state().unwrap();
    let data = fs::read(directory.join(LAYERED_OBJ_ANIMATIONS_FILE)).unwrap();
    let manifest = fs::read(directory.join(MANIFEST_FILE)).unwrap();
    let mut altered = model.clone();
    altered.descriptors[0].opaque[0] ^= 1;
    let bad = encode_layered_obj_animations(&altered).unwrap();
    let mut bad_manifest = source.manifest.clone();
    bad_manifest.resources[100].sha256 = gba_core::ffta_rom::digest(&bad);
    fs::write(directory.join(LAYERED_OBJ_ANIMATIONS_FILE), bad).unwrap();
    fs::write(directory.join(MANIFEST_FILE), toml::to_string_pretty(&bad_manifest).unwrap()).unwrap();
    assert!(attach_from_directory(&mut decoded, &directory).is_err());
    assert!(Arc::ptr_eq(decoded.bus.rom_resources().unwrap(), &owner));
    assert_eq!(decoded.save_state().unwrap(), state);
    fs::write(directory.join(LAYERED_OBJ_ANIMATIONS_FILE), data).unwrap();
    fs::write(directory.join(MANIFEST_FILE), manifest).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[130,129],"resources":101,"files":102,"unchanged_prior_resources":100,"source_spans":633,"source_bytes":20474,"original_initializer_state_pairs":16,"create_reuse_repair":true,"coherent_tamper_transaction":true,"ordinary_scene":false,"translated_native":"pending","performance_measurements":false})).unwrap()).unwrap();
}

#[test]
#[ignore = "private exact EUR ROM and TinyCC; Windows executable under Wine"]
fn european_layered_obj_original_translated_managed_initializers() {
    use gba_core::capi::RT_API;
    use recomp_core::analyze::{Analysis, Block, View};
    use std::collections::BTreeSet;

    let root = PathBuf::from(std::env::var_os("FFTA_LAYERED_NATIVE_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (130, 129));
    let directory = root.join("offline dataset");
    extract(&directory, &mut source).unwrap();
    let model = &source.layered_obj_animations.as_ref().unwrap().decoded;
    let spans = model.source_spans().unwrap();
    let mut trace = crate::make_machine(rom.clone(), None);
    let initial = trace.save_state().unwrap();
    let mut visited = BTreeSet::new();
    for descriptor in &model.descriptors {
        trace.load_state(&initial).unwrap();
        setup(&mut trace, descriptor);
        for _ in 0..100_000 {
            visited.insert((trace.cpu.regs[15], trace.cpu.thumb()));
            trace.bus.frame_ready = false;
            trace.step();
            if trace.cpu.regs[15] == RETURN { break; }
        }
        assert_eq!(trace.cpu.regs[15], RETURN);
    }
    let view = View { rom: &rom, ewram: None, iwram: Some(&trace.bus.iwram), bios: None };
    let blocks = visited.iter().map(|&(address, thumb)| {
        let first = if thumb { armv4t::decode_thumb(view.read16(address), address) } else { armv4t::decode_arm(view.read32(address), address) };
        let mut instrs = vec![first];
        if thumb {
            let next = armv4t::decode_thumb(view.read16(address + 2), address + 2);
            if armv4t::fuse_thumb_bl(&instrs[0], &next).is_some() { instrs.push(next); }
        }
        Block { addr: address, thumb, instrs }
    }).collect();
    let object = root.join("layered-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().expect("TinyCC required"), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut native = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut native, &directory).unwrap();
    let owner = native.bus.rom_resources().unwrap().clone();
    let mut total_bytes = 0;
    for descriptor in &model.descriptors {
        raw.load_state(&initial).unwrap();
        native.load_state(&initial).unwrap();
        setup(&mut raw, descriptor);
        setup(&mut native, descriptor);
        let before = owner.layered_obj_animations_stats().unwrap().fields.source_bytes;
        for (offset, bytes, _) in &spans { native.bus.rom[*offset as usize..*offset as usize + bytes.len()].fill(0xcd); }
        run(&mut raw);
        let mut dispatches = 0;
        while native.cpu.regs[15] != RETURN {
            assert!(dispatches < 100_000, "translated initializer did not return");
            native.bus.frame_ready = false;
            table.get(native.cpu.regs[15] | u32::from(native.cpu.thumb())).expect("traced original instruction compiled")(&RT_API, &mut native as *mut Machine as *mut std::ffi::c_void);
            dispatches += 1;
        }
        let used = owner.layered_obj_animations_stats().unwrap().fields.source_bytes - before;
        assert!(used > 0, "translated initializer did not read managed source");
        total_bytes += used;
        for (offset, bytes, _) in &spans { native.bus.rom[*offset as usize..*offset as usize + bytes.len()].copy_from_slice(bytes); }
        assert_eq!(raw.save_state().unwrap(), native.save_state().unwrap(), "translated initializer full state");
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[130,129],"translated_initializer_full_state_pairs":16,"translated_managed_source_bytes":total_bytes,"traced_instruction_addresses":visited.len(),"ordinary_scene":false,"performance_measurements":false})).unwrap()).unwrap();
}

fn setup_draw(machine: &mut Machine, descriptor: &gba_core::rom_data::layered_obj_animations::Descriptor, variant: u32) {
    setup(machine, descriptor);
    run(machine);
    machine.bus.iwram.fill(0);
    machine.bus.iwram[0x2c58..0x2dd8].fill(0x5a);
    machine.cpu.regs = std::array::from_fn(|i| 0x2200_0000 + i as u32);
    machine.cpu.regs[..4].copy_from_slice(&[TASK, 100, 80, 0]);
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = 0x0814_2664;
    machine.cpu.cpsr = 0xbf;
    for (offset, value) in [(0, 0), (4, 0), (8, variant), (12, 0)] { machine.bus.write32(STACK + offset, value); }
}

#[test]
#[ignore = "private exact EUR ROM and E130 dataset; Windows executable under Wine"]
fn european_layered_obj_original_draw_managed_interpreter() {
    let root = PathBuf::from(std::env::var_os("FFTA_LAYERED_DRAW_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let directory = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let dataset = read_dataset(&directory).unwrap();
    assert_eq!((dataset.manifest.extractor_revision, dataset.manifest.decoder_revision), (130, 129));
    let model = dataset.layered_obj_animations.as_ref().unwrap();
    let spans = model.source_spans().unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut drawn = 0;
    let mut managed_bytes = 0;
    for descriptor in &model.descriptors {
        for variant in 0..3u32 {
            for machine in [&mut raw, &mut managed] { setup_draw(machine, descriptor, variant); }
            let before = owner.layered_obj_animations_stats().unwrap().fields.source_bytes;
            for (offset, bytes, _) in &spans {
                managed.bus.rom[*offset as usize..*offset as usize + bytes.len()].fill(0xcd);
            }
            run(&mut raw);
            run(&mut managed);
            let used = owner.layered_obj_animations_stats().unwrap().fields.source_bytes - before;
            assert!(used > 0, "original draw did not read managed sequence/object fields");
            managed_bytes += used;
            if raw.bus.iwram[0x2c58..0x2dd8].iter().any(|&v| v != 0x5a) { drawn += 1; }
            for (offset, bytes, _) in &spans {
                managed.bus.rom[*offset as usize..*offset as usize + bytes.len()].copy_from_slice(bytes);
            }
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "full state for {:x} variant {variant}", descriptor.object_root);
        }
    }
    assert!(drawn > 0, "no selected draw wrote to the OBJ buffer");
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[130,129],"original_draw_interpreter_pairs":48,"draw_variants":[0,1,2],"draws_writing_obj_buffer":drawn,"managed_source_bytes":managed_bytes,"ordinary_scene":false,"translated_native":"pending","performance_measurements":false})).unwrap()).unwrap();
}

#[test]
#[ignore = "private exact EUR ROM, E130 dataset and TinyCC; Windows executable under Wine"]
fn european_layered_obj_original_draw_managed_translated() {
    use gba_core::capi::RT_API;
    use recomp_core::analyze::{Analysis, Block, View};
    use std::collections::BTreeSet;
    let root = PathBuf::from(std::env::var_os("FFTA_LAYERED_DRAW_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let directory = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let dataset = read_dataset(&directory).unwrap();
    assert_eq!((dataset.manifest.extractor_revision, dataset.manifest.decoder_revision), (130, 129));
    let model = dataset.layered_obj_animations.as_ref().unwrap();
    let spans = model.source_spans().unwrap();
    let mut trace = crate::make_machine(rom.clone(), None);
    let mut visited = BTreeSet::new();
    for descriptor in &model.descriptors {
        for variant in 0..3u32 {
            setup_draw(&mut trace, descriptor, variant);
            for _ in 0..100_000 {
                if trace.cpu.regs[15] == RETURN { break; }
                visited.insert((trace.cpu.regs[15], trace.cpu.thumb()));
                trace.bus.frame_ready = false;
                trace.step();
            }
            assert_eq!(trace.cpu.regs[15], RETURN);
        }
    }
    let view = View { rom: &rom, ewram: None, iwram: Some(&trace.bus.iwram), bios: None };
    let blocks = visited.iter().map(|&(address, thumb)| {
        let first = if thumb { armv4t::decode_thumb(view.read16(address), address) } else { armv4t::decode_arm(view.read32(address), address) };
        let mut instrs = vec![first];
        if thumb {
            let next = armv4t::decode_thumb(view.read16(address + 2), address + 2);
            if armv4t::fuse_thumb_bl(&instrs[0], &next).is_some() { instrs.push(next); }
        }
        Block { addr: address, thumb, instrs }
    }).collect();
    let object = root.join("layered-draw-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().expect("TinyCC required"), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut native = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut native, &directory).unwrap();
    let owner = native.bus.rom_resources().unwrap().clone();
    let mut managed_bytes = 0;
    let mut drawn = 0;
    for descriptor in &model.descriptors {
        for variant in 0..3u32 {
            setup_draw(&mut raw, descriptor, variant);
            setup_draw(&mut native, descriptor, variant);
            let before = owner.layered_obj_animations_stats().unwrap().fields.source_bytes;
            for (offset, bytes, _) in &spans { native.bus.rom[*offset as usize..*offset as usize + bytes.len()].fill(0xcd); }
            run(&mut raw);
            let mut dispatches = 0;
            while native.cpu.regs[15] != RETURN {
                assert!(dispatches < 100_000, "translated draw did not return");
                native.bus.frame_ready = false;
                table.get(native.cpu.regs[15] | u32::from(native.cpu.thumb())).expect("traced draw instruction compiled")(&RT_API, &mut native as *mut Machine as *mut std::ffi::c_void);
                dispatches += 1;
            }
            let used = owner.layered_obj_animations_stats().unwrap().fields.source_bytes - before;
            assert!(used > 0, "translated draw did not read managed source");
            managed_bytes += used;
            if raw.bus.iwram[0x2c58..0x2dd8].iter().any(|&v| v != 0x5a) { drawn += 1; }
            for (offset, bytes, _) in &spans { native.bus.rom[*offset as usize..*offset as usize + bytes.len()].copy_from_slice(bytes); }
            assert_eq!(raw.save_state().unwrap(), native.save_state().unwrap(), "translated draw full state for {:x} variant {variant}", descriptor.object_root);
        }
    }
    assert!(drawn > 0);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[130,129],"original_draw_translated_pairs":48,"draw_variants":[0,1,2],"draws_writing_obj_buffer":drawn,"managed_source_bytes":managed_bytes,"traced_instruction_addresses":visited.len(),"ordinary_scene":false,"performance_measurements":false})).unwrap()).unwrap();
}

#[test]
#[ignore = "private exact EUR ROM and E130 dataset; Windows executable under Wine"]
fn european_layered_obj_each_frame_original_draw_interpreter() {
    let root = PathBuf::from(std::env::var_os("FFTA_LAYERED_DRAW_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let directory = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let dataset = read_dataset(&directory).unwrap();
    assert_eq!((dataset.manifest.extractor_revision, dataset.manifest.decoder_revision), (130, 129));
    let model = dataset.layered_obj_animations.as_ref().unwrap();
    let spans = model.source_spans().unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut cases = 0;
    let mut bytes_read = 0;
    let mut written = 0;
    for (descriptor, bank) in model.descriptors.iter().zip(&model.banks) {
        for sequence in &bank.sequences {
            for (index, _) in sequence.frames.iter().enumerate() {
                for machine in [&mut raw, &mut managed] {
                    setup_draw(machine, descriptor, 0);
                    machine.bus.write16(TASK + 0x58, 1);
                    machine.bus.write32(TASK + 0x0c, 0x0800_0000 + sequence.source_offset + 2);
                    machine.bus.write16(TASK + 0x6e, index as u16);
                    machine.bus.write8(TASK + 0x8c, 0);
                }
                let before = owner.layered_obj_animations_stats().unwrap().fields.source_bytes;
                for (offset, bytes, _) in &spans { managed.bus.rom[*offset as usize..*offset as usize + bytes.len()].fill(0xcd); }
                run(&mut raw);
                run(&mut managed);
                let used = owner.layered_obj_animations_stats().unwrap().fields.source_bytes - before;
                assert!(used > 0, "frame did not read managed fields");
                bytes_read += used;
                if raw.bus.iwram[0x2c58..0x2dd8].iter().any(|&v| v != 0x5a) { written += 1; }
                for (offset, bytes, _) in &spans { managed.bus.rom[*offset as usize..*offset as usize + bytes.len()].copy_from_slice(bytes); }
                assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "full state bank {:x} sequence {:x} frame {index}", descriptor.object_root, sequence.source_offset);
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 559);
    assert!(written > 0);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[130,129],"selected_frame_pairs":cases,"sequences":34,"draw_variant":0,"draws_writing_obj_buffer":written,"managed_source_bytes":bytes_read,"ordinary_scene":false,"natural_timing":"not tested","performance_measurements":false})).unwrap()).unwrap();
}

#[test]
#[ignore = "private exact EUR ROM and E130 dataset; Windows executable under Wine"]
fn european_layered_obj_natural_multilayer_loop_interpreter() {
    let root = PathBuf::from(std::env::var_os("FFTA_LAYERED_DRAW_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let directory = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let dataset = read_dataset(&directory).unwrap();
    assert_eq!((dataset.manifest.extractor_revision, dataset.manifest.decoder_revision), (130, 129));
    let model = dataset.layered_obj_animations.as_ref().unwrap();
    let spans = model.source_spans().unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut calls = 0usize;
    let mut visited = 0usize;
    let mut bytes_read = 0;
    for (descriptor, bank) in model.descriptors.iter().zip(&model.banks) {
        for machine in [&mut raw, &mut managed] {
            setup(machine, descriptor);
            run(machine);
            machine.bus.write8(TASK + 0x8c, 0);
            for layer in 0..bank.sequences.len() {
                machine.bus.write16(TASK + 0x5a + 2 * layer as u32, 0);
                machine.bus.write16(TASK + 0x6e + 2 * layer as u32, 0);
                machine.bus.write8(TASK + 0x82 + layer as u32, 0);
            }
        }
        let ticks = bank.sequences.iter().map(|sequence| sequence.frames.iter().map(|frame| usize::from(frame.duration) + 1).sum::<usize>()).max().unwrap() + 1;
        let mut seen = bank.sequences.iter().map(|s| vec![false; s.frames.len()]).collect::<Vec<_>>();
        for _ in 0..ticks {
            for (layer, layer_seen) in seen.iter_mut().enumerate() {
                let index = usize::from(raw.bus.read16(TASK + 0x6e + 2 * layer as u32));
                assert!(index < layer_seen.len());
                layer_seen[index] = true;
            }
            for machine in [&mut raw, &mut managed] {
                machine.bus.iwram.fill(0);
                machine.bus.iwram[0x2c58..0x2dd8].fill(0x5a);
                machine.cpu.regs = std::array::from_fn(|i| 0x2200_0000 + i as u32);
                machine.cpu.regs[..4].copy_from_slice(&[TASK, 100, 80, 0]);
                machine.cpu.regs[13] = STACK;
                machine.cpu.regs[14] = RETURN | 1;
                machine.cpu.regs[15] = 0x0814_2664;
                machine.cpu.cpsr = 0xbf;
                for offset in [0, 4, 8, 12] { machine.bus.write32(STACK + offset, 0); }
            }
            let before = owner.layered_obj_animations_stats().unwrap().fields.source_bytes;
            for (offset, bytes, _) in &spans { managed.bus.rom[*offset as usize..*offset as usize + bytes.len()].fill(0xcd); }
            run(&mut raw);
            run(&mut managed);
            let used = owner.layered_obj_animations_stats().unwrap().fields.source_bytes - before;
            assert!(used > 0);
            bytes_read += used;
            for (offset, bytes, _) in &spans { managed.bus.rom[*offset as usize..*offset as usize + bytes.len()].copy_from_slice(bytes); }
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "natural draw state at bank {:x} call {calls}", descriptor.object_root);
            calls += 1;
        }
        for layer in &seen {
            assert!(layer.iter().all(|&was_seen| was_seen), "natural looping missed a modeled frame");
            visited += layer.len();
        }
    }
    assert_eq!(visited, 559);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[130,129],"banks":16,"layers":34,"visited_frames":visited,"draw_calls":calls,"managed_source_bytes":bytes_read,"loop_mode":0,"draw_variant":0,"ordinary_scene":false,"translated_native":"pending","performance_measurements":false})).unwrap()).unwrap();
}

fn setup_natural_loop(machine: &mut Machine, descriptor: &gba_core::rom_data::layered_obj_animations::Descriptor, bank: &gba_core::rom_data::obj_animations::Bank) {
    setup(machine, descriptor);
    run(machine);
    machine.bus.write8(TASK + 0x8c, 0);
    for layer in 0..bank.sequences.len() {
        machine.bus.write16(TASK + 0x5a + 2 * layer as u32, 0);
        machine.bus.write16(TASK + 0x6e + 2 * layer as u32, 0);
        machine.bus.write8(TASK + 0x82 + layer as u32, 0);
    }
}

fn start_natural_draw(machine: &mut Machine) {
    machine.bus.iwram.fill(0);
    machine.bus.iwram[0x2c58..0x2dd8].fill(0x5a);
    machine.cpu.regs = std::array::from_fn(|i| 0x2200_0000 + i as u32);
    machine.cpu.regs[..4].copy_from_slice(&[TASK, 100, 80, 0]);
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = 0x0814_2664;
    machine.cpu.cpsr = 0xbf;
    for offset in [0, 4, 8, 12] { machine.bus.write32(STACK + offset, 0); }
}

fn natural_loop_bound(bank: &gba_core::rom_data::obj_animations::Bank) -> usize {
    bank.sequences.iter().map(|sequence| sequence.frames.iter().map(|frame| usize::from(frame.duration) + 1).sum::<usize>()).max().unwrap() + 1
}

#[test]
#[ignore = "private exact EUR ROM, E130 dataset and TinyCC; Windows executable under Wine"]
fn european_layered_obj_natural_multilayer_loop_translated() {
    use gba_core::capi::RT_API;
    use recomp_core::analyze::{Analysis, Block, View};
    use std::collections::BTreeSet;
    let root = PathBuf::from(std::env::var_os("FFTA_LAYERED_DRAW_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let directory = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let dataset = read_dataset(&directory).unwrap();
    assert_eq!((dataset.manifest.extractor_revision, dataset.manifest.decoder_revision), (130, 129));
    let model = dataset.layered_obj_animations.as_ref().unwrap();
    let spans = model.source_spans().unwrap();
    let mut trace = crate::make_machine(rom.clone(), None);
    let mut visited = BTreeSet::new();
    let mut visited_frames = 0usize;
    for (descriptor, bank) in model.descriptors.iter().zip(&model.banks) {
        setup_natural_loop(&mut trace, descriptor, bank);
        let mut seen = bank.sequences.iter().map(|s| vec![false; s.frames.len()]).collect::<Vec<_>>();
        for _ in 0..natural_loop_bound(bank) {
            for (layer, layer_seen) in seen.iter_mut().enumerate() {
                let index = usize::from(trace.bus.read16(TASK + 0x6e + 2 * layer as u32));
                assert!(index < layer_seen.len());
                layer_seen[index] = true;
            }
            start_natural_draw(&mut trace);
            for _ in 0..100_000 {
                if trace.cpu.regs[15] == RETURN { break; }
                visited.insert((trace.cpu.regs[15], trace.cpu.thumb()));
                trace.bus.frame_ready = false;
                trace.step();
            }
            assert_eq!(trace.cpu.regs[15], RETURN);
        }
        for layer in &seen { assert!(layer.iter().all(|&v| v)); visited_frames += layer.len(); }
    }
    assert_eq!(visited_frames, 559);
    let view = View { rom: &rom, ewram: None, iwram: Some(&trace.bus.iwram), bios: None };
    let blocks = visited.iter().map(|&(address, thumb)| {
        let first = if thumb { armv4t::decode_thumb(view.read16(address), address) } else { armv4t::decode_arm(view.read32(address), address) };
        let mut instrs = vec![first];
        if thumb {
            let next = armv4t::decode_thumb(view.read16(address + 2), address + 2);
            if armv4t::fuse_thumb_bl(&instrs[0], &next).is_some() { instrs.push(next); }
        }
        Block { addr: address, thumb, instrs }
    }).collect();
    let object = root.join("layered-loop-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().expect("TinyCC required"), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut native = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut native, &directory).unwrap();
    let owner = native.bus.rom_resources().unwrap().clone();
    let mut calls = 0usize;
    let mut bytes_read = 0;
    for (descriptor, bank) in model.descriptors.iter().zip(&model.banks) {
        setup_natural_loop(&mut raw, descriptor, bank);
        setup_natural_loop(&mut native, descriptor, bank);
        for _ in 0..natural_loop_bound(bank) {
            start_natural_draw(&mut raw);
            start_natural_draw(&mut native);
            let before = owner.layered_obj_animations_stats().unwrap().fields.source_bytes;
            for (offset, bytes, _) in &spans { native.bus.rom[*offset as usize..*offset as usize + bytes.len()].fill(0xcd); }
            run(&mut raw);
            let mut dispatches = 0;
            while native.cpu.regs[15] != RETURN {
                assert!(dispatches < 100_000, "translated natural draw did not return");
                native.bus.frame_ready = false;
                table.get(native.cpu.regs[15] | u32::from(native.cpu.thumb())).expect("traced natural draw instruction compiled")(&RT_API, &mut native as *mut Machine as *mut std::ffi::c_void);
                dispatches += 1;
            }
            let used = owner.layered_obj_animations_stats().unwrap().fields.source_bytes - before;
            assert!(used > 0);
            bytes_read += used;
            for (offset, bytes, _) in &spans { native.bus.rom[*offset as usize..*offset as usize + bytes.len()].copy_from_slice(bytes); }
            assert_eq!(raw.save_state().unwrap(), native.save_state().unwrap(), "translated natural draw state at bank {:x} call {calls}", descriptor.object_root);
            calls += 1;
        }
    }
    assert_eq!(calls, 3457);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[130,129],"banks":16,"layers":34,"visited_frames":visited_frames,"translated_draw_calls":calls,"traced_instruction_addresses":visited.len(),"managed_source_bytes":bytes_read,"loop_mode":0,"draw_variant":0,"ordinary_scene":false,"performance_measurements":false})).unwrap()).unwrap();
}
