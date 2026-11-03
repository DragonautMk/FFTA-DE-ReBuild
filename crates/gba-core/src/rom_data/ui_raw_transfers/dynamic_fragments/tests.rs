use super::*;

#[test]
#[ignore = "requires private EUR ROM; Wine only, Linux and USA pending"]
fn european_dynamic_fragments_preserve_source_and_boundaries() {
    let rom = std::fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let model = from_rom(&rom).unwrap();
    model.verify_source(&rom).unwrap();
    let spans = model.source_spans().unwrap();
    assert_eq!(spans.len(), 7);
    assert_eq!(spans.iter().map(|s| s.1.len()).sum::<usize>(), 1304);
    for (offset, bytes, _) in &spans {
        assert_eq!(bytes.as_slice(), &rom[*offset as usize..*offset as usize + bytes.len()]);
        for (i, byte) in bytes.iter().enumerate() {
            assert_eq!(model.byte_at(*offset + i as u32), Some(*byte));
        }
        assert_eq!(model.byte_at(*offset - 1), None);
        assert_eq!(model.byte_at(*offset + bytes.len() as u32), None);
    }
    assert_eq!(model.rows[0].pairs, [[8,14],[12,10],[12,14]]);
    assert_eq!(model.rows[1].pairs, [[12,10],[8,14],[12,14]]);
    let encoded = serde_json::to_vec(&model).unwrap();
    assert_eq!(serde_json::from_slice::<Fragments>(&encoded).unwrap(), model);
    let mut changed = model.clone();
    changed.banks[0].words[0] ^= 1;
    changed.validate().unwrap();
    assert!(changed.verify_source(&rom).is_err());
    changed.rows[0].pairs[0][0] = 15;
    assert!(changed.validate().is_err());
    let mut changed = model.clone();
    changed.banks[0].relative_offset = 0;
    assert!(changed.validate().is_err());
    let mut bad_rom = rom;
    bad_rom[0] ^= 1;
    assert!(from_rom(&bad_rom).is_err());
}
