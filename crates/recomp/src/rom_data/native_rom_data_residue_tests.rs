//! Opt-in native API address inventory. No production observer or timing capture.
use gba_core::{
    capi::{RtApi, RT_API},
    rom_data::Manifest,
    Machine,
};
use std::{cell::RefCell, collections::BTreeSet, ffi::c_void, fs, path::PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
struct Read {
    guest_address: u32,
    physical_offset: u32,
    width: u8,
    image_mask: u8,
    unowned_mask: u8,
    // These are synchronization/dispatch contexts, not proven instruction sites.
    exported_pc: u32,
    outer_dispatch: u32,
}

#[cfg(feature = "rom-data-read-audit")]
#[derive(Clone, Debug, serde::Serialize)]
struct FiragaQueueWrite {
    sequence: u64,
    frame: u64,
    address: u32,
    value: u32,
    registers: [u32; 16],
    cpsr: u32,
    outer_dispatch: u32,
    queue_head: Option<u32>,
    descriptor_bytes: Option<Vec<u8>>,
}

#[derive(Default)]
struct Observation {
    original: Option<std::ptr::NonNull<crate::BlockTable>>,
    outer_dispatch: u32,
    rom_len: u32,
    owned: Vec<(u32, u32)>,
    target_range: Option<(u32, u32)>,
    reads: BTreeSet<Read>,
    #[cfg(feature = "rom-data-read-audit")]
    firaga_queue_trace: bool,
    #[cfg(feature = "rom-data-read-audit")]
    write_sequence: u64,
    #[cfg(feature = "rom-data-read-audit")]
    firaga_source_stores: Vec<FiragaQueueWrite>,
    #[cfg(feature = "rom-data-read-audit")]
    firaga_queue_snapshots: Vec<FiragaQueueWrite>,
    #[cfg(feature = "rom-data-read-audit")]
    firaga_queue_overflow: bool,
}

thread_local! { static OBSERVATION: RefCell<Observation> = RefCell::new(Observation::default()); }

// Reset the raw table pointer even if an assertion outside the C ABI unwinds.
struct Scope;
impl Drop for Scope {
    fn drop(&mut self) {
        OBSERVATION.with(|cell| *cell.borrow_mut() = Observation::default());
    }
}

fn record(pointer: *mut c_void, address: u32, width: u8) {
    if !(0x08..=0x0d).contains(&(address >> 24)) {
        return;
    }
    let offset = address & 0x01ff_ffff;
    OBSERVATION.with(|cell| {
        let mut observation = cell.borrow_mut();
        if offset >= observation.rom_len {
            return; // Physical-image inventory excludes cartridge open bus/EEPROM.
        }
        if observation.target_range.is_some_and(|(start, end)| {
            offset >= end || offset.saturating_add(u32::from(width)) <= start
        }) {
            return;
        }
        let mut image_mask = 0;
        let mut unowned_mask = 0;
        for byte in 0..width {
            let off = offset + u32::from(byte);
            if off < observation.rom_len {
                image_mask |= 1 << byte;
                let index = observation.owned.partition_point(|&(_, end)| end <= off);
                if !observation
                    .owned
                    .get(index)
                    .is_some_and(|&(start, _)| start <= off)
                {
                    unowned_mask |= 1 << byte;
                }
            }
        }
        let exported_pc = unsafe { (*(pointer as *const Machine)).cpu.regs[15] };
        let outer_dispatch = observation.outer_dispatch;
        observation.reads.insert(Read {
            guest_address: address,
            physical_offset: offset,
            width,
            image_mask,
            unowned_mask,
            exported_pc,
            outer_dispatch,
        });
    });
}

extern "C" fn read8(pointer: *mut c_void, address: u32) -> u32 {
    record(pointer, address, 1);
    (RT_API.read8)(pointer, address)
}
extern "C" fn read16(pointer: *mut c_void, address: u32) -> u32 {
    record(pointer, address, 2);
    (RT_API.read16)(pointer, address)
}
extern "C" fn read32(pointer: *mut c_void, address: u32) -> u32 {
    record(pointer, address, 4);
    (RT_API.read32)(pointer, address)
}

#[cfg(feature = "rom-data-read-audit")]
fn dma_write_context(pointer: *mut c_void, address: u32, width: u8, value: u32)
    -> Option<gba_core::rom_data::read_audit::NativeDmaWrite>
{
    let machine = unsafe { &*(pointer as *const Machine) };
    let outer_dispatch = OBSERVATION.with(|cell| {
        let mut observation = cell.borrow_mut();
        if observation.firaga_queue_trace {
            observation.write_sequence += 1;
            let source_store = width == 4 && value == 0x0840_e844 && matches!(address >> 24, 2 | 3);
            let dma_source = width == 4 && address == 0x0400_00d4 && value == 0x0840_e844;
            if source_store || dma_source {
                // Inspect backing storage only: this diagnostic never calls the guest bus.
                let backing = |at: u32, len: usize| -> Option<&[u8]> {
                    let (memory, index) = match at >> 24 {
                        2 => (&machine.bus.ewram, (at & 0x3ffff) as usize),
                        3 => (&machine.bus.iwram, (at & 0x7fff) as usize),
                        _ => return None,
                    };
                    memory.get(index..index.checked_add(len)?)
                };
                let queue_head = backing(0x0300_2bcc, 4)
                    .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()));
                let event = FiragaQueueWrite {
                    sequence: observation.write_sequence, frame: machine.bus.frames, address, value,
                    registers: machine.cpu.regs, cpsr: machine.cpu.cpsr,
                    outer_dispatch: observation.outer_dispatch, queue_head,
                    descriptor_bytes: backing(machine.cpu.regs[4], 0x1c).map(<[u8]>::to_vec),
                };
                let entries = if source_store {
                    &mut observation.firaga_source_stores
                } else {
                    &mut observation.firaga_queue_snapshots
                };
                if entries.len() < 512 { entries.push(event); }
                else { observation.firaga_queue_overflow = true; }
            }
        }
        observation.outer_dispatch
    });
    if address >= 0x0400_00e0 || address.saturating_add(u32::from(width)) <= 0x0400_00b0 {
        return None;
    }
    Some(gba_core::rom_data::read_audit::NativeDmaWrite::enter(
        gba_core::rom_data::read_audit::NativeDmaWriteContext {
            address, width, value, exported_pc: machine.cpu.regs[15], outer_dispatch,
            lr: machine.cpu.regs[14], cpsr: machine.cpu.cpsr,
            args: machine.cpu.regs[..4].try_into().unwrap(),
        },
    ))
}
extern "C" fn write8(pointer: *mut c_void, address: u32, value: u32) {
    #[cfg(feature = "rom-data-read-audit")]
    let _context = dma_write_context(pointer, address, 1, value);
    (RT_API.write8)(pointer, address, value);
}
extern "C" fn write16(pointer: *mut c_void, address: u32, value: u32) {
    #[cfg(feature = "rom-data-read-audit")]
    let _context = dma_write_context(pointer, address, 2, value);
    (RT_API.write16)(pointer, address, value);
}
extern "C" fn write32(pointer: *mut c_void, address: u32, value: u32) {
    #[cfg(feature = "rom-data-read-audit")]
    let _context = dma_write_context(pointer, address, 4, value);
    (RT_API.write32)(pointer, address, value);
}

