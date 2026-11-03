//! Conditional EUR packed source model and the original descriptor readers.
use super::*;
use gba_core::rom_data::conditional_packed_lookup::{Catalog, Lookup, RECORD_COUNT, SOURCE_OFFSET};
use gba_core::Machine;

#[test]
#[ignore = "private EUR source and original reader check; Wine only, Linux/USA pending"]
fn european_conditional_packed_lookup_original_readers() {
    const STOP: u32 = 0x03007f00;
    let root = PathBuf::from(std::env::var_os("FFTA_PACKED_LOOKUP_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let model = &catalog.decoded;
    model.verify_source(&rom).unwrap();
    assert_eq!((catalog.sources.len(),catalog.sources[0].offset,catalog.sources[0].length),(1,SOURCE_OFFSET,132));
    assert_eq!(model.source_bytes().unwrap(),rom[SOURCE_OFFSET as usize..SOURCE_OFFSET as usize+132]);
    for at in SOURCE_OFFSET..SOURCE_OFFSET+132 {assert_eq!(model.byte_at(at),Some(rom[at as usize]));}
    assert_eq!(model.byte_at(SOURCE_OFFSET-1),None);
    assert_eq!(model.byte_at(SOURCE_OFFSET+132),None);
    assert_eq!(model.records.len(),RECORD_COUNT);
    let bytes = serde_json::to_vec(model).unwrap();
    assert_eq!(serde_json::from_slice::<Lookup>(&bytes).unwrap(),*model);
    let mut bad = model.clone();bad.records.pop();assert!(bad.validate().is_err());
    bad=model.clone();bad.first_index=0;assert!(bad.validate().is_err());
    bad=model.clone();bad.records[0].low5=32;assert!(bad.validate().is_err());
    bad=model.clone();bad.records[0].high3=8;assert!(bad.validate().is_err());
    bad=model.clone();bad.records[0].low2=4;assert!(bad.validate().is_err());
    bad=model.clone();bad.records[0].high6=64;assert!(bad.validate().is_err());
    bad=model.clone();bad.records[0].byte0^=1;assert!(bad.validate().is_ok());assert!(bad.verify_source(&rom).is_err());
    assert!(serde_json::from_str::<Lookup>("{\"source_offset\":0,\"first_index\":1,\"records\":[],\"unexpected\":true}").is_err());
    for index in [0,1,33,34,251,u32::MAX] {
        assert_eq!(Lookup::encoded_view(index),0x086e6ee4u32.wrapping_add(index.wrapping_mul(4)));
        assert_eq!(model.select(index).is_some(),(1..=33).contains(&index));
    }
    fn run_to(m:&mut Machine,end:u32,max:usize) {
        let mut steps=0;
        while m.cpu.regs[15]!=end {
            assert!(steps<max,"pc {:08x}, wanted {:08x}",m.cpu.regs[15],end);
            m.bus.frame_ready=false;m.step();steps+=1;
        }
    }
    // These descriptor locations come from the authenticated original bank table.
    // bank0/row0 is zero, bank1/row11=1, bank5/row65=33,
    // bank0/row1=251 is type1 and its table address aliases metadata14.
    let cases = [(0u8,0u8,0u16,0u8),(1,11,1,5),(5,65,33,5),(0,1,251,1)];
    for (bank,row,index,kind) in cases {
        let table=0x6df4d4+usize::from(bank)*4;
        let bank_ptr=u32::from_le_bytes(rom[table..table+4].try_into().unwrap());
        let descriptor=bank_ptr+u32::from(row)*8;
        let at=(descriptor-0x08000000) as usize;
        assert_eq!(u16::from_le_bytes(rom[at+4..at+6].try_into().unwrap()),index);
        assert_eq!(rom[at+6],kind);
        let mut m=crate::make_machine(rom.clone(),None);
        m.bus.ffta_sprites.enabled=false;
        m.cpu.regs=std::array::from_fn(|i|0x11000000+i as u32);
        m.cpu.regs[0]=u32::from(bank);m.cpu.regs[1]=u32::from(row);m.cpu.regs[14]=STOP|1;m.cpu.regs[15]=0x080d0ed4;m.cpu.cpsr=0xbf;
        run_to(&mut m,STOP,16);
        assert_eq!(m.cpu.regs[0],descriptor);
        // Original zero guard and packed reader: both low bits retain their roles.
        m.cpu.regs[0]=descriptor;m.cpu.regs[15]=0x08132046;m.cpu.cpsr=0xbf;
        run_to(&mut m,0x08132068,20);
        let pos=(Lookup::encoded_view(u32::from(index))-0x08000000) as usize;
        let expected=if index==0 {0} else {u32::from((rom[pos+1]>>5)|((rom[pos+2]&3)<<3))};
        assert_eq!(m.cpu.regs[0],expected,"packed reader index {index}");
        if let Some(record)=model.select(u32::from(index)) {assert_eq!(u32::from(record.combined()),expected);}
        // Original type-5 check branches around the table for all other types.
        m.cpu.regs[0]=descriptor;m.cpu.regs[15]=0x08134176;m.cpu.cpsr=0xbf;
        if kind==5 {
            run_to(&mut m,0x0813418c,16);
            assert_eq!(m.cpu.regs[0],u32::from(rom[pos+1]&31));
            assert_eq!(m.cpu.regs[0],u32::from(model.select(u32::from(index)).unwrap().low5));
        } else {run_to(&mut m,0x081341cc,8);}
    }
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","records":33,"source_bytes":132,"original_descriptor_cases":[[0,0,0],[1,11,1],[5,65,33],[0,1,251]],"type_guard":true,"metadata14_alias_raw":true,"Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false})).unwrap()).unwrap();
}
