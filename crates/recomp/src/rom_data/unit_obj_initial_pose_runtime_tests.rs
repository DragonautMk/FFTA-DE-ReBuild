//! Two reduced Wine filters: immutable unit source lifecycle and original/native guest reads.
use super::*;
use gba_core::{capi::RT_API, rom_data::unit_obj_initial_pose as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};

fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_UNIT_OBJ_MANAGED_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn report(name: &str, value: serde_json::Value) {
    fs::write(
        root().join(name),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
}
fn ranges(files: &Dataset) -> Vec<(usize, usize, u8)> {
    let mut result: Vec<_> = files
        .unit_obj_initial_pose
        .as_ref()
        .unwrap()
        .source_spans()
        .unwrap()
        .into_iter()
        .map(|(at, b, _)| (at as usize, b.len(), 0x5a))
        .collect();
    let old = files
        .manifest
        .resources
        .iter()
        .find(|r| r.id == OBJ_STATIC_LISTS_ID)
        .unwrap()
        .sources
        .iter()
        .find(|s| s.offset == 0x394f20)
        .unwrap();
    result.push((old.offset as usize, old.length as usize, 0xa5));
    result
}
fn sources(m: &mut Machine, files: &Dataset, rom: &[u8], poison: bool) {
    for (at, len, salt) in ranges(files) {
        m.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
        if poison {
            for byte in &mut m.bus.rom[at..at + len] {
                *byte ^= salt;
            }
        }
    }
}

#[test]
#[ignore = "private EUR e57 unit OBJ publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_unit_obj_initial_pose_managed_authentication_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..52]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e56").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (56, 55));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (57, 56)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 53);
    assert_eq!(source.manifest.resources.len(), 52);
    for i in 1..52 {
        assert_eq!(files[i], old[i]);
    }
    let data = parse_unit_obj_initial_pose(&files[52]).unwrap();
    let accepted = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_UNIT_OBJ_SOURCE_ROOT").unwrap());
    assert_eq!(
        files[52],
        fs::read(accepted.join(UNIT_OBJ_INITIAL_POSE_FILE)).unwrap()
    );
    assert_eq!(
        source.manifest.resources[51].dependencies,
        vec![METADATA52_ID.to_string(), METADATA14_ID.to_string()]
    );
    assert_eq!(source.manifest.resources[51].sources.len(), 415);
    assert_eq!(
        source.manifest.resources[51]
            .sources
            .iter()
            .map(|s| s.length)
            .sum::<u32>(),
        146358
    );
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| if p
        .ends_with(UNIT_OBJ_INITIAL_POSE_FILE)
    {
        Err("injected unit OBJ publication failure".into())
    } else {
        write_synced(p, b)
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e56-d55/generation-retained");
    fs::create_dir_all(&retained).unwrap();
    for (n, b) in names().iter().zip(&old) {
        fs::write(retained.join(n), b).unwrap();
    }
    fs::write(
        retained.parent().unwrap().join("current"),
        b"generation-retained\n",
    )
    .unwrap();
    let prepared = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e57-d56"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(
        prepared.directory.join(UNIT_OBJ_INITIAL_POSE_FILE),
        b"partial",
    )
    .unwrap();
    let repaired = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(repaired.action, "repaired");
    let relative = repaired
        .directory
        .strip_prefix(&managed)
        .unwrap()
        .to_path_buf();
    let retained_relative = retained.strip_prefix(&managed).unwrap().to_path_buf();
    let moved = r.join("moved resources");
    fs::rename(managed, &moved).unwrap();
    assert_eq!(
        preparation::prepare(&moved, &mut source, &mut |_| Ok(()))
            .unwrap()
            .directory,
        moved.join(relative)
    );
    for (n, b) in names().iter().zip(&old) {
        assert_eq!(
            fs::read(moved.join(&retained_relative).join(n)).unwrap(),
            *b
        );
        assert_eq!(fs::read(prior_dir.join(n)).unwrap(), *b);
    }
    let fileset = read_dataset(&dataset()).unwrap();
    let mut missing = fileset.files();
    missing.unit_obj_initial_pose = None;
    assert!(source.validate_files(missing).is_err());
    let mut missing = fileset.files();
    missing.metadata52 = None;
    assert!(source.validate_files(missing).is_err());
    let mut inconsistent = Extraction::from_rom(&rom).unwrap();
    inconsistent.metadata14 = None;
    inconsistent.manifest.resources.remove(24);
    assert_eq!(
        inconsistent.validate_files(fileset.files()),
        Err("Extraction resource set is inconsistent".into())
    );
    let mut m = crate::make_machine(rom.clone(), None);
    let boot = m.save_state().unwrap();
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let mut guards = 0;
    for case in 0..16 {
        let mut manifest = source.manifest.clone();
        let mut json: serde_json::Value = serde_json::from_slice(&files[52]).unwrap();
        match case {
            0 => {
                json["pixels"][0]["tiles"][0]["indices"][0][0] =
                    (data.pixels[0].tiles[0].indices[0][0] ^ 15).into()
            }
            1 => json["sequences"][0]["opaque_header"] = 0x8000u16.into(),
            2 => json["sequences"][0]["frames"][0]["bytes_0a_13"][0] = 255.into(),
            3 => manifest.resources[51].sources[0].sha256 = "0".repeat(64),
            4 => manifest.resources[51].sources[0].offset += 2,
            5 => manifest.resources[51].sources[0].length -= 2,
            6 => manifest.resources[51].sources[0].role = "unknown/choice".into(),
            7 => manifest.resources[51].path = "../choice.json".into(),
            8 => manifest.resources[51].dependencies = vec![AUXILIARY_ARCHIVE_ID.into()],
            9 => manifest.resources[51].sha256 = "0".repeat(64),
            10 => manifest
                .coverage
                .retain(|id| id != UNIT_OBJ_INITIAL_POSE_ID),
            11 => {
                manifest.resources[51].sources.pop();
            }
            12 => json["unknown"] = 1.into(),
            13 => {
                json["bindings"].as_array_mut().unwrap().pop();
            }
            14 => json["sequences"][0]["frames"][0]["operation"] = 2.into(),
            _ => json["pixels"][0]["tiles"][0]["indices"][0][0] = 16.into(),
        }
        let bytes = serde_json::to_vec_pretty(&json).unwrap();
        if case != 9 {
            manifest.resources[51].sha256 = gba_core::ffta_rom::digest(&bytes);
        }
        fs::write(dataset().join(UNIT_OBJ_INITIAL_POSE_FILE), bytes).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(
            attach_from_directory(&mut m, &dataset()).is_err(),
            "guard{case}"
        );
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), boot);
        guards += 1;
    }
    fs::write(dataset().join(UNIT_OBJ_INITIAL_POSE_FILE), &files[52]).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &files[0]).unwrap();
    fs::rename(
        dataset().join(UNIT_OBJ_INITIAL_POSE_FILE),
        r.join("missing.json"),
    )
    .unwrap();
    assert!(attach_from_directory(&mut m, &dataset()).is_err());
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    assert_eq!(m.save_state().unwrap(), boot);
    guards += 1;
    fs::rename(
        r.join("missing.json"),
        dataset().join(UNIT_OBJ_INITIAL_POSE_FILE),
    )
    .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let copy = temp.path().join("copy");
    fs::create_dir(&copy).unwrap();
    for name in names() {
        fs::copy(dataset().join(name), copy.join(name)).unwrap();
    }
    attach_from_directory(&mut m, &copy).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    fs::rename(copy, temp.path().join("unavailable")).unwrap();
    let save = temp.path().join("test.sav");
    let mut slots = crate::save_state::Slots::new(&save, &rom);
    slots.save(&m, 1, &vec![[0; 4]; 240 * 160], false).unwrap();
    m.cpu.regs[4] = 99;
    let changed = m.save_state().unwrap();
    slots.load(&mut m, 1).unwrap();
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
    sources(&mut m, &fileset, &rom, true);
    for (at, bytes, _) in fileset
        .unit_obj_initial_pose
        .as_ref()
        .unwrap()
        .source_spans()
        .unwrap()
    {
        for (n, &byte) in bytes.iter().enumerate() {
            assert_eq!(m.bus.read8(0x08000000 + at + n as u32), byte);
        }
    }
    let i = owner.unit_obj_initial_pose_stats().unwrap();
    assert_eq!(
        [
            i.source_bytes,
            i.directory_bytes,
            i.index_bytes,
            i.sequence_header_bytes,
            i.frame_bytes,
            i.object_bytes,
            i.pixel_bytes,
            i.sequences_used
        ],
        [146358, 484, 484, 484, 9540, 198, 135168, 121]
    );
    let old = owner.obj_list_stats().unwrap().source_bytes;
    assert_eq!(
        m.bus.read32(0x08394f20),
        u32::from_le_bytes(rom[0x394f20..0x394f24].try_into().unwrap())
    );
    assert_eq!(owner.obj_list_stats().unwrap().source_bytes - old, 4);
    assert_eq!(
        owner.unit_obj_initial_pose_stats().unwrap().source_bytes,
        i.source_bytes
    );
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,57,56],"files":53,"resources":52,"source_ranges":415,"source_bytes":146358,"all51prior_decoded_and_e56_retained":true,"guards":guards,"actual_metadata52_metadata14_dependencies":true,"failed_attach_owner_state_transactional":true,"publication_failure_create_reuse_repair_move":true,"unavailable_load_undo_backup_restart_sharedowner":true,"typed_directory_index_header_frame_object_pixel_bytes":[484,484,484,9540,198,135168],"sequences_used":121,"prior_OBJ_samepage_typed_bytes":4,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}

