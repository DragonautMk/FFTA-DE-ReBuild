//! Per-run request identity ledger. It never executes or publishes commands.
use std::collections::{BTreeMap,VecDeque};

const MAX_IDS:usize=4096;
const MAX_RESULTS:usize=128;
const MAX_RESULT_BYTES:usize=2*1024*1024;
const MAX_SINGLE_RESULT:usize=128*1024;

enum State<T>{Pending,Complete(Option<(T,usize)>)}
struct Entry<T>{fingerprint:String,state:State<T>}
pub(super) enum Admission<T>{New,Pending,Cached(T),Unavailable,Conflict,Full}
pub(super) struct Ledger<T>{
    entries:BTreeMap<String,Entry<T>>,order:VecDeque<String>,bytes:usize,
    max_ids:usize,max_results:usize,max_bytes:usize,max_single:usize,
}
impl<T:Clone> Default for Ledger<T>{
    fn default()->Self{Self{entries:BTreeMap::new(),order:VecDeque::new(),bytes:0,max_ids:MAX_IDS,max_results:MAX_RESULTS,max_bytes:MAX_RESULT_BYTES,max_single:MAX_SINGLE_RESULT}}
}
impl<T:Clone> Ledger<T>{
    pub fn stats(&self)->(usize,usize,usize,usize,usize){(self.entries.len(),self.max_ids,self.order.len(),self.bytes,self.max_bytes)}
    pub fn admit(&mut self,id:&str,fingerprint:&str,reserve_for_active_cancel:bool)->Admission<T>{
        if let Some(entry)=self.entries.get(id){
            if entry.fingerprint!=fingerprint{return Admission::Conflict;}
            return match &entry.state{State::Pending=>Admission::Pending,State::Complete(Some((value,_)))=>Admission::Cached(value.clone()),State::Complete(None)=>Admission::Unavailable};
        }
        if self.entries.len()>=self.max_ids+usize::from(reserve_for_active_cancel){return Admission::Full;}
        self.entries.insert(id.to_owned(),Entry{fingerprint:fingerprint.to_owned(),state:State::Pending});Admission::New
    }
    /// Completes an admitted request at most once. Eviction discards only the
    /// response, never its identity; an evicted request cannot be executed again.
    pub fn finish(&mut self,id:&str,fingerprint:&str,value:T,size:usize)->bool{
        if !self.entries.get(id).is_some_and(|e|e.fingerprint==fingerprint&&matches!(e.state,State::Pending)){return false;}
        let retain=size<=self.max_single && size<=self.max_bytes && self.max_results>0;
        if retain {
            while self.order.len()>=self.max_results || self.bytes.saturating_add(size)>self.max_bytes {
                let Some(old)=self.order.pop_front() else{break};
                if let Some(Entry{state:State::Complete(slot),..})=self.entries.get_mut(&old){if let Some((_,n))=slot.take(){self.bytes-=n;}}
            }
            self.bytes+=size;self.order.push_back(id.to_owned());
        }
        self.entries.get_mut(id).unwrap().state=State::Complete(retain.then_some((value,size)));true
    }
    #[cfg(test)]
    pub(super) fn limits(max_ids:usize,max_results:usize,max_bytes:usize,max_single:usize)->Self{
        Self{entries:BTreeMap::new(),order:VecDeque::new(),bytes:0,max_ids,max_results,max_bytes,max_single}
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn same_identity_is_never_reissued_and_changed_payload_conflicts(){
        let mut l=Ledger::<String>::limits(3,2,20,10);
        assert!(matches!(l.admit("one","hash-a",false),Admission::New));assert!(matches!(l.admit("one","hash-a",false),Admission::Pending));
        assert!(matches!(l.admit("one","hash-b",false),Admission::Conflict));assert!(l.finish("one","hash-a","result".into(),6));
        assert!(!l.finish("one","hash-a","second".into(),6));assert!(matches!(l.admit("one","hash-a",false),Admission::Cached(v) if v=="result"));
    }
    #[test]
    fn result_eviction_and_capacity_never_reopen_old_ids(){
        let mut l=Ledger::<String>::limits(3,1,8,8);
        for id in ["one","two"]{assert!(matches!(l.admit(id,id,false),Admission::New));assert!(l.finish(id,id,id.into(),4));}
        assert!(matches!(l.admit("one","one",false),Admission::Unavailable));assert!(matches!(l.admit("two","two",false),Admission::Cached(_)));
        assert!(matches!(l.admit("three","three",false),Admission::New));assert!(l.finish("three","three","oversize".into(),9));
        assert!(matches!(l.admit("three","three",false),Admission::Unavailable));assert!(matches!(l.admit("four","four",false),Admission::Full));
        assert!(matches!(l.admit("one","one",false),Admission::Unavailable));assert!(matches!(l.admit("two","changed",false),Admission::Conflict));
        assert!(matches!(l.admit("cancel","target",true),Admission::New));assert!(matches!(l.admit("another","target",true),Admission::Full));
    }
}
