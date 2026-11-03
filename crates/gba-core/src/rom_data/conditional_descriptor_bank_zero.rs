//! Conditional EUR bank-zero pointer and records 1..15; record 0 stays raw.
use super::{conditional_descriptor_banks::Record,SourceRange};
use crate::ffta_rom;
use serde::{Deserialize,Serialize};

pub const POINTER_OFFSET:u32=0x6df4d4;
pub const BANK_ROOT:u32=0x6df534;
pub const RECORD_OFFSET:u32=BANK_ROOT+8;
pub const FIRST_ROW:u8=1;
pub const RECORD_COUNT:usize=15;
pub const SOURCE_SPANS:[(u32,u32,&str,&str);2]=[
    (POINTER_OFFSET,4,"a0d558900c5a1dfd5decb8a297f16ebce2a88c99bf766d64b2a287bd5b401993","conditional-descriptor-bank-zero-pointer"),
    (RECORD_OFFSET,120,"ea4c45009fdc0a9d00a4bf7ceaa61f2652176174d76e22495ade59eeab6f855f","conditional-descriptor-bank-zero-records"),
];
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BankZero {pub pointer_offset:u32,pub bank_root:u32,pub first_row:u8,pub records:Vec<Record>}
impl BankZero {
    pub fn validate(&self)->Result<(),String> {if self.pointer_offset!=POINTER_OFFSET || self.bank_root!=BANK_ROOT || self.first_row!=FIRST_ROW || self.records.len()!=RECORD_COUNT {return Err("Invalid conditional descriptor bank-zero projection".into());}Ok(())}
    pub fn select(&self,row:u8)->Option<&Record> {self.records.get(row.checked_sub(self.first_row)? as usize)}
    pub fn source_bytes(&self)->Result<[Vec<u8>;2],String> {self.validate()?;Ok([(0x08000000u32+self.bank_root).to_le_bytes().to_vec(),self.records.iter().flat_map(Record::bytes).collect()])}
    pub fn byte_at(&self,offset:u32)->Option<u8> {
        if let Some(n)=offset.checked_sub(self.pointer_offset).filter(|n|*n<4) {return Some((0x08000000u32+self.bank_root).to_le_bytes()[n as usize]);}
        let n=offset.checked_sub(RECORD_OFFSET)? as usize;Some(self.records.get(n/8)?.bytes()[n%8])
    }
    pub fn verify_source(&self,rom:&[u8])->Result<(),String> {self.validate()?;if self!=&Catalog::from_rom(rom)?.decoded {return Err("Conditional descriptor bank-zero differs from authenticated EUR source".into());}Ok(())}
}
pub struct Catalog {pub decoded:BankZero,pub sources:Vec<SourceRange>}
impl Catalog {
    pub fn from_rom(rom:&[u8])->Result<Self,String> {
        if rom.len()!=0x1000000 || ffta_rom::digest(rom)!=ffta_rom::EUROPE_SHA256 {return Err("Conditional descriptor bank-zero requires exact EUR ROM; USA pending".into());}
        for (offset,length,digest,_) in SOURCE_SPANS {if ffta_rom::digest(&rom[offset as usize..(offset+length) as usize])!=digest {return Err("Conditional descriptor bank-zero fingerprint mismatch".into());}}
        let records=rom[RECORD_OFFSET as usize..RECORD_OFFSET as usize+RECORD_COUNT*8].chunks_exact(8).map(|v|Record {field0:u16::from_le_bytes([v[0],v[1]]),field2:u16::from_le_bytes([v[2],v[3]]),field4:u16::from_le_bytes([v[4],v[5]]),kind:v[6],byte7:v[7]}).collect();
        let decoded=BankZero {pointer_offset:POINTER_OFFSET,bank_root:BANK_ROOT,first_row:FIRST_ROW,records};decoded.validate()?;
        for (i,bytes) in decoded.source_bytes()?.iter().enumerate() {let (offset,length,digest,_)=SOURCE_SPANS[i];if bytes.len()!=length as usize || ffta_rom::digest(bytes)!=digest || bytes!=&rom[offset as usize..(offset+length) as usize] {return Err("Conditional descriptor bank-zero reconstruction mismatch".into());}}
        let sources=SOURCE_SPANS.into_iter().map(|(offset,length,sha256,role)|SourceRange {offset,length,sha256:sha256.into(),role:role.into()}).collect();Ok(Self {decoded,sources})
    }
}
