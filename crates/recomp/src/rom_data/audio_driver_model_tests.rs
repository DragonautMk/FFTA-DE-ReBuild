//! One focused EUR table decoder/original-reader check; previous tone/sequence acceptance stays closed.
use super::*;
use gba_core::{
    rom_data::{audio_driver::*, audio_sequences},
    Machine,
};
use serde_json::Value;
use std::collections::BTreeSet;
const CHANNEL: u32 = 0x02010000;
const RETURN: u32 = 0x08000300;
const STACK: u32 = 0x03007e00;
fn put(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}
fn reset(m: &mut Machine, pc: u32) {
    m.bus.ewram.fill(0);
    m.bus.iwram.fill(0);
    m.bus.iwram[0x7c00..0x7e20].fill(0x5a);
    put(&mut m.bus.iwram, 0x7ff0, 0x020084e0);
    m.cpu.regs = [0; 16];
    for n in 4..12 {
        m.cpu.regs[n] = 0x11110000 + n as u32;
    }
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = pc;
    m.cpu.cpsr = 0xbf;
    m.bus.reg_ie = 0;
    m.bus.ime = false;
}
fn until(
    m: &mut Machine,
    end: u32,
    pcs: &mut BTreeSet<(u32, bool)>,
    reads: &mut BTreeSet<u32>,
) -> usize {
    let mut n = 0;
    while m.cpu.regs[15] != end {
        assert!(n < 1500, "pc{:08x}", m.cpu.regs[15]);
        let pc = m.cpu.regs[15];
        let len = match pc {
            0x08148a74 | 0x08148a8c | 0x0814953e | 0x0814956a | 0x08149586 | 0x08149a38 => 1,
            0x0814957a | 0x08149592 | 0x08148fe8 => 2,
            0x08148a84 | 0x08148a98 => 4,
            _ => 0,
        };
        for i in 0..len {
            let at = m.cpu.regs[0] + i;
            assert!((0x08000000..0x09000000).contains(&at));
            reads.insert(at - 0x08000000);
        }
        pcs.insert((pc, m.cpu.cpsr & 32 != 0));
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    n
}
fn outside_stack(m: &Machine, before: &[u8], scratch: usize) {
    assert_eq!(
        &m.bus.iwram[..0x7e00 - scratch],
        &before[..0x7e00 - scratch]
    );
    assert_eq!(&m.bus.iwram[0x7e00..], &before[0x7e00..]);
}
fn returned(m: &Machine, ewram: &[u8], iwram: &[u8]) {
    assert_eq!(m.cpu.regs[13], STACK);
    assert_eq!(m.bus.ewram, ewram);
    outside_stack(m, iwram, 20);
    for n in 4..12 {
        assert_eq!(m.cpu.regs[n], 0x11110000 + n as u32);
    }
}
#[test]
#[ignore = "private EUR M4A driver table decoder/pitch/rate/volume source readers; Wine only, Linux/USA pending"]
fn european_audio_driver_model_matches_original_integer_readers_and_aliases() {
    let out = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DRIVER_ROOT").unwrap());
    assert!(!out.exists());
    let evidence = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DRIVER_EVIDENCE").unwrap());
    let independent: Value =
        serde_json::from_slice(&fs::read(evidence.join("driver-independent.json")).unwrap())
            .unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let cat = Catalog::from_rom(&rom).unwrap();
    let data = &cat.decoded;
    let sequence: audio_sequences::AudioSequences = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_DRIVER_SEQUENCE_OWNER").unwrap()).unwrap(),
    )
    .unwrap();
    data.validate_sequence_owner(&sequence).unwrap();
    let tables = independent["tables"].as_array().unwrap();
    assert_eq!(tables.len(), 7);
    let spans = data.source_spans().unwrap();
    assert_eq!(cat.sources.len(), 7);
    assert_eq!(spans.iter().map(|s| s.1.len()).sum::<usize>(), 484);
    for (id, ((at, bytes, role), e)) in spans.iter().zip(tables).enumerate() {
        assert_eq!(
            (*at, bytes.len() as u64),
            (
                e["offset"].as_u64().unwrap() as u32,
                e["length"].as_u64().unwrap()
            )
        );
        assert_eq!(
            gba_core::ffta_rom::digest(bytes),
            e["sha256"].as_str().unwrap()
        );
        assert_eq!(bytes, &rom[*at as usize..*at as usize + bytes.len()]);
        assert!(role.starts_with("audio/driver-lut/"));
        assert_eq!(data.byte(id, bytes.len()), None);
        assert_eq!(data.byte(id, usize::MAX), None);
        for (n, &v) in bytes.iter().enumerate() {
            assert_eq!(data.source_byte(*at + n as u32), Some(v));
        }
    }
    assert_eq!(
        serde_json::to_value(&data.pcm_keys.entries).unwrap(),
        tables[0]["values"]
    );
    assert_eq!(
        serde_json::to_value(&data.pcm_bases.entries).unwrap(),
        tables[1]["values"]
    );
    assert_eq!(
        serde_json::to_value(&data.sample_counts.entries).unwrap(),
        tables[2]["values"]
    );
    assert_eq!(
        serde_json::to_value(&data.cgb_keys.entries).unwrap(),
        tables[3]["values"]
    );
    assert_eq!(
        serde_json::to_value(&data.cgb_bases.entries).unwrap(),
        tables[4]["values"]
    );
    assert_eq!(
        serde_json::to_value(&data.noise_codes.entries).unwrap(),
        tables[5]["values"]
    );
    assert_eq!(
        serde_json::to_value(&data.wave_volume_codes.entries).unwrap(),
        tables[6]["values"]
    );
    for selector in 0..16 {
        let at = 0x370c04i64 + 2 * (selector - 1);
        let expected = u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap());
        assert_eq!(
            data.sample_count_lookup((selector as u32) << 16),
            Some(expected)
        );
    }
    for level in [16, 64, 255] {
        assert_eq!(data.wave_volume_code(level), None);
    }
    assert_eq!(data.source_byte(0x370b1f), None);
    assert_eq!(data.source_byte(0x370d04), None);
    let bytes = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<AudioDriver>(&bytes).unwrap(),
        *data
    );
    let mut unknown = serde_json::to_value(data).unwrap();
    unknown
        .as_object_mut()
        .unwrap()
        .insert("unknown".into(), true.into());
    assert!(serde_json::from_value::<AudioDriver>(unknown).is_err());
    assert!(serde_json::from_str::<KeyCode>("{\"base\":0,\"shift\":0,\"unknown\":true}").is_err());
    assert!(
        serde_json::from_str::<Table<i16>>("{\"source_offset\":0,\"entries\":[32768]}").is_err()
    );
    for case in 0..12 {
        let mut bad = data.clone();
        match case {
            0 => bad.pcm_keys.entries[0].base = 12,
            1 => bad.pcm_keys.entries[0].shift = 16,
            2 => {
                bad.pcm_keys.entries.pop();
            }
            3 => {
                bad.pcm_bases.entries.pop();
            }
            4 => {
                bad.sample_counts.entries.pop();
            }
            5 => {
                bad.cgb_keys.entries.pop();
            }
            6 => {
                bad.cgb_bases.entries.pop();
            }
            7 => {
                bad.noise_codes.entries.pop();
            }
            8 => {
                bad.wave_volume_codes.entries.pop();
            }
            9 => bad.pcm_keys.source_offset += 1,
            10 => bad.sample_counts.source_offset -= 2,
            _ => bad.wave_volume_codes.source_offset += 1,
        }
        assert!(bad.validate().is_err(), "shape guard {case}");
    }
    let mut edited = data.clone();
    edited.wave_volume_codes.entries[0] ^= 1;
    edited.validate().unwrap();
    assert!(edited.verify_source(&rom).is_err());
    let mut edited = data.clone();
    edited.cgb_bases.entries[0] ^= 0x100;
    edited.validate().unwrap();
    assert!(edited.verify_source(&rom).is_err());
    let mut bad_owner = sequence.clone();
    bad_owner.driver.clock_source_offset += 1;
    assert!(data.validate_sequence_owner(&bad_owner).is_err());
    assert!(Catalog::from_rom(&[]).is_err());
    let mut m = Machine::new(rom.clone());
    let mut pcs = BTreeSet::new();
    let mut reads = BTreeSet::new();
    let mut instructions = 0;
    let mut pcm_cases = BTreeSet::new();
    let wave_word = 14069760u32;
    for key in 0..256 {
        for fraction in [0, 255] {
            pcm_cases.insert((wave_word, key, fraction));
        }
    }
    for key in [0, 11, 12, 35, 60, 120, 177, 178, 179, 255] {
        pcm_cases.insert((wave_word, key, 127));
    }
    for word in [0, 1, 0x80000000, 0xffffffff] {
        for (key, fraction) in [(0, 0), (60, 127), (178, 1), (255, 255)] {
            pcm_cases.insert((word, key, fraction));
        }
    }
    for (key, fraction) in [
        (0x1234003c, 0xabcdef7f),
        (0xffff00b2, 0x12340001),
        (0xdeadbeef, 0xffffff00),
    ] {
        pcm_cases.insert((wave_word, key, fraction));
    }
    for &(word, key, fraction) in &pcm_cases {
        reset(&mut m, 0x08148a5c);
        put(&mut m.bus.ewram, 0x10004, word);
        m.cpu.regs[..3].copy_from_slice(&[CHANNEL, key, fraction]);
        let ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        instructions += until(&mut m, RETURN, &mut pcs, &mut reads);
        assert_eq!(
            m.cpu.regs[0],
            data.pcm_frequency_word(word, key, fraction).unwrap(),
            "PCM {word:x} {key:x} {fraction:x}"
        );
        returned(&m, &ew, &iw);
    }
    let mut tonal_cases = BTreeSet::new();
    for key in 0..256 {
        tonal_cases.insert((3, key, 0));
    }
    for key in [0, 35, 36, 47, 48, 165, 166, 167, 255] {
        for fraction in [1, 127, 255] {
            tonal_cases.insert((3, key, fraction));
        }
    }
    for kind in [0, 1, 2, 255] {
        for key in [35, 36, 166, 255] {
            tonal_cases.insert((kind, key, 127));
        }
    }
    tonal_cases.insert((0x12340003, 0xabcdef24, 0x123456ff));
    for &(kind, key, fraction) in &tonal_cases {
        reset(&mut m, 0x08149510);
        m.cpu.regs[..3].copy_from_slice(&[kind, key, fraction]);
        let ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        instructions += until(&mut m, RETURN, &mut pcs, &mut reads);
        assert_eq!(
            m.cpu.regs[0],
            data.cgb_frequency_word(kind, key, fraction).unwrap(),
            "CGB {kind:x} {key:x} {fraction:x}"
        );
        returned(&m, &ew, &iw);
    }
    let mut noise_cases = BTreeSet::new();
    for key in 0..256 {
        noise_cases.insert((4, key, 255));
    }
    for row in [(0x104, 0xabcdef15, 0x12340000), (4, 80, 0), (4, 255, 0)] {
        noise_cases.insert(row);
    }
    for &(kind, key, fraction) in &noise_cases {
        reset(&mut m, 0x08149510);
        m.cpu.regs[..3].copy_from_slice(&[kind, key, fraction]);
        let ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        instructions += until(&mut m, RETURN, &mut pcs, &mut reads);
        assert_eq!(
            m.cpu.regs[0],
            data.cgb_frequency_word(kind, key, fraction).unwrap()
        );
        returned(&m, &ew, &iw);
    }
    let mut rate_cases = Vec::new();
    for selector in 0..16 {
        rate_cases.push(selector << 16);
    }
    rate_cases.extend([0xff070000, 0xffff0000]);
    for &mode in &rate_cases {
        reset(&mut m, 0x08148fcc);
        m.cpu.regs[0] = mode;
        let iw = m.bus.iwram.clone();
        let mut ew = m.bus.ewram.clone();
        ew[0x84e8] = ((mode >> 16) & 15) as u8;
        put(
            &mut ew,
            0x84f0,
            u32::from(data.sample_count_lookup(mode).unwrap()),
        );
        instructions += until(&mut m, 0x08148fec, &mut pcs, &mut reads);
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.cpu.regs[13], STACK - 16);
        outside_stack(&m, &iw, 16);
    }
    let mut volume_cases = Vec::new();
    volume_cases.extend(0u8..16);
    volume_cases.extend([16, 64, 255]);
    for &level in &volume_cases {
        reset(&mut m, 0x08149a32);
        m.cpu.regs[4] = CHANNEL;
        m.bus.ewram[0x10009] = level;
        put(&mut m.bus.iwram, 0x7e0c, 0x02020000);
        let iw = m.bus.iwram.clone();
        let mut ew = m.bus.ewram.clone();
        ew[0x20000] = rom[0x370cf4 + usize::from(level)];
        instructions += until(&mut m, 0x08149a3e, &mut pcs, &mut reads);
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
        if level < 16 {
            assert_eq!(ew[0x20000], data.wave_volume_code(level).unwrap());
        }
        assert_eq!(m.cpu.regs[13], STACK);
    }
    // The public SoundMode zero-nibble gate skips the direct helper's selector0 alias.
    reset(&mut m, 0x08149070);
    put(&mut m.bus.ewram, 0x84e0, 0x68736d53);
    let ew = m.bus.ewram.clone();
    let iw = m.bus.iwram.clone();
    let before = reads.clone();
    instructions += until(&mut m, RETURN, &mut pcs, &mut reads);
    assert_eq!(reads, before);
    returned(&m, &ew, &iw);
    let owned: Vec<_> = reads
        .iter()
        .copied()
        .filter(|at| (0x370b20..0x370d04).contains(at))
        .collect();
    let outside: Vec<_> = reads
        .iter()
        .copied()
        .filter(|at| !(0x370b20..0x370d04).contains(at))
        .collect();
    assert_eq!(owned.len(), 484);
    assert_eq!(outside, vec![0x370d04, 0x370d34, 0x370df3]);
    assert!(
        pcs.iter().any(|&(_, thumb)| !thumb),
        "unsigned multiply-high ARM helper executed"
    );
    assert_eq!(m.bus.rom, rom);
    fs::create_dir_all(&out).unwrap();
    fs::write(out.join("audio-driver-tables.json"), bytes).unwrap();
    fs::write(out.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"tables":7,"source_bytes":484,"lossless_all_fields_independent_hashes_json_source_roundtrip":true,"strict_shape_json_guards":15,"coherent_unsigned_signed_unused_bit_edits_rejected":true,"actual_sequence_boundary_owner_validated":true,"pcm_full_returns":pcm_cases.len(),"cgb_tonal_full_returns":tonal_cases.len(),"noise_full_returns":noise_cases.len(),"sample_count_prefixes":rate_cases.len(),"wave_volume_prefixes":volume_cases.len(),"public_zero_frequency_skip":true,"distinct_original_owned_bytes":owned.len(),"distinct_original_alias_bytes_outside_family":outside,"unsigned_mulhi_arm_helper":true,"ram_readonly_or_predicted_prefix_writes_sp_callee_outside_stack":true,"source_probes":"full PCM/CGB/noise/zero-mode returns, explicit sample-count/volume internal prefixes; not full init/CGB mixer/state-clock/native/ordinary acceptance","instructions_coverage":instructions,"pcs":pcs.len(),"managed":"e43-d42 unchanged","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}

#[test]
#[ignore = "private EUR balanced CGB gain producer counterexamples; Wine only, Linux/USA pending"]
fn european_cgb_balanced_gain_preserves_adjacent_volume_indices() {
    let out = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DRIVER_GAIN_ROOT").unwrap());
    assert!(!out.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let mut m = Machine::new(rom.clone());
    let mut pcs = BTreeSet::new();
    let mut reads = BTreeSet::new();
    let mut instructions = 0;
    let mut rows = Vec::new();
    for (right, left) in [
        (0u8, 0u8),
        (255, 0),
        (0, 255),
        (127, 127),
        (128, 128),
        (255, 127),
        (255, 128),
        (255, 255),
    ] {
        reset(&mut m, 0x08149608);
        m.cpu.regs[0] = CHANNEL;
        m.bus.ewram[0x10002] = right;
        m.bus.ewram[0x10003] = left;
        m.bus.ewram[0x10006] = 15;
        m.bus.ewram[0x1001c] = 0xff;
        let single = if right >= left {
            right / 2 >= left
        } else {
            left / 2 >= right
        };
        let raw = (u32::from(right) + u32::from(left)) >> 4;
        let level = if single { raw.min(15) } else { raw };
        let mask = if single {
            if right >= left {
                0x0f
            } else {
                0xf0
            }
        } else {
            0xff
        };
        let mut expected = m.bus.ewram.clone();
        expected[0x1000a] = level as u8;
        expected[0x10019] = ((15 * level + 15) >> 4) as u8;
        expected[0x1001b] = mask;
        let iw = m.bus.iwram.clone();
        instructions += until(&mut m, RETURN, &mut pcs, &mut reads);
        assert_eq!(m.bus.ewram, expected);
        assert_eq!(m.cpu.regs[13], STACK);
        outside_stack(&m, &iw, 8);
        for n in 4..12 {
            assert_eq!(m.cpu.regs[n], 0x11110000 + n as u32);
        }
        rows.push(serde_json::json!({"right":right,"left":left,"single_side":single,"level":level,"pan_mask":mask,"sustain":expected[0x10019]}));
    }
    assert!(reads.is_empty());
    assert_eq!(rows[4]["level"], 16);
    assert_eq!(rows[7]["level"], 31);
    assert_eq!(rows[1]["level"], 15);
    assert_eq!(rows[6]["level"], 23);
    fs::create_dir_all(&out).unwrap();
    fs::write(out.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"original_gain_full_returns":8,"rows":rows,"ram_sp_callee_outside8byte_stack":true,"balanced_route_has_no15clamp":true,"normal_global16entry_domain":"not proven; physical table16 does not bound live indexes","all1138prior_decoder_source_proof":"retained, not rerun","instructions_coverage":instructions,"pcs":pcs.len(),"new_table_bytes":0,"managed":"e43-d42 unchanged","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
