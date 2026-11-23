//! EUR Star Cross compact-bank publication, original readers and host sprite lists.
use super::*;
use gba_core::{capi::RT_API, rom_data::{compact_obj_catalog as model, RuntimeResources}, Bus, Machine};

// The parts of the authenticated Star Cross bank that E291 did not own.
pub(super) const NEW_SPANS: &[(usize, usize)] = &[
    (0x6bacdc, 0x6bad0a), (0x6bad16, 0x6bb4f4),
    (0x6bb524, 0x6bb528), (0x6bb52c, 0x6bb560),
];
const START: usize = 0x6bacd8;
const END: usize = 0x6bb572;
const STACK: u32 = 0x03007c00;

fn invert_spans(machine: &mut Machine, rom: &[u8], spans: &[(usize, usize)]) {
    for &(start, end) in spans {
        for at in start..end { machine.bus.rom[at] = !rom[at]; }
    }
}

fn sprite_machine(rom: &[u8]) -> Machine {
    let mut machine = crate::make_machine(rom.to_vec(), None);
    machine.bus.iwram[0x5f28..0x6d68].copy_from_slice(&rom[0xd968e8..0xd97728]);
    machine.cpu.regs[13] = STACK;
    machine.cpu.cpsr = 0xbf;
    machine
}