#[derive(Clone, Debug)]
enum Probe {
    Reset(u32),
    Root(u32, u32),
    Constructor(u32),
    Upload(u32),
    UploadRam(u32),
    Flush(u32, u32),
    Read(u32, usize),
}
impl Probe {
    fn bounds(&self) -> (u32, u32) {
        match self {
            Self::Reset(_) => (0x08022e20, 0x08000300),
            Self::Root(..) => (0x08022174, 0x08000300),
            Self::Constructor(_) => (0x08022788, 0x080227aa),
            Self::Upload(_) | Self::UploadRam(_) => (0x08022400, 0x08000300),
            Self::Flush(..) => (0x08000800, 0x08000300),
            Self::Read(_, 1) => (0x080221de, 0x080221e0),
            Self::Read(_, 2) => (0x08022e4c, 0x08022e4e),
            Self::Read(_, 4) => (0x0802217e, 0x08022180),
            _ => unreachable!(),
        }
    }
    fn setup(&self, m: &mut Machine) {
        m.bus.ewram.fill(0x5a);
        m.bus.iwram.fill(0);
        m.bus.vram.fill(0xa5);
        m.cpu.regs = [0; 16];
        for n in 4..12 {
            m.cpu.regs[n] = 0x41414000 + n as u32 * 0x101;
        }
        m.cpu.regs[13] = 0x03007e00;
        m.cpu.regs[14] = 0x08000301;
        m.cpu.regs[15] = self.bounds().0;
        m.cpu.cpsr = 0xbf;
        m.bus.reg_ie = 0;
        m.bus.ime = false;
        match *self {
            Self::Reset(id) => m.cpu.regs[..4].copy_from_slice(&[0x02010000, id, 0, 0]),
            Self::Root(id, anim) => m.cpu.regs[..2].copy_from_slice(&[id, anim]),
            Self::Constructor(id) => {
                m.cpu.regs[..4].copy_from_slice(&[0x02011000, 0x02010000, 0, 20]);
                m.bus.write32(0x03007e00, id);
            }
            Self::Upload(frame) | Self::UploadRam(frame) => {
                m.bus.ewram[0x10000..0x10048].fill(0);
                m.bus.ewram[0x11000..0x111d4].fill(0);
                m.bus.write32(0x02010000, 0x45);
                m.bus.write16(0x0201000e, 1);
                m.bus.write16(0x02010012, 0x10);
                m.bus.write32(0x02010020, u32::MAX);
                let pointer = if matches!(self, Self::UploadRam(_)) {
                    let raw: Vec<_> = (0..20)
                        .map(|i| m.bus.read8(0x08000000 + frame + i))
                        .collect();
                    // An ordinary source-bus copy creates the mutable test frame.
                    m.bus.ewram[0x18000..0x18014].copy_from_slice(&raw);
                    m.bus.ewram[0x18008] = 23;
                    m.bus.ewram[0x1800b] = 83;
                    m.bus.ewram[0x18011] = 1;
                    m.bus.ewram[0x18012] = 1;
                    0x02018000
                } else {
                    0x08000000 + frame
                };
                m.bus.write32(0x02010038, pointer);
                m.bus.write16(0x020111cc, 4);
                m.cpu.regs[..2].copy_from_slice(&[0x02010000, 0x02011000]);
            }
            Self::Flush(source, len) => {
                m.bus.ewram[0x11000..0x1101c].fill(0);
                m.bus.ewram[0x11001] = 1;
                m.bus.write32(0x02011004, 0x08000000 + source);
                m.bus.write32(0x02011008, 0x06010200);
                m.bus.write32(0x0201100c, 0x84000000 | len / 4);
                m.bus.write32(0x03002bcc, 0x02011000);
            }
            Self::Read(at, 1 | 2) => m.cpu.regs[4] = at,
            Self::Read(at, 4) => m.cpu.regs[0] = at,
            _ => unreachable!(),
        }
    }
}
fn original(m: &mut Machine, p: &Probe, pcs: &mut BTreeSet<u32>) -> usize {
    p.setup(m);
    let mut n = 0;
    while m.cpu.regs[15] != p.bounds().1 {
        assert!(n < 20000, "{p:?} PC{:08x}", m.cpu.regs[15]);
        pcs.insert(m.cpu.regs[15]);
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    n
}
#[test]
#[ignore = "private EUR e57 unit OBJ runtime/native/state-clock; Wine only, Linux/USA pending"]
fn european_unit_obj_initial_pose_native_reads_uploads_and_boundaries() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.unit_obj_initial_pose.as_ref().unwrap();
    let mut cases = vec![
        Probe::Reset(0),
        Probe::Reset(63),
        Probe::Reset(123),
        Probe::Root(0, 1),
        Probe::Root(123, 140),
        Probe::Root(65535, 65537),
        Probe::Constructor(0),
    ];
    let mut covered = BTreeSet::new();
    let mut samples = Vec::new();
    for sequence in &data.sequences {
        for (i, f) in sequence.frames.iter().enumerate() {
            if covered.insert(f.object_offset) {
                samples.push((sequence.source_offset + 4 + i as u32 * 20, f.clone()));
            }
        }
    }
    assert_eq!(samples.len(), 6);
    for (at, f) in &samples {
        cases.push(Probe::Upload(*at));
        let list = data
            .objects
            .iter()
            .find(|o| o.source_offset == model::OBJECT_BASE + f.object_offset)
            .unwrap();
        cases.push(Probe::Flush(
            model::PIXEL_BASE + f.pixel_offset,
            model::transfer_bytes(model::transfer_tiles(list)),
        ));
    }
    // The bus copy creates a mutable RAM frame; live fields stay authoritative.
    cases.push(Probe::UploadRam(samples[0].0));
    cases.extend([
        Probe::Read(0x08394834, 4),
        Probe::Read(0x08000000 + data.bindings[0].index_source, 4),
        Probe::Read(0x08000000 + data.sequences[0].source_offset, 2),
        Probe::Read(0x08000000 + samples[0].0 + 8, 1),
        Probe::Read(0x08a50020, 2),
        Probe::Read(0x08898a40, 1),
        Probe::Read(0x08394a24, 4),
        Probe::Read(0x08394f20, 4),
        Probe::Read(0x083948dc, 4),
        Probe::Read(0x02018000, 4),
    ]);
    let mut tracer = crate::make_machine(rom.clone(), None);
    let start = tracer.save_state().unwrap();
    let mut pcs = BTreeSet::new();
    let mut instructions = 0;
    for p in &cases {
        tracer.load_state(&start).unwrap();
        instructions += original(&mut tracer, p, &mut pcs);
    }
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks: Vec<Block> = pcs
        .iter()
        .map(|&addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let out = root().join("native");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let object = out.join("unit-obj-initial-pose.o");
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
    let owner = decoded.bus.rom_resources().unwrap().clone();
    let api = gba_core::capi::RtApi {
        read8,
        read16,
        read32,
        guard: prefix_guard,
        ..RT_API
    };
    COMPILED.with(|v| v.set([0; 6]));
    let mut dispatches = 0;
    for p in &cases {
        for m in [&mut legacy, &mut decoded, &mut native] {
            m.load_state(&start).unwrap();
        }
        original(&mut legacy, p, &mut BTreeSet::new());
        for m in [&mut decoded, &mut native] {
            sources(m, &files, &rom, true);
        }
        let before = owner.unit_obj_initial_pose_stats().unwrap().source_bytes;
        original(&mut decoded, p, &mut BTreeSet::new());
        p.setup(&mut native);
        STOP_PC.with(|v| v.set(p.bounds().1));
        let mut n = 0;
        while native.cpu.regs[15] != p.bounds().1 {
            assert!(n < 20000, "native{p:?} PC{:08x}", native.cpu.regs[15]);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | 1)
                .unwrap_or_else(|| panic!("missing native{p:?} PC{:08x}", native.cpu.regs[15]))(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            n += 1;
            dispatches += 1;
        }
        if matches!(
            p,
            Probe::Reset(_)
                | Probe::Constructor(_)
                | Probe::Upload(_)
                | Probe::UploadRam(_)
                | Probe::Flush(..)
        ) {
            assert!(
                owner.unit_obj_initial_pose_stats().unwrap().source_bytes > before,
                "positive normal{p:?}"
            );
        }
        if matches!(p, Probe::UploadRam(_)) {
            assert_eq!(decoded.bus.read16(0x0201000a), 23);
        }
        for m in [&mut decoded, &mut native] {
            sources(m, &files, &rom, false);
        }
        let expected = legacy.save_state().unwrap();
        for m in [&mut decoded, &mut native] {
            assert!(
                m.save_state().unwrap() == expected,
                "complete state/clock{p:?}"
            );
        }
    }
    let compiled = COMPILED.with(Cell::get);
    assert!(
        compiled.iter().all(|n| *n > 0),
        "all six typed API categories{compiled:?}"
    );
    decoded.load_state(&start).unwrap();
    sources(&mut decoded, &files, &rom, true);
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for (at, bytes, _) in data.source_spans().unwrap() {
            for boundary in [at, at + bytes.len() as u32] {
                for off in boundary - 3..boundary + 4 {
                    for mirror in [0x08000000, 0x0a000000, 0x0c000000] {
                        assert_eq!(decoded.bus.read8(mirror + off), rom[off as usize]);
                        assert_eq!(
                            decoded.bus.read16(mirror + off),
                            u16::from_le_bytes(
                                rom[off as usize..off as usize + 2].try_into().unwrap()
                            )
                        );
                        assert_eq!(
                            (RT_API.read32)(
                                &mut decoded as *mut Machine as *mut std::ffi::c_void,
                                mirror + off
                            ),
                            u32::from_le_bytes(
                                rom[off as usize..off as usize + 4].try_into().unwrap()
                            )
                        );
                    }
                }
            }
        }
    }
    decoded.bus.real_bios = false;
    // Last directory halfword/new + two deliberately edited raw bytes.
    let at = 0x394a22usize;
    decoded.bus.rom[at + 2..at + 4]
        .iter_mut()
        .for_each(|b| *b ^= 0x33);
    let before = owner.unit_obj_initial_pose_stats().unwrap().source_bytes;
    let mut expected: [u8; 4] = rom[at..at + 4].try_into().unwrap();
    expected[2..].iter_mut().for_each(|b| *b ^= 0x33);
    assert_eq!(
        (RT_API.read32)(
            &mut decoded as *mut Machine as *mut std::ffi::c_void,
            0x08000000 + at as u32
        ),
        u32::from_le_bytes(expected)
    );
    assert_eq!(
        owner.unit_obj_initial_pose_stats().unwrap().source_bytes - before,
        2
    );
    decoded.bus.rom[at + 2..at + 4].copy_from_slice(&rom[at + 2..at + 4]);
    sources(&mut decoded, &files, &rom, false);
    let mut mixed_swi = None;
    for (swi, src, count) in [(0xdf0b, 0x394a22, 643), (0xdf0c, 0x898a40, 8)] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a000000 + src, 0x02032400, count]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, swi);
        }
        sources(&mut decoded, &files, &rom, true);
        let b = owner.unit_obj_initial_pose_stats().unwrap().source_bytes;
        let old = owner.obj_list_stats().unwrap().source_bytes;
        legacy.step();
        decoded.step();
        let delta = (
            owner.unit_obj_initial_pose_stats().unwrap().source_bytes - b,
            owner.obj_list_stats().unwrap().source_bytes - old,
        );
        assert!(delta.0 > 0);
        if src == 0x394a22 {
            assert_eq!(delta, (2, 8));
            mixed_swi = Some(delta);
        }
        sources(&mut decoded, &files, &rom, false);
        assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    }
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.bus.write32(0x040000d4, 0x0a394a22);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 643);
    }
    sources(&mut decoded, &files, &rom, true);
    let b = owner.unit_obj_initial_pose_stats().unwrap().source_bytes;
    let old = owner.obj_list_stats().unwrap().source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    let mixed_dma = (
        owner.unit_obj_initial_pose_stats().unwrap().source_bytes - b,
        owner.obj_list_stats().unwrap().source_bytes - old,
    );
    assert_eq!(mixed_dma, (2, 8));
    sources(&mut decoded, &files, &rom, false);
    assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({"revision":[1,57,56],"original_interpreter_native_paths":cases.len(),"compiled_directory_index_header_frame_object_pixel_bytes":compiled,"full_states_clocks_equal":true,"raw_ID_animation_and_live_RAMframe":true,"widths_mirrors_realBIOS_bus_CpuSet_FastSet_DMA":true,"mixed_raw_API_new_bytes":2,"mixed_new_raw_OBJ_SWI_bytes":mixed_swi,"mixed_new_raw_OBJ_DMA_bytes":mixed_dma,"instructions_coverage_only":instructions,"pcs":pcs.len(),"native_dispatches_coverage_only":dispatches,"scope":"typed initial unit sources/original scheduler and DMA; no wholeunit allocator or ordinary ABI claim","linux":"pending","usa":"pending","performance_measurements":false}),
    );
}

