//! Filtered execution-site evidence for newly observed low ROM words.
use gba_core::{
    rom_data::read_audit::{Capture, ExecutionSite, Reader, SiteCapture},
    Bus,
};
use std::{collections::BTreeSet, fs, path::PathBuf};

fn clear_audio(machine: &mut gba_core::Machine) {
    machine.bus.audio_buf.clear();
    machine.bus.psg_tap.clear();
    machine.bus.fifo_tap[0].clear();
    machine.bus.fifo_tap[1].clear();
    machine.bus.hle_tap.clear();
}

fn probe_sites(rom: &[u8]) {
    let mut m = crate::make_machine(rom.to_vec(), None);
    for cached in [false, true] {
        m.cpu.regs[15] = 0x08018dd0;
        m.cpu.cpsr = 0xbf;
        let addresses = Capture::start();
        let sites = SiteCapture::start([0x18df4]);
        if cached {
            let mut cache = gba_core::exec::DecodeCache::default();
            gba_core::exec::step_cached(&mut m.cpu, &mut m.bus, &mut cache);
        } else {
            gba_core::exec::step(&mut m.cpu, &mut m.bus);
        }
        let sites = sites.finish();
        let addresses = addresses.finish();
        assert_eq!(sites.len(), 1);
        let read = sites.first().unwrap();
        assert_eq!(read.read.reader, Reader::Bus);
        assert_eq!((read.read.physical_offset, read.read.length), (0x18df4, 4));
        assert_eq!(
            read.instruction,
            Some(ExecutionSite {
                pc: 0x08018dd0,
                thumb: true
            })
        );
        assert_eq!(
            addresses,
            BTreeSet::from([read.read]),
            "existing address-only keys unchanged"
        );
        assert_eq!(
            m.cpu.regs[1],
            u32::from_le_bytes(rom[0x18df4..0x18df8].try_into().unwrap())
        );
    }
    let sites = SiteCapture::start([0x18df4]);
    m.bus.read32(0x08018df4);
    let rows = sites.finish();
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows.first().unwrap().instruction,
        None,
        "execution scope restored after return"
    );
    let sites = SiteCapture::start([0x18df4]);
    m.bus.read32(0x08018df4);
    drop(sites);
    assert!(
        SiteCapture::start([0x18df4]).finish().is_empty(),
        "activation reset on drop"
    );
}

#[test]
#[ignore = "private EUR filtered source-reader sites; Wine only, no old scene equivalence replay"]
fn european_lower_rom_words_record_interpreter_sites() {
    crate::configure_headless_widescreen();
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_LOWER_SITE_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let words: Vec<u32> = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_LOWER_SITE_WORDS").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(words.len(), 113);
    assert_eq!(words.iter().copied().collect::<BTreeSet<_>>().len(), 113);
    probe_sites(&rom);
    let native = std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap();
    let (_library, table) = crate::load_native(std::path::Path::new(&native)).unwrap();
    let input = crate::InputScript::load(&std::env::var("FFTA_STATE_TEST_INPUT").unwrap()).unwrap();
    let mut m = crate::make_machine(rom, None);
    super::attach_from_directory(
        &mut m,
        &PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap()),
    )
    .unwrap();
    crate::save_state::load(
        &mut m,
        std::path::Path::new(&std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()),
    )
    .unwrap();
    crate::arm_audio_hle(&mut m, None);
    m.bus.tap_channels = true;
    clear_audio(&mut m);
    let first = m.bus.frames;
    let capture = SiteCapture::start(words.iter().copied());
    let mut dispatch = [0u64; 2];
    for relative in 0..12 {
        m.bus.keys = input.keys_at(relative);
        input.apply_mouse(&mut m.bus, relative).unwrap();
        let pointer = &mut m as *mut gba_core::Machine as *mut std::ffi::c_void;
        let (native, fallback) =
            crate::run_frame_native(&mut m, &table, pointer, crate::FRAME_STEP_GUARD);
        dispatch[0] += native;
        dispatch[1] += fallback;
        assert_eq!(m.bus.frames, first + relative + 1);
        clear_audio(&mut m);
    }
    let sites = capture.finish();
    assert!(!sites.is_empty());
    let observed: BTreeSet<_> = words
        .iter()
        .copied()
        .filter(|word| {
            sites.iter().any(|site| {
                let read = site.read;
                read.physical_offset < word + 4
                    && read.physical_offset as u64 + read.length as u64 > u64::from(*word)
            })
        })
        .collect();
    let with_scope = sites.iter().filter(|row| row.instruction.is_some()).count();
    let outside_scope = sites.len() - with_scope;
    let report = serde_json::json!({
        "task":"ROMDATA-01-BUS-HOST-LOWER-RESIDUE", "frames":12,"first_guest_frame":first,"last_guest_frame":m.bus.frames,
        "site_probes":"PASS/cached-uncached original ThumbLDR/outsideNone/drop/address-key preservation",
        "target_words":words.len(),"observed_words":observed.len(),"reads_with_instruction_scope":with_scope,"reads_outside_instruction_scope":outside_scope,
        "dispatch_coverage_only":dispatch,"sites":sites,
        "scope":"single currentdecoded source-site observation only; interpreter execution/HLE scope may include nested accesses, opcode/address validation required; native/host outside scope staysNone; no old scene equivalence acceptance",
        "linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false
    });
    fs::write(
        root.join("sites.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_LOWER_SITES target_words=113 observed_words={} instruction_scoped={with_scope} outside_scoped={outside_scope} PASS", observed.len());
}