const OBSERVED_API: RtApi = RtApi {
    read8,
    read16,
    read32,
    write8,
    write16,
    write32,
    ..RT_API
};

extern "C" fn enter(_api: *const RtApi, pointer: *mut c_void) -> u32 {
    let machine = unsafe { &*(pointer as *const Machine) };
    let key = machine.cpu.regs[15] | machine.cpu.thumb() as u32;
    let original = OBSERVATION.with(|cell| {
        let mut observation = cell.borrow_mut();
        observation.outer_dispatch = key;
        // The Box and native library outlive Scope and every trampoline call.
        unsafe { observation.original.unwrap().as_ref() }
            .get(key)
            .unwrap()
    });
    // Generated internal chains retain this API; interpreter/host reads do not.
    original(&OBSERVED_API, pointer)
}

fn wrapped(original: &crate::BlockTable) -> crate::BlockTable {
    let entries = |values: &[Option<crate::BlockFn>]| {
        values
            .iter()
            .map(|value| value.map(|_| enter as crate::BlockFn))
            .collect()
    };
    crate::BlockTable {
        source: None,
        rom: entries(&original.rom),
        iwram: entries(&original.iwram),
        ewram: entries(&original.ewram),
        bios: entries(&original.bios),
        other: original
            .other
            .keys()
            .map(|&key| (key, enter as crate::BlockFn))
            .collect(),
        len: original.len,
    }
}

