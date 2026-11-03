//! Conditional descriptor bytes composed with previous owners and raw ROM.
use super::*;
use crate::rom_data::conditional_descriptor_banks as model;

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct DescriptorBanksStats {
    pub bus_reads:u64,pub source_bytes:u64,pub pointer_bytes:u64,
    pub field0_bytes:u64,pub field2_bytes:u64,pub field4_bytes:u64,pub kind_bytes:u64,pub byte7_bytes:u64,
}
pub(super) struct DescriptorBanksResources {data:model::DescriptorBanks,counters:[AtomicU64;8]}
impl DescriptorBanksResources {
    pub(super) fn source_page(index:usize)->bool {matches!(index >> 12,0x6df|0x6e0)}
    pub(super) fn new(data:model::DescriptorBanks,sources:&[SourceRange])->Result<Self,String> {
        let bytes=data.source_bytes()?;
        if sources.len()!=model::SOURCE_SPANS.len() {return Err("Conditional descriptor source count mismatch".into());}
        for (i,(offset,length,digest,role)) in model::SOURCE_SPANS.into_iter().enumerate() {
            if sources[i].offset!=offset || sources[i].length!=length || sources[i].role!=role || sources[i].sha256!=digest || ffta_rom::digest(&bytes[i])!=digest {
                return Err("Conditional descriptor source binding mismatch".into());
            }
        }
        Ok(Self {data,counters:std::array::from_fn(|_|AtomicU64::new(0))})
    }
    pub(super) fn guest_read<const N:usize>(&self,index:usize,rom:&[u8],prior:Option<[u8;N]>)->Option<[u8;N]> {
        if !model::SOURCE_SPANS.iter().any(|(start,len,_,_)| index<(*start+*len) as usize && index.saturating_add(N)>*start as usize) {return prior;}
        let mut counts=[1u64,0,0,0,0,0,0,0];
        let bytes=std::array::from_fn(|i| {
            let at=index+i;
            if let Some(byte)=self.data.byte_at(at as u32) {
                counts[1]+=1;
                if (model::POINTER_OFFSET as usize..model::POINTER_OFFSET as usize+92).contains(&at) {counts[2]+=1;}
                else {let off=(at-model::BANK_OFFSETS[0] as usize)%8;counts[match off {0|1=>3,2|3=>4,4|5=>5,6=>6,_=>7}]+=1;}
                byte
            } else {prior.as_ref().map_or_else(||rom[at],|b|b[i])}
        });
        for (counter,count) in self.counters.iter().zip(counts) {counter.fetch_add(count,Ordering::Relaxed);}
        Some(bytes)
    }
    pub(super) fn stats(&self)->DescriptorBanksStats {
        let [bus_reads,source_bytes,pointer_bytes,field0_bytes,field2_bytes,field4_bytes,kind_bytes,byte7_bytes]=self.counters.each_ref().map(|v|v.load(Ordering::Relaxed));
        DescriptorBanksStats {bus_reads,source_bytes,pointer_bytes,field0_bytes,field2_bytes,field4_bytes,kind_bytes,byte7_bytes}
    }
    pub(super) fn resident_bytes(&self)->usize {std::mem::size_of::<Self>()+self.data.banks.capacity()*std::mem::size_of::<model::Bank>()+self.data.banks.iter().map(|b|b.records.capacity()*std::mem::size_of::<model::Record>()).sum::<usize>()}
}
