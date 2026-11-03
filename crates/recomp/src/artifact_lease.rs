//! Host-only directory lease shared with scripts/artifact_coordinator.py.
//! Atomic mkdir works across Python/Wine/native hosts; stale leases fail closed.
use std::{fs, io::Write, path::{Path, PathBuf}};
use serde_json::{Value, json};
pub(crate) const PROTOCOL:&str="directory-v1";
pub(crate) const NAME:&str=".debug-artifacts-lease";
pub(crate) struct Lease {path:PathBuf,owner:Value}
impl Lease {
    pub fn acquire(root:&Path,role:&str)->Result<Self,String>{
        let meta=fs::symlink_metadata(root).map_err(|e|e.to_string())?;
        if !meta.is_dir() || meta.file_type().is_symlink(){return Err("Artifact root must be an ordinary directory".into());}
        #[cfg(windows)] {use std::os::windows::fs::MetadataExt;if meta.file_attributes()&0x400!=0{return Err("Artifact root must not be a reparse point".into());}}
        let path=root.join(NAME);
        fs::create_dir(&path).map_err(|e|format!("Active or interrupted artifact lease {}: {e}; inspect owner, never reclaim automatically",path.display()))?;
        let token=format!("{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e|e.to_string())?.as_nanos());
        let owner=json!({"schema":1,"protocol":PROTOCOL,"token":token,"pid":std::process::id(),"role":role});
        let mut f=fs::OpenOptions::new().write(true).create_new(true).open(path.join("owner.json")).map_err(|e|e.to_string())?;
        f.write_all(serde_json::to_string(&owner).unwrap().as_bytes()).and_then(|_|f.sync_all()).map_err(|e|e.to_string())?;
        Ok(Self{path,owner})
    }
}
impl Drop for Lease {
    fn drop(&mut self){
        let same=crate::diagnostics::read(&self.path.join("owner.json"),4096).ok().and_then(|b|serde_json::from_slice::<Value>(&b).ok()).is_some_and(|v|v==self.owner);
        if same {let _=fs::remove_file(self.path.join("owner.json"));let _=fs::remove_dir(&self.path);}
    }
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn directory_lease_contention_release_and_interrupted_owner(){
        let d=tempfile::tempdir().unwrap();let a=Lease::acquire(d.path(),"synthetic-runtime").unwrap();
        assert!(Lease::acquire(d.path(),"synthetic-cleaner").is_err());drop(a);
        let a=Lease::acquire(d.path(),"synthetic-cleaner").unwrap();
        fs::write(d.path().join(NAME).join("owner.json"),b"{\"interrupted_or_replaced\":true}").unwrap();drop(a);
        assert!(d.path().join(NAME).is_dir());assert!(Lease::acquire(d.path(),"another").is_err());
    }
}