fn owned_ranges(manifest: &Manifest) -> Vec<(u32, u32)> {
    let mut ranges: Vec<_> = manifest
        .resources
        .iter()
        .flat_map(|r| &r.sources)
        .map(|source| {
            let end = source.offset.checked_add(source.length).unwrap();
            assert!(end <= manifest.source_rom_size);
            (source.offset, end)
        })
        .filter(|&(start, end)| start < end)
        .collect();
    ranges.sort_unstable();
    let mut merged: Vec<(u32, u32)> = Vec::new();
    for (start, end) in ranges {
        if let Some(last) = merged.last_mut().filter(|last| start <= last.1) {
            last.1 = last.1.max(end);
        } else {
            merged.push((start, end));
        }
    }
    merged
}

fn clear_audio(machine: &mut Machine) {
    machine.bus.audio_buf.clear();
    machine.bus.psg_tap.clear();
    machine.bus.hle_tap.clear();
    machine.bus.fifo_tap.iter_mut().for_each(Vec::clear);
}

#[test]
#[ignore = "requires exact private EUR ROM/native/Bank fixtures, accepted dataset and fresh report path"]
fn native_api_rom_residue_preserves_bounded_scene() {
    crate::configure_headless_widescreen();
    let path = |key| PathBuf::from(std::env::var_os(key).expect(key));
    let report = path("FFTA_ROM_RESIDUE_REPORT");
    assert!(!report.exists(), "preserve earlier inventory reports");
    let rom = fs::read(path("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let dataset = path("FFTA_STATE_TEST_DATASET");
    let manifest: Manifest =
        toml::from_str(&fs::read_to_string(dataset.join("manifest.toml")).unwrap()).unwrap();
    assert_eq!(manifest.source_rom_sha256, gba_core::ffta_rom::digest(&rom));
    assert_eq!(manifest.source_rom_size as usize, rom.len());
    let (_library, table) = crate::load_native(&path("FFTA_STATE_TEST_NATIVE")).unwrap();
    let original = Box::new(table);
    let observed_table = wrapped(&original);
    let input = crate::InputScript::load(path("FFTA_STATE_TEST_INPUT").to_str().unwrap()).unwrap();
    let mut reference = crate::make_machine(rom.clone(), None);
    let mut observed = crate::make_machine(rom, None);
    for machine in [&mut reference, &mut observed] {
        super::attach_from_directory(machine, &dataset).unwrap();
        crate::save_state::load(machine, &path("FFTA_STATE_TEST_STATE")).unwrap();
        crate::arm_audio_hle(machine, None);
        machine.bus.tap_channels = true;
        clear_audio(machine);
    }
    assert!(reference.save_state().unwrap() == observed.save_state().unwrap());
    OBSERVATION.with(|cell| {
        assert!(cell.borrow().original.is_none());
        *cell.borrow_mut() = Observation {
            original: Some(std::ptr::NonNull::from(original.as_ref())),
            rom_len: manifest.source_rom_size,
            owned: owned_ranges(&manifest),
            ..Observation::default()
        };
    });
    let _scope = Scope;
    let first = reference.bus.frames;
    let frames = 12;
    let mut dispatch = [0u64; 2];
    let mut audio = [0u64; 5];
    for relative in 0..frames {
        let mut counts = [(0, 0); 2];
        for (index, (machine, table)) in [
            (&mut reference, original.as_ref()),
            (&mut observed, &observed_table),
        ]
        .into_iter()
        .enumerate()
        {
            machine.bus.keys = input.keys_at(relative);
            input.apply_mouse(&mut machine.bus, relative).unwrap();
            let pointer = machine as *mut Machine as *mut c_void;
            counts[index] =
                crate::run_frame_native(machine, table, pointer, crate::FRAME_STEP_GUARD);
            assert_eq!(machine.bus.frames, first + relative + 1, "incomplete frame");
            assert!(machine.bus.frame_ready);
        }
        assert_eq!(counts[0], counts[1], "dispatch at {relative}");
        dispatch[0] += counts[0].0;
        dispatch[1] += counts[0].1;
        assert_eq!(
            reference.bus.framebuffer, observed.bus.framebuffer,
            "central at {relative}"
        );
        assert_eq!(
            reference.bus.wide_framebuffer, observed.bus.wide_framebuffer,
            "wide at {relative}"
        );
        assert_eq!(reference.bus.oam, observed.bus.oam, "OAM at {relative}");
        assert_eq!(
            reference.bus.audio_buf, observed.bus.audio_buf,
            "mixed at {relative}"
        );
        assert_eq!(
            reference.bus.psg_tap, observed.bus.psg_tap,
            "PSG at {relative}"
        );
        assert_eq!(
            reference.bus.fifo_tap, observed.bus.fifo_tap,
            "FIFO at {relative}"
        );
        assert_eq!(reference.bus.hle_tap.len(), observed.bus.hle_tap.len());
        assert!(
            reference
                .bus
                .hle_tap
                .iter()
                .zip(&observed.bus.hle_tap)
                .all(|(a, b)| a.to_bits() == b.to_bits()),
            "HLE at {relative}"
        );
        assert!(
            reference.save_state().unwrap() == observed.save_state().unwrap(),
            "full state/clock at {relative}"
        );
        for (sum, length) in audio.iter_mut().zip([
            observed.bus.audio_buf.len(),
            observed.bus.psg_tap.len(),
            observed.bus.fifo_tap[0].len(),
            observed.bus.fifo_tap[1].len(),
            observed.bus.hle_tap.len(),
        ]) {
            *sum += length as u64;
        }
        clear_audio(&mut reference);
        clear_audio(&mut observed);
    }
    assert!(dispatch[0] > 0 && audio.iter().all(|&length| length > 0));
    OBSERVATION.with(|cell| {
        let observation = cell.borrow();
        assert!(!observation.reads.is_empty(), "observer must see native ROM API reads");
        let mut bytes = BTreeSet::new();
        let mut unowned_bytes = BTreeSet::new();
        for read in &observation.reads {
            for byte in 0..read.width {
                if read.image_mask & (1 << byte) != 0 { bytes.insert(read.physical_offset + u32::from(byte)); }
                if read.unowned_mask & (1 << byte) != 0 { unowned_bytes.insert(read.physical_offset + u32::from(byte)); }
            }
        }
        let report_value = serde_json::json!({
            "schema": 1, "frames": frames, "first_guest_frame": first,
            "last_guest_frame": observed.bus.frames, "source_rom_sha256": manifest.source_rom_sha256,
            "extractor_revision": manifest.extractor_revision, "decoder_revision": manifest.decoder_revision,
            "resource_count": manifest.resources.len(), "merged_owned_ranges": observation.owned.len(),
            "unique_context_reads": observation.reads.len(), "unique_image_bytes": bytes.len(),
            "unique_unowned_bytes": unowned_bytes.len(), "dispatch_coverage_only": dispatch,
            "audio_samples": audio, "per_frame_full_state_clock_buffers_dispatch_audio_equal": true,
            "reads": observation.reads, "performance_measurements": false,
            "scope": "native read8/16/32 API requests beginning inside the physical ROM image; no read values",
            "blind_spots": ["interpreter fallback", "host/HLE/direct DMA reads", "cross-region requests beginning outside ROM", "other scenes"],
            "context_limit": "exported PC may be stale; outer dispatch is not an exact LDR site; unowned bytes are leads, not decoded asset boundaries"
        });
        fs::write(&report, serde_json::to_vec_pretty(&report_value).unwrap()).unwrap();
        eprintln!("ROM_DATA_NATIVE_RESIDUE frames={frames} contexts={} image_bytes={} unowned_bytes={} transparency=PASS", observation.reads.len(), bytes.len(), unowned_bytes.len());
    });
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact private EUR ROM and recorded scene; Wine only, Linux/USA pending"]
fn european_soil_14d6d8_native_api_context() {
    use gba_core::rom_data::read_audit::SiteCapture;

    crate::configure_headless_widescreen();
    let path = |key| PathBuf::from(std::env::var_os(key).expect(key));
    let report = path("FFTA_SOIL_14D6D8_REPORT");
    assert!(!report.exists(), "preserve earlier source-site reports");
    let rom = fs::read(path("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let dataset = path("FFTA_STATE_TEST_DATASET");
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join("manifest.toml")).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (239, 238));
    let owned = owned_ranges(&manifest);
    let target = (0x14d6d8u32, 0x14d6e0u32);
    assert!(owned.iter().all(|&(start, end)| end <= target.0 || start >= target.1));

    let (_library, table) = crate::load_native(&path("FFTA_STATE_TEST_NATIVE")).unwrap();
    let original = Box::new(table);
    let observed_table = wrapped(&original);
    let input = crate::InputScript::load(path("FFTA_STATE_TEST_INPUT").to_str().unwrap()).unwrap();
    let mut machine = crate::make_machine(rom, None);
    super::attach_from_directory(&mut machine, &dataset).unwrap();
    crate::save_state::load(&mut machine, &path("FFTA_STATE_TEST_STATE")).unwrap();
    crate::arm_audio_hle(&mut machine, None);
    machine.bus.tap_channels = true;
    clear_audio(&mut machine);
    OBSERVATION.with(|cell| {
        assert!(cell.borrow().original.is_none());
        *cell.borrow_mut() = Observation {
            original: Some(std::ptr::NonNull::from(original.as_ref())),
            rom_len: manifest.source_rom_size,
            owned,
            target_range: Some(target),
            ..Observation::default()
        };
    });
    let _scope = Scope;
    let first = machine.bus.frames;
    let sites = SiteCapture::start([target.0, target.0 + 4]);
    for relative in 0..600 {
        machine.bus.keys = input.keys_at(relative);
        input.apply_mouse(&mut machine.bus, relative).unwrap();
        let pointer = &mut machine as *mut Machine as *mut c_void;
        crate::run_frame_native(&mut machine, &observed_table, pointer, crate::FRAME_STEP_GUARD);
        assert_eq!(machine.bus.frames, first + relative + 1);
        clear_audio(&mut machine);
    }
    let sites = sites.finish();
    let mut sampled = BTreeSet::new();
    for site in &sites {
        for byte in 0..site.read.length {
            let at = site.read.physical_offset + u32::try_from(byte).unwrap();
            if (target.0..target.1).contains(&at) { sampled.insert(at); }
        }
    }
    assert_eq!(sampled, (target.0..target.1).collect(), "both selected words must remain observed");
    OBSERVATION.with(|cell| {
        let observation = cell.borrow();
        let value = serde_json::json!({
            "status": "PASS", "frames": 600, "first_guest_frame": first,
            "last_guest_frame": machine.bus.frames, "revision": [239, 238],
            "target_start": target.0, "target_end": target.1,
            "sampled_bytes": sampled.len(), "native_context_reads": observation.reads,
            "site_reads": sites, "performance_measurements": false,
            "scope": "one EUR continuation; filtered native read API and bus-site evidence",
            "limits": "Native exported PC may be stale and outer dispatch may include chained blocks; neither is an exact instruction site. Interpreter/host/HLE reads are not native API calls. No state equivalence or owner is established.",
            "Linux": "pending", "USA": "pending", "native_windows": "pending"
        });
        fs::write(&report, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
        eprintln!("ROM_DATA_SOIL_14D6D8_CONTEXT sampled_bytes={} native_context_reads={} PASS", sampled.len(), observation.reads.len());
    });
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact private EUR ROM and recorded scene; Wine only, Linux/USA pending"]
fn european_firaga_40e844_native_api_context() {
    use gba_core::rom_data::read_audit::{DmaCapture, SiteCapture};

    crate::configure_headless_widescreen();
    let path = |key| PathBuf::from(std::env::var_os(key).expect(key));
    let report = path("FFTA_FIRAGA_40E844_REPORT");
    assert!(!report.exists(), "preserve earlier source-site reports");
    let rom = fs::read(path("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let dataset = path("FFTA_STATE_TEST_DATASET");
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join("manifest.toml")).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (260, 259));
    let owned = owned_ranges(&manifest);
    let target = (0x40e844u32, 0x40e944u32);
    assert!(owned.iter().all(|&(start, end)| end <= target.0 || start >= target.1));

    // Synthetic immediate DMA: observation must leave complete state and guest clock unchanged.
    let mut baseline = crate::make_machine(rom.clone(), None);
    let mut observed = crate::make_machine(rom.clone(), None);
    let program = |machine: &mut Machine, api: &RtApi| {
        let pointer = machine as *mut Machine as *mut c_void;
        (api.write32)(pointer, 0x0400_00d4, 0x0840_e844);
        (api.write32)(pointer, 0x0400_00d8, 0x0201_0000);
        (api.write16)(pointer, 0x0400_00dc, 8);
        (api.write16)(pointer, 0x0400_00de, 0x8000);
    };
    program(&mut baseline, &RT_API);
    let synthetic = DmaCapture::start(target.0, target.0 + 16, 64);
    program(&mut observed, &OBSERVED_API);
    let synthetic = synthetic.finish();
    assert!(!synthetic.overflow && synthetic.non_dma_requests == 0 && synthetic.dma_requests == 8);
    assert_eq!(synthetic.transfers.len(), 1);
    let transfer = &synthetic.transfers[0];
    assert_eq!((transfer.transfer.channel, transfer.transfer.initial_src, transfer.transfer.initial_dst,
        transfer.transfer.count, transfer.transfer.unit, transfer.transfer.timing),
        (3, 0x0840_e844, 0x0201_0000, 8, 2, 0));
    assert_eq!(transfer.target_bytes.len(), 16);
    assert!(transfer.transfer.source_writer.and_then(|v| v.native).is_some());
    assert!(transfer.transfer.enable_writer.and_then(|v| v.native).is_some());
    assert_eq!(baseline.save_state().unwrap(), observed.save_state().unwrap(), "DMA observation must preserve state and clock");

    let (_library, table) = crate::load_native(&path("FFTA_STATE_TEST_NATIVE")).unwrap();
    let original = Box::new(table);
    let observed_table = wrapped(&original);
    let input = crate::InputScript::load(path("FFTA_STATE_TEST_INPUT").to_str().unwrap()).unwrap();
    let mut machine = crate::make_machine(rom, None);
    super::attach_from_directory(&mut machine, &dataset).unwrap();
    crate::save_state::load(&mut machine, &path("FFTA_STATE_TEST_STATE")).unwrap();
    crate::arm_audio_hle(&mut machine, None);
    machine.bus.tap_channels = true;
    clear_audio(&mut machine);
    OBSERVATION.with(|cell| {
        assert!(cell.borrow().original.is_none());
        *cell.borrow_mut() = Observation {
            original: Some(std::ptr::NonNull::from(original.as_ref())),
            rom_len: manifest.source_rom_size,
            owned,
            target_range: Some(target),
            firaga_queue_trace: true,
            ..Observation::default()
        };
    });
    let _scope = Scope;
    let first = machine.bus.frames;
    let sites = SiteCapture::start((target.0..target.1).step_by(4));
    let dma = DmaCapture::start(target.0, target.1, 4096);
    for relative in 0..530 {
        machine.bus.keys = input.keys_at(relative);
        input.apply_mouse(&mut machine.bus, relative).unwrap();
        let pointer = &mut machine as *mut Machine as *mut c_void;
        crate::run_frame_native(&mut machine, &observed_table, pointer, crate::FRAME_STEP_GUARD);
        assert_eq!(machine.bus.frames, first + relative + 1);
        clear_audio(&mut machine);
    }
    let dma = dma.finish();
    let sites = sites.finish();
    assert!(!dma.overflow, "bounded DMA provenance capture overflowed");
    assert_eq!(dma.target_requests, dma.dma_requests + dma.non_dma_requests);
    let mut sampled = BTreeSet::new();
    for site in &sites {
        for byte in 0..site.read.length {
            let at = site.read.physical_offset + u32::try_from(byte).unwrap();
            if (target.0..target.1).contains(&at) { sampled.insert(at); }
        }
    }
    assert_eq!(sampled, (target.0..target.1).collect(), "selected byte range must remain observed");
    OBSERVATION.with(|cell| {
        let observation = cell.borrow();
        assert!(!observation.firaga_queue_overflow, "bounded queue trace overflowed");
        let value = serde_json::json!({
            "status": "PASS", "frames": 530, "first_guest_frame": first,
            "last_guest_frame": machine.bus.frames, "revision": [260, 259],
            "target_start": target.0, "target_end": target.1,
            "synthetic_dma_state_clock_equal": true, "dma_capture": dma,
            "queue_trace_overflow": observation.firaga_queue_overflow,
            "source_stores": observation.firaga_source_stores,
            "queue_snapshots": observation.firaga_queue_snapshots,
            "sampled_bytes": sampled.len(), "native_context_reads": observation.reads,
            "site_reads": sites, "performance_measurements": false,
            "scope": "one EUR continuation; filtered native read API and bus-site evidence",
            "limits": "Native exported PC may be stale and outer dispatch may include chained blocks; neither is an exact instruction site. Interpreter/host/HLE reads are not native API calls. No state equivalence or owner is established.",
            "Linux": "pending", "USA": "pending", "native_windows": "pending"
        });
        fs::write(&report, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
        eprintln!("ROM_DATA_FIRAGA_40E844_CONTEXT sampled_bytes={} native_context_reads={} PASS", sampled.len(), observation.reads.len());
    });
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact private EUR ROM and recorded White Flowers scene; Wine only, Linux/USA pending"]
fn european_white_flowers_397b1e_native_context() {
    use gba_core::rom_data::read_audit::{DmaCapture, SiteCapture};

    crate::configure_headless_widescreen();
    let path = |key| PathBuf::from(std::env::var_os(key).expect(key));
    let report = path("FFTA_WHITE_FLOWERS_397B1E_REPORT");
    assert!(!report.exists(), "preserve earlier source-site reports");
    let rom = fs::read(path("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let dataset = path("FFTA_STATE_TEST_DATASET");
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join("manifest.toml")).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (289, 288));
    let owned = owned_ranges(&manifest);
    let target = (0x397b1eu32, 0x397b21u32);
    assert!(owned.iter().all(|&(start, end)| end <= target.0 || start >= target.1));

    let (_library, table) = crate::load_native(&path("FFTA_STATE_TEST_NATIVE")).unwrap();
    let original = Box::new(table);
    let observed_table = wrapped(&original);
    let input = crate::InputScript::load(path("FFTA_STATE_TEST_INPUT").to_str().unwrap()).unwrap();
    let mut machine = crate::make_machine(rom, None);
    super::attach_from_directory(&mut machine, &dataset).unwrap();
    crate::save_state::load(&mut machine, &path("FFTA_STATE_TEST_STATE")).unwrap();
    crate::arm_audio_hle(&mut machine, None);
    machine.bus.tap_channels = true;
    clear_audio(&mut machine);
    OBSERVATION.with(|cell| {
        assert!(cell.borrow().original.is_none());
        *cell.borrow_mut() = Observation {
            original: Some(std::ptr::NonNull::from(original.as_ref())),
            rom_len: manifest.source_rom_size,
            owned,
            target_range: Some(target),
            ..Observation::default()
        };
    });
    let _scope = Scope;
    let first = machine.bus.frames;
    let sites = SiteCapture::start([0x397b1c, 0x397b20]);
    let dma = DmaCapture::start(target.0, target.1, 512);
    for relative in 0..600 {
        machine.bus.keys = input.keys_at(relative);
        input.apply_mouse(&mut machine.bus, relative).unwrap();
        let pointer = &mut machine as *mut Machine as *mut c_void;
        crate::run_frame_native(&mut machine, &observed_table, pointer, crate::FRAME_STEP_GUARD);
        assert_eq!(machine.bus.frames, first + relative + 1);
        clear_audio(&mut machine);
    }
    let sites = sites.finish();
    let dma = dma.finish();
    assert!(!dma.overflow, "DMA observation must remain complete");
    let mut sampled = BTreeSet::new();
    for site in &sites {
        for byte in 0..site.read.length {
            let at = site.read.physical_offset + u32::try_from(byte).unwrap();
            if (target.0..target.1).contains(&at) { sampled.insert(at); }
        }
    }
    assert_eq!(sampled, (target.0..target.1).collect(), "all three selected bytes must remain observed");
    OBSERVATION.with(|cell| {
        let observation = cell.borrow();
        let value = serde_json::json!({
            "status": "PASS", "frames": 600, "first_guest_frame": first,
            "last_guest_frame": machine.bus.frames, "revision": [289, 288],
            "target_start": target.0, "target_end": target.1,
            "sampled_bytes": sampled.len(), "native_context_reads": observation.reads,
            "site_reads": sites, "dma": dma, "performance_measurements": false,
            "scope": "one EUR continuation; filtered native read API, DMA and bus-site evidence",
            "limits": "Native exported PC may be stale and outer dispatch can include chained blocks; neither is an exact instruction site. Non-DMA can still be native or host. No source extent or owner is established.",
            "Linux": "pending", "USA": "pending", "native_windows": "pending"
        });
        fs::write(&report, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
        eprintln!("ROM_DATA_WHITE_FLOWERS_397B1E sampled_bytes={} native_context_reads={} PASS", sampled.len(), observation.reads.len());
    });
}