#[test]
#[ignore = "exact EUR ROM, E291 predecessor and compiled reader; Wine only"]
fn european_star_cross_compact_bank_publication_and_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_STARCROSS_ROOT").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_COMPACT_STARCROSS_PRIOR").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (291, 290, 148));
    let old: model::CompactObjCatalog = serde_json::from_slice(&fs::read(prior.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();

    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (292, 291, 148));
    for entry in &old_manifest.resources {
        if entry.id == COMPACT_OBJ_CATALOG_ID { continue; }
        let now = source.manifest.resources.iter().find(|current| current.id == entry.id).unwrap();
        assert_eq!(now, entry);
        assert_eq!(fs::read(prepared.directory.join(&now.path)).unwrap(), fs::read(prior.join(&entry.path)).unwrap());
    }
    let data = &source.compact_obj_catalog.as_ref().unwrap().decoded;
    data.verify_source(&rom).unwrap();
    assert_eq!(parse_compact_obj_catalog(&fs::read(prepared.directory.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap(), *data);
    assert_eq!(data.selector_tables, old.selector_tables);
    assert_eq!(data.red_spring_records, old.red_spring_records);
    assert_eq!(data.soil_evidence_metadata, old.soil_evidence_metadata);
    for previous in &old.banks {
        let current = data.banks.iter().find(|bank| bank.root == previous.root).unwrap();
        if current.root != START as u32 { assert_eq!(current, previous); continue; }
        assert_eq!((current.flags, current.sequence_relative_offset), (previous.flags, previous.sequence_relative_offset));
        for list in &previous.lists { assert_eq!(current.lists.iter().find(|new| new.index == list.index), Some(list)); }
        for sequence in &previous.sequences { assert_eq!(current.sequence(sequence.sequence.selector), Some(&sequence.sequence)); }
    }
    let bank = data.banks.iter().find(|bank| bank.root == START as u32).unwrap();
    assert_eq!(bank.sequences.iter().map(|s| (s.sequence.selector, s.sequence.frames.len())).collect::<Vec<_>>(),
        [(0,6), (1,18), (2,4), (3,3)]);
    assert_eq!(bank.lists.len(), 29);
    assert!(bank.list(22).unwrap().objects.is_empty());
    assert_eq!(bank.lists[..23].iter().map(|entry| entry.list.objects.len()).sum::<usize>(), 328);
    assert_eq!(NEW_SPANS.iter().map(|(a,z)| z-a).sum::<usize>(), 2116);
    assert_eq!(model::BANK_SPECS.iter().find(|spec| spec.root == START as u32).unwrap().spans, &[(START as u32, END as u32)]);

    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = managed.save_state().unwrap();
    invert_spans(&mut managed, &rom, NEW_SPANS);
    let before = owner.compact_obj_catalog_stats().unwrap().source_bytes;
    for &(a,z) in NEW_SPANS { for at in a..z {
        assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]);
    } }
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 2116);
    for edge in NEW_SPANS.iter().flat_map(|&(a,z)| [a,z]).chain([START,END]) {
        for delta in [-3isize,-1,0,1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
            assert_eq!((RT_API.read16)(ptr, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
            assert_eq!((RT_API.read32)(ptr, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
        }
    }
    for &(a,z) in NEW_SPANS { managed.bus.rom[a..z].copy_from_slice(&rom[a..z]); }
    assert_eq!(managed.save_state().unwrap(), initial);

    let mut forged = read_dataset(&prepared.directory).unwrap();
    forged.compact_obj_catalog.as_mut().unwrap().banks.iter_mut()
        .find(|bank| bank.root == START as u32).unwrap().lists[0].list.objects[0].tile ^= 1;
    let forged_bytes = encode_compact_obj_catalog(forged.compact_obj_catalog.as_ref().unwrap()).unwrap();
    forged.manifest.resources.iter_mut().find(|entry| entry.id == COMPACT_OBJ_CATALOG_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&forged_bytes);
    forged.compact_obj_catalog_bytes = Some(forged_bytes);
    assert!(RuntimeResources::from_base_files(&rom, forged.files()).is_err());
    assert!(std::sync::Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    assert_eq!(managed.save_state().unwrap(), initial);

    let object = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap());
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut native = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut native, &prepared.directory).unwrap();
    let native_owner = native.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    for selector in 0..=2u16 {
        for machine in [&mut raw, &mut native] {
            machine.load_state(&initial).unwrap();
            machine.bus.ffta_sprites.enabled = false;
            machine.bus.write32(0x02007f08, 0x08000000 + bank.sequence_root());
            machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
            machine.cpu.regs[3] = u32::from(selector); machine.cpu.regs[4] = 0x02007f00;
            machine.cpu.regs[13] = STACK; machine.cpu.regs[15] = 0x08010870; machine.cpu.cpsr = 0xbf;
        }
        invert_spans(&mut native, &rom, NEW_SPANS);
        super::compact_obj_catalog_tests::prefix(&mut raw, 0x08010870, 0x08010884);
        let before = native_owner.compact_obj_catalog_stats().unwrap().source_bytes;
        let mut steps = 0;
        while native.cpu.regs[15] != 0x08010884 {
            assert!(steps < 32); native.bus.frame_ready = false;
            table.get(native.cpu.regs[15] | 1).unwrap()(&RT_API, &mut native as *mut Machine as *mut std::ffi::c_void);
            steps += 1;
        }
        assert_eq!(native_owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 4);
        for &(a,z) in NEW_SPANS { native.bus.rom[a..z].copy_from_slice(&rom[a..z]); }
        assert_eq!(native.save_state().unwrap(), raw.save_state().unwrap());
    }

    let mut raw = sprite_machine(&rom);
    let mut typed = sprite_machine(&rom);
    attach_from_directory(&mut typed, &prepared.directory).unwrap();
    let host_owner = typed.bus.rom_resources().unwrap().clone();
    invert_spans(&mut typed, &rom, &[(START,END)]);
    gba_core::mem::set_extra(120);
    let mut checked = 0;
    for entry in bank.lists.iter().filter(|entry| entry.index < 23) {
        let list = &entry.list;
        if list.objects.is_empty() { continue; }
        let mut oam = vec![0u8; 1024];
        for (i, object) in list.objects.iter().enumerate() {
            let [a0,a1,a2] = object.attributes();
            let attrs = [(a0 & 0xff00) | ((64 + i32::from(object.y)) as u16 & 255),
                (a1 & !511) | ((100 + i32::from(object.x)) as u16 & 511), a2];
            for (j, attr) in attrs.into_iter().enumerate() { oam[i*8+j*2..i*8+j*2+2].copy_from_slice(&attr.to_le_bytes()); }
        }
        let before = host_owner.compact_obj_catalog_stats().unwrap().host_reads;
        for machine in [&mut raw, &mut typed] {
            machine.bus.frames = checked + 1;
            machine.cpu.regs[..4].copy_from_slice(&[0x08000000 + list.source_offset, 100, 64, 0]);
            machine.cpu.regs[15] = 0x030065a8;
            for (i, value) in [0,0,0,0,0,0x02020000,0,0].into_iter().enumerate() {
                machine.bus.write32(STACK + i as u32 * 4, value);
            }
            machine.bus.ffta_sprites.enabled = true;
            machine.ffta_sprite_entry();
            machine.bus.ffta_sprites.publish(0x03000030, 0x07000000, 1024, true, &oam);
        }
        assert!(host_owner.compact_obj_catalog_stats().unwrap().host_reads > before);
        assert_eq!(format!("{:?}", typed.bus.ffta_sprites.active), format!("{:?}", raw.bus.ffta_sprites.active));
        assert_eq!(typed.bus.ffta_sprites.active.as_ref().unwrap().len(), list.objects.len());
        checked += 1;
    }
    gba_core::mem::set_extra(0);
    assert_eq!(checked, 22);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[292,291], "resources":148, "preserved_other_resources":147,
        "prior_banks_preserved":old.banks.len(), "bank_bytes":END-START, "new_bytes":2116,
        "new_selectors":[0,1], "bank_lists":29, "typed_host_lists":checked,
        "poisoned_mirrored_byte_and_boundary_reads":true, "original_interpreter_vs_native_prefixes":3,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
