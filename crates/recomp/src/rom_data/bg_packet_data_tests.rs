//! Readable packet resources against retained EUR original-loader evidence.
use super::*;
use gba_core::rom_data::bg_packet_data::{Catalog, Program, Programs, Role, Writes};

#[test]
fn packet_json_roundtrip_rejects_unknown_and_inconsistent_fields() {
    let encoded = [1, 255, 255, 255, 0, 0, 2, 0, 0xff, 0x80, 0, 0];
    let (program, _) = Program::decode(&encoded, Role::Halfwords, 0x1000).unwrap();
    let bytes = serde_json::to_vec_pretty(&program).unwrap();
    let parsed: Program = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed, program);
    parsed.validate(Role::Halfwords).unwrap();
    for nested in [false, true] {
        let mut value = serde_json::to_value(&program).unwrap();
        let object = if nested {
            &mut value["writes"]
        } else {
            &mut value
        };
        object
            .as_object_mut()
            .unwrap()
            .insert("unknown".into(), 1.into());
        assert!(serde_json::from_value::<Program>(value).is_err());
    }
    let mut value = serde_json::to_value(&program).unwrap();
    value["writes"]["packets"][0]["values"][0] = 0.into();
    let edited: Program = serde_json::from_value(value).unwrap();
    assert!(edited.validate(Role::Halfwords).is_err());
    let mut value = serde_json::to_value(&program).unwrap();
    value["writes"]["kind"] = "unknown".into();
    assert!(serde_json::from_value::<Program>(value).is_err());
}

#[test]
#[ignore = "requires exact EUR ROM, retained packet-catalog captures and absent FFTA_STATE_TEST_PACKET_DATA_ROOT"]
fn european_packet_data_matches_retained_original_catalog() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_PACKET_DATA_ROOT").unwrap());
    assert!(!root.exists(), "preserve typed resource evidence");
    let captures = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_PACKET_CAPTURES").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256,
        "new verification is deliberately EUR-only"
    );
    let expected: serde_json::Value =
        serde_json::from_slice(&fs::read(captures.join("result.json")).unwrap()).unwrap();
    assert_eq!(expected["wrapper_invocations"], 486);
    let catalog = Catalog::from_rom(&rom).unwrap();
    let descriptor_root = gba_core::rom_data::bg_descriptors::Catalog::from_rom(&rom)
        .unwrap()
        .source
        .offset;
    assert_eq!(catalog.sources.len(), 352);
    let mut provenance = catalog.sources.iter();
    let mut encoded_bytes = 0;
    let mut body_bytes = 0;
    let mut write_count = 0;
    for (role, count) in [
        (Role::Paired, 122),
        (Role::Halfwords, 117),
        (Role::Auxiliary, 113),
    ] {
        let family = catalog.decoded.family(role);
        assert_eq!(family.descriptor_programs.len(), 162);
        assert_eq!(family.programs.len(), count);
        let original = expected["families"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["role"] == role.name())
            .unwrap();
        for (id, program) in family.programs.iter().enumerate() {
            let source = provenance.next().unwrap();
            let old = original["sources"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["source"] == source.offset)
                .unwrap();
            assert_eq!(source.offset, descriptor_root + program.relative_source);
            assert_eq!(source.length as usize, program.encoded_source.len());
            assert_eq!(
                source.role,
                format!("bg-packet-program/{}/{id}", role.name())
            );
            assert_eq!(source.sha256, old["encoded_sha256"].as_str().unwrap());
            assert_eq!(
                gba_core::ffta_rom::digest(&program.encoded_source),
                source.sha256
            );
            assert_eq!(
                program.encoded_source,
                rom[source.offset as usize..(source.offset + source.length) as usize]
            );
            assert_eq!(
                program.packet_body,
                fs::read(captures.join(old["body_file"].as_str().unwrap())).unwrap()
            );
            assert_eq!(
                gba_core::ffta_rom::digest(&program.packet_body),
                old["body_sha256"].as_str().unwrap()
            );
            assert_eq!(
                serde_json::to_value(program.parent_descriptor).unwrap(),
                old["parent"]
            );
            encoded_bytes += program.encoded_source.len();
            body_bytes += program.packet_body.len();
            write_count += match &program.writes {
                Writes::Paired(v) => v
                    .payload
                    .packets
                    .iter()
                    .map(|p| p.values.len())
                    .sum::<usize>(),
                Writes::Halfwords(v) => v.packets.iter().map(|p| p.values.len()).sum::<usize>(),
            };
        }
        for id in 0..162 {
            let program = family.program(id).unwrap();
            assert_eq!(
                descriptor_root + program.relative_source,
                original["records"][id].as_u64().unwrap() as u32
            );
            assert!(family.ordered_programs(id).unwrap().len() <= 2);
        }
    }
    assert!(provenance.next().is_none());
    assert_eq!((encoded_bytes, body_bytes), (398227, 523794));
    let bytes = serde_json::to_vec_pretty(&catalog.decoded).unwrap();
    assert_eq!(bytes, serde_json::to_vec_pretty(&catalog.decoded).unwrap());
    let parsed: Programs = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed, catalog.decoded);
    parsed.verify_source(&rom).unwrap();
    // A coherent content edit passes structural validation but not base-ROM
    // authentication, even if its file hash is recomputed by a caller.
    let mut forged = parsed.clone();
    let index = forged
        .halfwords
        .programs
        .iter()
        .position(|p| p.encoded_source[0] == 1)
        .unwrap();
    let original = &forged.halfwords.programs[index];
    let mut encoded = original.encoded_source.clone();
    encoded[8] ^= 0x80; // First literal halfword, after wrapper/packet header.
    forged.halfwords.programs[index] =
        Program::decode(&encoded, Role::Halfwords, original.relative_source)
            .unwrap()
            .0;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let mut bad_rom = rom.clone();
    bad_rom[catalog.sources[0].offset as usize + 1] ^= 1;
    assert!(Catalog::from_rom(&bad_rom).is_err());
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("bg-packet-programs.json"), &bytes).unwrap();
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "region":"europe","programs":352,"descriptor_links":486,"encoded_bytes":encoded_bytes,
            "body_bytes":body_bytes,"decoded_write_values":write_count,
            "json_bytes":bytes.len(),"json_sha256":gba_core::ffta_rom::digest(&bytes),
            "retained_original_bodies_equal":true,"source_authentication":true,
            "coherent_edit_rejected":true,"no_original_oracle_replay":true,
            "usa_validation":"pending by user instruction",
        }))
        .unwrap(),
    )
    .unwrap();
    eprintln!("BG_PACKET_DATA region=europe programs=352 descriptor_links=486 encoded_bytes={encoded_bytes} body_bytes={body_bytes} write_values={write_count} retained_bodies_equal=true json_roundtrip=true coherent_edit_rejected=true usa=pending");
}