thread_local! {
    static COMPILED:Cell<[u64;6]>=const{Cell::new([0;6])};
    static STOP_PC:Cell<u32>=const{Cell::new(0)};
}
// The production emitter chains neighboring blocks. A prefix endpoint can
// also be an interior instruction in another probe's compiled graph. Keep the
// real event guard, and yield at the declared checkpoint before executing it.
extern "C" fn prefix_guard(m: *mut std::ffi::c_void, a: u32, len: u32, expect: u64) -> u32 {
    if len == 0 && expect == gba_core::capi::EVENT_GUARD_TAG {
        let pc = unsafe { (&*(m as *const Machine)).cpu.regs[15] };
        if STOP_PC.with(Cell::get) == pc {
            return gba_core::capi::EVENT_YIELD;
        }
        let budget = (RT_API.guard)(m, a, len, expect);
        if budget & gba_core::capi::EVENT_YIELD != 0 {
            return budget;
        }
        let machine = unsafe { &*(m as *const Machine) };
        let off = (pc & 0x01ffffff) as usize;
        let bytes = if pc >> 24 == 3 {
            &machine.bus.iwram[(pc & 0x7fff) as usize..][..2]
        } else {
            &machine.bus.rom[off..off + 2]
        };
        let instr = armv4t::decode_thumb(u16::from_le_bytes(bytes.try_into().unwrap()), pc);
        let cost = gba_core::machine_instr_cost((pc >> 24) as usize & 15, &instr) as u32;
        budget.min(cost.saturating_add(1))
    } else {
        (RT_API.guard)(m, a, len, expect)
    }
}
fn usage(m: *mut std::ffi::c_void) -> [u64; 6] {
    let resources = unsafe { (&*(m as *const Machine)).bus.rom_resources().unwrap() };
    let i = resources.unit_obj_initial_pose_stats().unwrap();
    [
        i.directory_bytes,
        i.index_bytes,
        i.sequence_header_bytes,
        i.frame_bytes,
        i.object_bytes,
        i.pixel_bytes,
    ]
}
fn count(before: [u64; 6], after: [u64; 6]) {
    COMPILED.with(|v| {
        let mut x = v.get();
        for ((value, before), after) in x.iter_mut().zip(before).zip(after) {
            *value += after - before;
        }
        v.set(x);
    });
}
extern "C" fn read8(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let b = usage(m);
    let v = (RT_API.read8)(m, a);
    count(b, usage(m));
    v
}
extern "C" fn read16(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let b = usage(m);
    let v = (RT_API.read16)(m, a);
    count(b, usage(m));
    v
}
extern "C" fn read32(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let b = usage(m);
    let v = (RT_API.read32)(m, a);
    count(b, usage(m));
    v
}

fn names() -> [&'static str; 53] {
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
    ]
}
