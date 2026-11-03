//! One decoder check against retained independent source evidence; old tone/endpoint probes stay closed.
use super::*;
use gba_core::{
    rom_data::{audio_directory, audio_tones::*},
    Machine,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
fn n(v: &Value, key: &str) -> u32 {
    v[key].as_u64().unwrap().try_into().unwrap()
}
fn read(name: &str) -> Value {
    let b = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_TONE_EVIDENCE").unwrap());
    serde_json::from_slice(&fs::read(b.join(name)).unwrap()).unwrap()
}
#[test]
#[ignore = "private EUR lossless M4A tone/PCM decoder and new2CGB probes; Wine only, Linux/USA pending"]
fn european_audio_tone_model_matches_retained_independent_sources_and_cgb_copy() {
    let out = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_TONE_MODEL_ROOT").unwrap());
    assert!(!out.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let independent = read("tone-wave-independent.json");
    let sequence = read("sequence-producers.json");
    let plan = read("producer-source-plan.json");
    let cat = Catalog::from_rom(&rom).unwrap();
    let data = &cat.decoded;
    let directory = audio_directory::Catalog::from_rom(&rom).unwrap().decoded;
    data.validate_directory(&directory).unwrap();
    data.verify_source(&rom).unwrap();
    assert_eq!(
        (
            cat.producer.tracks,
            cat.producer.states,
            cat.producer.default_ram_notes,
            cat.producer.max_pattern_depth,
            cat.producer.consumed_bytes,
            cat.producer.consumed_ranges,
            cat.producer.programs,
            cat.producer.note_pairs
        ),
        (1610, 147424, 18, 1, 177047, 805, 390, 2065)
    );
    let consumed: Vec<(u32, u32)> = sequence["merged_consumed_ranges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| (a[0].as_u64().unwrap() as u32, a[1].as_u64().unwrap() as u32))
        .collect();
    assert_eq!(cat.consumed_bytecode, consumed);
    let mut bank_programs = 0;
    for b in &data.banks {
        let expected = &sequence["summary"]["bank_selectors"][format!("{:#x}", b.source_address)];
        assert_eq!(
            b.programs.iter().map(|p| p.selector).collect::<Vec<_>>(),
            expected
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as u8)
                .collect::<Vec<_>>()
        );
        for p in &b.programs {
            let keys = sequence["note_keys"].as_array().unwrap().iter().find(|v| {
                n(v, "bank") == b.source_address && n(v, "program") == u32::from(p.selector)
            });
            let expected: Vec<u8> = keys.map_or_else(Vec::new, |v| {
                v["keys"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|k| k.as_u64().unwrap() as u8)
                    .collect()
            });
            assert_eq!(p.note_keys, expected);
            bank_programs += 1;
        }
    }
    assert_eq!(bank_programs, 390);
    let tones: BTreeMap<_, _> = independent["tones"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| (n(t, "source_offset"), t))
        .collect();
    for t in &data.tones {
        let e = tones[&t.source_offset];
        assert_eq!(
            (t.ctype, t.key, t.length, t.pan_or_sweep, t.word04, t.word08),
            (
                n(e, "type") as u8,
                n(e, "key") as u8,
                n(e, "length") as u8,
                n(e, "pan_or_sweep") as u8,
                n(e, "word04"),
                n(e, "word08")
            )
        );
        assert_eq!(t.byte(12), None);
        assert_eq!(t.byte(usize::MAX), None);
    }
    let mut expected_maps = BTreeSet::new();
    for parent in independent["keymap_domains"].as_array().unwrap() {
        for k in parent["key_targets"].as_array().unwrap() {
            expected_maps.insert((
                n(parent, "bank") - 0x08000000 + 12 * n(parent, "program"),
                n(k, "key") as u8,
                n(k, "map_offset"),
                n(k, "selector") as u8,
                n(k, "tone_offset"),
            ));
        }
    }
    let actual_maps: BTreeSet<_> = data
        .map_links
        .iter()
        .map(|l| {
            let p = data.tone(l.parent_tone).unwrap();
            let b = &data.map_bytes[l.byte as usize];
            (
                p.source_offset,
                l.key,
                b.source_offset,
                b.selector,
                data.tone(l.tone).unwrap().source_offset,
            )
        })
        .collect();
    assert_eq!(actual_maps, expected_maps);
    assert_eq!((actual_maps.len(), data.map_bytes.len()), (348, 347));
    let waves: BTreeMap<_, _> = independent["pcm_waves"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| (n(w, "source_offset"), w))
        .collect();
    let mut samples = 0;
    for w in &data.pcm_waves {
        let e = waves[&w.source_offset];
        assert_eq!(
            (
                w.header_type,
                w.flags,
                w.frequency_word,
                w.loop_start,
                w.declared_length
            ),
            (
                n(e, "type") as u16,
                n(e, "flags") as u16,
                n(e, "frequency_word"),
                n(e, "loop_start"),
                n(e, "size")
            )
        );
        let bytes = w.source_bytes();
        assert_eq!(
            gba_core::ffta_rom::digest(&bytes[..16]),
            e["header_sha256"].as_str().unwrap()
        );
        assert_eq!(
            gba_core::ffta_rom::digest(&bytes[16..16 + w.samples.len()]),
            e["payload_sha256"].as_str().unwrap()
        );
        let fixed = e["ctype_values"]
            .as_array()
            .unwrap()
            .iter()
            .all(|v| v.as_u64() == Some(8));
        assert_eq!(
            w.interpolation_endpoint,
            if fixed {
                None
            } else {
                Some(n(e, "guard_byte") as u8 as i8)
            }
        );
        assert_eq!(w.byte(bytes.len()), None);
        assert_eq!(w.byte(usize::MAX), None);
        samples += w.samples.len();
    }
    assert_eq!(samples, 1931528);
    let expected: BTreeMap<_, _> = plan["ranges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            (
                (n(s, "offset"), n(s, "length")),
                s["sha256"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        (
            cat.sources.len(),
            cat.sources.iter().map(|s| s.length as usize).sum::<usize>()
        ),
        (736, NEW_SOURCE_BYTES)
    );
    for s in &cat.sources {
        assert_eq!(s.sha256, expected[&(s.offset, s.length)]);
        assert!(s.role.starts_with("audio/"));
    }
    for (at, b, _) in data.source_spans() {
        assert_eq!(&rom[at as usize..at as usize + b.len()], b);
    }
    assert_eq!(
        data.pcm_waves[data.directory_aliases[0].wave as usize].source_offset,
        FIRST_WAVE
    );
    let bytes = serde_json::to_vec_pretty(data).unwrap();
    let parsed: AudioTones = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(&parsed, data);
    let unknown = serde_json::json!({"banks":[],"tones":[],"map_bytes":[],"map_links":[],"pcm_waves":[],"cgb_waves":[],"directory_aliases":[],"unknown":true});
    assert!(serde_json::from_value::<AudioTones>(unknown).is_err());
    let mut nested = serde_json::to_value(&data.tones[0]).unwrap();
    nested["unknown"] = true.into();
    assert!(serde_json::from_value::<Tone>(nested).is_err());
    let mut nested = serde_json::to_value(&data.pcm_waves[0]).unwrap();
    nested["samples"][0] = 128.into();
    assert!(serde_json::from_value::<PcmWave>(nested).is_err());
    for case in 0..25 {
        let mut bad = data.clone();
        match case {
            0 => bad.banks[0].programs[0].selector ^= 1,
            1 => {
                bad.banks[0].programs[0].note_keys = vec![128];
            }
            2 => bad.tones[0].id = 1,
            3 => bad.tones[0].source_offset += 4,
            4 => bad.tones[0].ctype = 0x20,
            5 => bad.map_bytes[0].id = 1,
            6 => bad.map_bytes[0].source_offset += 1,
            7 => bad.map_links[0].byte = (bad.map_links[0].byte + 1) % 347,
            8 => bad.map_links[0].tone = 0,
            9 => bad.map_links[0].key = 128,
            10 => bad.pcm_waves[0].id = 1,
            11 => bad.pcm_waves[0].declared_length += 1,
            12 => bad.pcm_waves[0].loop_start = bad.pcm_waves[0].declared_length + 1,
            13 => {
                bad.pcm_waves[0].samples.pop();
            }
            14 => bad.pcm_waves[0].interpolation_endpoint = None,
            15 => {
                bad.pcm_waves
                    .iter_mut()
                    .find(|w| w.interpolation_endpoint.is_none())
                    .unwrap()
                    .interpolation_endpoint = Some(0)
            }
            16 => bad.pcm_waves[0].tone_ids.clear(),
            17 => bad.pcm_waves[0].source_offset += 4,
            18 => bad.pcm_waves[0].header_type = 1,
            19 => bad.cgb_waves[0].amplitudes[0] = 16,
            20 => bad.cgb_waves[0].id = 1,
            21 => bad.directory_aliases[0].wave = (bad.directory_aliases[0].wave + 1) % 277,
            22 => {
                bad.pcm_waves.pop();
            }
            23 => {
                bad.map_links.pop();
            }
            _ => {
                bad.tones.pop();
            }
        }
        assert!(bad.validate().is_err(), "shape/link guard {case}");
    }
    let mut edited = data.clone();
    edited.pcm_waves[0].samples[0] ^= 1;
    edited.validate_directory(&directory).unwrap();
    assert!(edited.verify_source(&rom).is_err());
    let mut edited = data.clone();
    let alias = edited.directory_aliases[0].wave as usize;
    edited.pcm_waves[alias].flags ^= 0x4000;
    edited.validate().unwrap();
    assert_eq!(
        edited.source_spans(),
        data.source_spans(),
        "shared bytes have no duplicate new owner"
    );
    assert!(edited.validate_directory(&directory).is_err());
    let mut edited = data.clone();
    edited.banks[0].id += 1;
    assert!(edited.validate_directory(&directory).is_err());
    assert!(Catalog::from_rom(&[]).is_err());
    // New source category only: exact original four-word CGB copy, stopped before unrelated controls.
    let mut m = Machine::new(rom.clone());
    let mut instructions = 0;
    for w in &data.cgb_waves {
        assert_eq!(w.byte(16), None);
        assert_eq!(w.byte(usize::MAX), None);
        m.bus.ewram.fill(0);
        m.bus.io[0x90..0xa0].fill(0);
        m.cpu.regs = [0; 16];
        m.cpu.regs[4] = 0x02010000;
        m.cpu.regs[13] = 0x03007e00;
        m.cpu.regs[15] = 0x081497b0;
        m.cpu.cpsr = 0xbf;
        m.bus.reg_ie = 0;
        m.bus.ime = false;
        m.bus.ewram[0x10024..0x10028]
            .copy_from_slice(&(0x08000000 + w.source_offset).to_le_bytes());
        let mut expected = m.bus.ewram.clone();
        expected[0x10028..0x1002c].copy_from_slice(&(0x08000000 + w.source_offset).to_le_bytes());
        let mut count = 0;
        while m.cpu.regs[15] != 0x081497cc {
            assert!(count < 32);
            m.bus.frame_ready = false;
            m.step();
            count += 1;
        }
        instructions += count;
        assert_eq!(&m.bus.io[0x90..0xa0], w.source_bytes());
        assert_eq!(m.bus.ewram, expected);
        assert_eq!(m.cpu.regs[13], 0x03007e00);
    }
    assert_eq!(m.bus.rom, rom);
    fs::create_dir_all(&out).unwrap();
    fs::write(out.join("audio-tones.json"), &bytes).unwrap();
    fs::write(out.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({
        "producer":cat.producer,"source_ranges":736,"new_source_bytes":NEW_SOURCE_BYTES,"canonical_tones":429,"programs":390,"map_links":348,"map_bytes":347,
        "pcm_waves":277,"signed_payload_bytes":samples,"interpolated_endpoints":276,"cgb_waves":2,"cgb_new_original_copy_instructions":instructions,
        "retained_independent_program_note_domains_all_tone_fields_header_payload_hashes_spans":true,"lossless_json_source_roundtrip":true,
        "strict_shape_link_sample_type_guards":28,"coherent_source_edit_rejected":true,"alias_only_edit_same_new_spans_real_directory_dependency_rejected":true,
        "old390BD348maps277endpoints":"retained proof; not rerun","managed":"e41-d40 unchanged","linux":"pending","usa":"pending","performance_measurements":false
    })).unwrap()).unwrap();
}
