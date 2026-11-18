//! ROM-bound cartridge save backups. No game content is stored in this module.
use std::{fs::{self,OpenOptions},io::{Read,Write},path::{Path,PathBuf},sync::atomic::{AtomicU64,Ordering}};

const MAX_SAVE:usize=128*1024;
static NEXT:AtomicU64=AtomicU64::new(0);

#[derive(Clone,Debug)]
struct Entry { id:String, path:PathBuf, digest:String }

pub struct Backups { source:PathBuf, directory:PathBuf }
impl Backups {
    pub fn new(source:&Path,rom_key:&str)->Self {
        Self { source:source.to_owned(),directory:source.parent().unwrap_or(Path::new("."))
            .join("SaveBackups").join(rom_key).join(source.file_name().unwrap_or_default()) }
    }
    fn entries(&self)->Result<Vec<Entry>,String> {
        let listing=match fs::read_dir(&self.directory) {
            Ok(v)=>v, Err(e) if e.kind()==std::io::ErrorKind::NotFound=>return Ok(Vec::new()),
            Err(e)=>return Err(format!("Cannot list save backups: {e}")),
        };
        let mut entries=Vec::new();
        for item in listing {
            let item=item.map_err(|e|e.to_string())?;
            if !item.file_type().map_err(|e|e.to_string())?.is_file(){continue;}
            let name=item.file_name();let Some(name)=name.to_str() else{continue};
            let Some(id)=name.strip_prefix("backup-").and_then(|v|v.strip_suffix(".sav")) else{continue};
            let Some((generation,digest))=id.split_once('-') else{continue};
            if generation.len()!=20 || !generation.bytes().all(|v|v.is_ascii_digit()) || generation.parse::<u64>().is_err()
                || digest.len()!=64 || !digest.bytes().all(|v|v.is_ascii_digit()||(b'a'..=b'f').contains(&v)){continue;}
            entries.push(Entry{id:id.to_owned(),path:item.path(),digest:digest.to_owned()});
        }
        entries.sort_by(|a,b|b.id.cmp(&a.id));
        Ok(entries)
    }
    fn read_bounded(path:&Path)->Result<Vec<u8>,String> {
        let mut result=Vec::new();
        fs::File::open(path).map_err(|e|e.to_string())?.take((MAX_SAVE+1) as u64)
            .read_to_end(&mut result).map_err(|e|e.to_string())?;
        if result.len()>MAX_SAVE {return Err("Cartridge save exceeds 128 KiB".into());}
        Ok(result)
    }
    fn verified(entry:&Entry,expected:usize)->Result<Vec<u8>,String> {
        let bytes=Self::read_bounded(&entry.path)?;
        if bytes.len()!=expected || gba_core::ffta_rom::digest(&bytes)!=entry.digest {
            return Err("Backup is damaged or has an incompatible size".into());
        }
        Ok(bytes)
    }
    /// Capture and return exactly the saved bytes loaded by the new session.
    pub fn capture_startup(&self,expected:usize)->Result<Option<Vec<u8>>,String> {
        let bytes=match fs::File::open(&self.source) {
            Ok(f)=>{let mut b=Vec::new();f.take((MAX_SAVE+1) as u64).read_to_end(&mut b).map_err(|e|e.to_string())?;b},
            Err(e) if e.kind()==std::io::ErrorKind::NotFound=>return Ok(None),
            Err(e)=>return Err(format!("Cannot read cartridge save for backup: {e}")),
        };
        if bytes.len()!=expected || bytes.len()>MAX_SAVE {return Err("Cartridge save has an incompatible size".into());}
        let entries=self.entries()?;
        let next=match entries.first() {Some(e)=>e.id[..20].parse::<u64>().map_err(|e|e.to_string())?.checked_add(1).ok_or("Backup sequence exhausted")?,None=>1};
        let id=format!("{next:020}-{}",gba_core::ffta_rom::digest(&bytes));
        fs::create_dir_all(&self.directory).map_err(|e|e.to_string())?;
        atomic_write(&self.directory.join(format!("backup-{id}.sav")),&bytes,false)?;
        for old in self.entries()?.into_iter().skip(2) {fs::remove_file(old.path).map_err(|e|format!("Cannot retire old backup: {e}"))?;}
        Ok(Some(bytes))
    }
    pub fn choices(&self,expected:usize)->Result<Vec<(String,bool)>,String> {
        Ok(self.entries()?.into_iter().take(2).map(|e| {
            let valid=Self::verified(&e,expected).is_ok(); (e.id,valid)
        }).collect())
    }
    pub fn read_choice(&self,id:&str,expected:usize)->Result<Vec<u8>,String> {
        let matches:Vec<_>=self.entries()?.into_iter().take(2).filter(|e|e.id==id).collect();
        if matches.len()!=1{return Err("The selected backup is no longer available".into());}
        Self::verified(&matches[0],expected)
    }
    pub fn replace_save(&self,bytes:&[u8])->Result<(),String>{atomic_write(&self.source,bytes,true)}
}

