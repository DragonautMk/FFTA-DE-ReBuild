//! Typed conditional packed bytes; other owners and live guest indices remain unchanged.
use super::*;
use crate::rom_data::conditional_packed_lookup as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedLookupStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub byte0_bytes: u64,
    pub packed1_bytes: u64,
    pub packed2_bytes: u64,
    pub byte3_bytes: u64,
}
pub(super) struct PackedLookupResources { data: model::Lookup, counters: [AtomicU64; 6] }
impl PackedLookupResources {
    pub(super) fn source_page(index:usize)->bool { index >> 12 == 0x6e6 }
    pub(super) fn new(data:model::Lookup,sources:&[SourceRange])->Result<Self,String> {
        let bytes=data.source_bytes()?;
        if sources.len()!=1 || sources[0].offset!=model::SOURCE_OFFSET || sources[0].length as usize!=model::RECORD_COUNT*4
            || sources[0].role!="conditional-packed-lookup" || sources[0].sha256!=model::SOURCE_SHA256
            || ffta_rom::digest(&bytes)!=model::SOURCE_SHA256 {
            return Err("Conditional packed lookup source binding mismatch".into());
        }
        Ok(Self {data,counters:std::array::from_fn(|_|AtomicU64::new(0))})
    }
    pub(super) fn guest_read<const N:usize>(&self,index:usize,rom:&[u8],prior:Option<[u8;N]>)->Option<[u8;N]> {
        let start=model::SOURCE_OFFSET as usize;let end=start+model::RECORD_COUNT*4;
        if index>=end || index.saturating_add(N)<=start {return prior;}
        let mut counts=[1u64,0,0,0,0,0];
        let bytes=std::array::from_fn(|i| {
            let at=index+i;
            if let Some(byte)=self.data.byte_at(at as u32) {counts[1]+=1;counts[2+(at-start)%4]+=1;byte}
            else {prior.as_ref().map_or_else(||rom[at],|b|b[i])}
        });
        for (counter,count) in self.counters.iter().zip(counts) {counter.fetch_add(count,Ordering::Relaxed);}
        Some(bytes)
    }
    pub(super) fn stats(&self)->PackedLookupStats {
        let [bus_reads,source_bytes,byte0_bytes,packed1_bytes,packed2_bytes,byte3_bytes]=self.counters.each_ref().map(|v|v.load(Ordering::Relaxed));
        PackedLookupStats {bus_reads,source_bytes,byte0_bytes,packed1_bytes,packed2_bytes,byte3_bytes}
    }
    pub(super) fn resident_bytes(&self)->usize {std::mem::size_of::<Self>()+self.data.records.capacity()*std::mem::size_of::<model::Record>()}
}
