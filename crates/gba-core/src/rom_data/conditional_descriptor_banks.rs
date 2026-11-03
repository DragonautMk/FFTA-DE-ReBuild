//! EUR descriptor banks supported by the original bank-count producer.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const TABLE_ROOT:u32=0x6df4d4;
pub const POINTER_OFFSET:u32=TABLE_ROOT+4;
pub const BANK_OFFSETS:[u32;23]=[0x6df5b4,0x6dfa24,0x6dfc8c,0x6dff84,0x6e022c,0x6e04ec,0x6e052c,0x6e05dc,0x6e0624,0x6e068c,0x6e06e4,0x6e0734,0x6e0784,0x6e07cc,0x6e0814,0x6e085c,0x6e08a4,0x6e08f4,0x6e093c,0x6e0994,0x6e09ec,0x6e0a74,0x6e0afc];
pub const BANK_COUNTS:[usize;23]=[142,77,95,85,88,8,22,9,13,11,10,10,9,9,9,9,10,9,11,11,17,11,9];
pub const SOURCE_SPANS:[(u32,u32,&str,&str);3]=[
    (0x6df4d8,92,"d45c878472e941028e4841e0e89057f2be58661346093b5cb25221a539427dbf","conditional-descriptor-bank-pointers"),
    (0x6df5b4,5400,"43f885e3363987bbb855ab19b943d7c93a174a0c16ddd38362e37d8be97b39e6","conditional-descriptor-banks-1-22"),
    (0x6e0afc,72,"9702d38f1fd34c6d0b358812f43789f6b2efbd1c219c34e7a4f35a963c2ad6e0","conditional-descriptor-bank-23"),
];
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {pub field0:u16,pub field2:u16,pub field4:u16,pub kind:u8,pub byte7:u8}
impl Record {
    pub fn bytes(&self)->[u8;8] {let a=self.field0.to_le_bytes();let b=self.field2.to_le_bytes();let c=self.field4.to_le_bytes();[a[0],a[1],b[0],b[1],c[0],c[1],self.kind,self.byte7]}
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bank {pub index:u8,pub source_offset:u32,pub records:Vec<Record>}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DescriptorBanks {pub pointer_offset:u32,pub banks:Vec<Bank>}
impl DescriptorBanks {
    pub fn validate(&self)->Result<(),String> {
        if self.pointer_offset!=POINTER_OFFSET || self.banks.len()!=23 {return Err("Invalid conditional descriptor bank shape".into());}
        for (i,bank) in self.banks.iter().enumerate() {
            if bank.index as usize!=i+1 || bank.source_offset!=BANK_OFFSETS[i] || bank.records.len()!=BANK_COUNTS[i] {return Err("Invalid conditional descriptor bank binding".into());}
        }
        Ok(())
    }
    pub fn select(&self,bank:u8,row:u8)->Option<&Record> {self.banks.get(bank.checked_sub(1)? as usize)?.records.get(row as usize)}
    pub fn encoded_pointer_view(bank:u32)->u32 {(0x08000000u32+TABLE_ROOT).wrapping_add((bank as u8 as u32)*4)}
    pub fn encoded_record_view(pointer:u32,row:u32)->u32 {pointer.wrapping_add((row as u8 as u32)*8)}
    pub fn source_bytes(&self)->Result<[Vec<u8>;3],String> {
        self.validate()?;
        let pointers=self.banks.iter().flat_map(|bank|(0x08000000u32+bank.source_offset).to_le_bytes()).collect();
        let first=self.banks[..22].iter().flat_map(|bank|bank.records.iter().flat_map(Record::bytes)).collect();
        let last=self.banks[22].records.iter().flat_map(Record::bytes).collect();
        Ok([pointers,first,last])
    }
    pub fn byte_at(&self,offset:u32)->Option<u8> {
        let p=offset.checked_sub(self.pointer_offset);
        if let Some(p)=p.filter(|p|*p<92) {let bank=self.banks.get((p/4) as usize)?;return Some((0x08000000u32+bank.source_offset).to_le_bytes()[(p%4) as usize]);}
        for bank in &self.banks {if let Some(n)=offset.checked_sub(bank.source_offset) {if n<(bank.records.len()*8) as u32 {return Some(bank.records[(n/8) as usize].bytes()[(n%8) as usize]);}}}
        None
    }
    pub fn verify_source(&self,rom:&[u8])->Result<(),String> {self.validate()?;if self!=&Catalog::from_rom(rom)?.decoded {return Err("Conditional descriptor banks differ from authenticated EUR source".into());}Ok(())}
}
pub struct Catalog {pub decoded:DescriptorBanks,pub sources:Vec<SourceRange>}
impl Catalog {
    pub fn from_rom(rom:&[u8])->Result<Self,String> {
        if rom.len()!=0x1000000 || ffta_rom::digest(rom)!=ffta_rom::EUROPE_SHA256 {return Err("Conditional descriptor banks require exact EUR ROM; USA pending".into());}
        for (offset,length,digest,_) in SOURCE_SPANS {if ffta_rom::digest(&rom[offset as usize..(offset+length) as usize])!=digest {return Err("Conditional descriptor source fingerprint mismatch".into());}}
        let banks=BANK_OFFSETS.into_iter().zip(BANK_COUNTS).enumerate().map(|(i,(offset,count))|{
            let records=rom[offset as usize..offset as usize+count*8].chunks_exact(8).map(|v|Record {field0:u16::from_le_bytes([v[0],v[1]]),field2:u16::from_le_bytes([v[2],v[3]]),field4:u16::from_le_bytes([v[4],v[5]]),kind:v[6],byte7:v[7]}).collect();
            Bank {index:(i+1) as u8,source_offset:offset,records}
        }).collect();
        let decoded=DescriptorBanks {pointer_offset:POINTER_OFFSET,banks};decoded.validate()?;
        for (i,bytes) in decoded.source_bytes()?.iter().enumerate() {let (offset,length,digest,_)=SOURCE_SPANS[i];if bytes.len()!=length as usize || ffta_rom::digest(bytes)!=digest || bytes!=&rom[offset as usize..(offset+length) as usize] {return Err("Conditional descriptor reconstruction mismatch".into());}}
        let sources=SOURCE_SPANS.into_iter().map(|(offset,length,sha256,role)|SourceRange {offset,length,sha256:sha256.into(),role:role.into()}).collect();
        Ok(Self {decoded,sources})
    }
}