pub(crate) fn atomic_write(path:&Path,bytes:&[u8],replace:bool)->Result<(),String>{
    let parent=path.parent().ok_or("Save directory missing")?;
    let temporary=parent.join(format!(".ffta-backup-{}-{}.tmp",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
    let result=(||{
        let mut file=OpenOptions::new().write(true).create_new(true).open(&temporary).map_err(|e|e.to_string())?;
        file.write_all(bytes).map_err(|e|e.to_string())?;file.sync_all().map_err(|e|e.to_string())?;drop(file);
        if replace {fs::rename(&temporary,path).map_err(|e|e.to_string())?;}
        else {publish_noclobber(&temporary,path).map_err(|e|format!("Cannot publish save backup: {e}"))?;}
        Ok(())
    })();
    let _=fs::remove_file(&temporary);
    result
}

// Android may deny hard links even inside the app's own data directory.
// renameat2 publishes the completed file atomically without replacing a
// concurrently created backup; if unsupported, fail rather than clobber it.
#[cfg(any(target_os="android",target_os="linux"))]
fn publish_noclobber(source:&Path,destination:&Path)->std::io::Result<()> {
    use std::{ffi::CString,os::unix::ffi::OsStrExt};
    let source=CString::new(source.as_os_str().as_bytes()).map_err(|_|std::io::Error::new(std::io::ErrorKind::InvalidInput,"Invalid backup path"))?;
    let destination=CString::new(destination.as_os_str().as_bytes()).map_err(|_|std::io::Error::new(std::io::ErrorKind::InvalidInput,"Invalid backup path"))?;
    let result=unsafe {libc::syscall(libc::SYS_renameat2,libc::AT_FDCWD,source.as_ptr(),libc::AT_FDCWD,destination.as_ptr(),libc::RENAME_NOREPLACE)};
    if result==0 {Ok(())} else {Err(std::io::Error::last_os_error())}
}

#[cfg(not(any(target_os="android",target_os="linux")))]
fn publish_noclobber(source:&Path,destination:&Path)->std::io::Result<()> {
    fs::hard_link(source,destination)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn publication_is_atomic_and_never_replaces_an_existing_backup() {
        let root=std::env::var_os("FFTA_VALIDATION_WORK").map(PathBuf::from).unwrap_or_else(std::env::temp_dir)
            .join(format!("android-backup-publish-{}",std::process::id()));
        fs::create_dir(&root).unwrap();
        let target=root.join("backup.sav");
        atomic_write(&target,b"first",false).unwrap();
        assert!(atomic_write(&target,b"second",false).is_err());
        assert_eq!(fs::read(&target).unwrap(),b"first");
        assert_eq!(fs::read_dir(&root).unwrap().count(),1);
        fs::remove_dir_all(root).unwrap();
    }
}
