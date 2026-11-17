// Shared desktop backlog algorithm, adapted to an explicit Android-private directory.
//! Host-only PCM backlog for 1x audio while the guest advances faster.
//! Memory stays bounded; the temporary file is discarded on mode changes.
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct NormalAudio {
    path: PathBuf,
    writer: Option<BufWriter<File>>,
    reader: Option<BufReader<File>>,
    bytes: Vec<u8>,
    pairs: u64,
}

impl NormalAudio {
    #[cfg(test)]
    pub fn new() -> io::Result<Self> {
        Self::new_in(&std::env::temp_dir())
    }

    pub fn new_in(directory: &Path) -> io::Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos();
        let path = directory.join(format!(
            "ffta-audio-{}-{stamp}-{}.tmp",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let writer = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        // A separate handle is required: try_clone shares a file offset.
        let reader = match File::open(&path) {
            Ok(file) => file,
            Err(error) => {
                drop(writer);
                let _ = std::fs::remove_file(&path);
                return Err(error);
            }
        };
        Ok(Self {
            path,
            writer: Some(BufWriter::with_capacity(65536, writer)),
            reader: Some(BufReader::with_capacity(65536, reader)),
            bytes: Vec::new(),
            pairs: 0,
        })
    }

    pub fn push(&mut self, samples: &[i16]) -> io::Result<()> {
        if samples.len() % 2 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "incomplete stereo audio pair",
            ));
        }
        self.bytes.clear();
        self.bytes
            .extend(samples.iter().flat_map(|sample| sample.to_le_bytes()));
        let writer = self.writer.as_mut().unwrap();
        writer.write_all(&self.bytes)?;
        writer.flush()?;
        self.pairs += (samples.len() / 2) as u64;
        Ok(())
    }

    pub fn next_pair(&mut self) -> io::Result<Option<(i16, i16)>> {
        if self.pairs == 0 {
            return Ok(None);
        }
        let mut bytes = [0u8; 4];
        self.reader.as_mut().unwrap().read_exact(&mut bytes)?;
        self.pairs -= 1;
        Ok(Some((
            i16::from_le_bytes([bytes[0], bytes[1]]),
            i16::from_le_bytes([bytes[2], bytes[3]]),
        )))
    }
}

impl Drop for NormalAudio {
    fn drop(&mut self) {
        // Windows needs all handles closed before the file can be removed.
        drop(self.writer.take());
        drop(self.reader.take());
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn queued_audio_keeps_order_and_reads_new_data_after_empty() {
        let mut queue = NormalAudio::new().unwrap();
        let path = queue.path.clone();
        queue.push(&[1, -1, 32767, -32768]).unwrap();
        assert_eq!(queue.next_pair().unwrap(), Some((1, -1)));
        assert_eq!(queue.next_pair().unwrap(), Some((32767, -32768)));
        assert_eq!(queue.next_pair().unwrap(), None);
        queue.push(&[7, 9]).unwrap();
        assert_eq!(queue.next_pair().unwrap(), Some((7, 9)));
        drop(queue);
        assert!(!path.exists());
    }
    #[test]
    fn audio_backlog_has_bounded_memory_and_independent_file_offsets() {
        let mut queue = NormalAudio::new().unwrap();
        let samples = vec![42; 16384];
        for _ in 0..32 {
            queue.push(&samples).unwrap();
        }
        assert_eq!(queue.bytes.len(), samples.len() * 2);
        assert_eq!(queue.pairs, 32 * 8192);
        for _ in 0..32 * 8192 {
            assert_eq!(queue.next_pair().unwrap(), Some((42, 42)));
        }
        assert_eq!(queue.next_pair().unwrap(), None);
    }
}

// Independent x1 output, like the desktop audio callback. This mutex never
// owns the emulator: a slow fast-forward batch cannot block the audio reader.
use std::sync::{Arc,Mutex};
static EXTERNAL:std::sync::atomic::AtomicBool=std::sync::atomic::AtomicBool::new(false);
static OUTPUT:Mutex<Option<Arc<Mutex<NormalAudio>>>>=Mutex::new(None);
pub fn external_reader()->bool{EXTERNAL.load(Ordering::Relaxed)}
pub fn set_external_reader(enabled:bool){EXTERNAL.store(enabled,Ordering::Relaxed);}
pub fn publish(queue:Option<Arc<Mutex<NormalAudio>>>){*OUTPUT.lock().unwrap_or_else(|e|e.into_inner())=queue;}
pub fn pull(samples:&mut[i16])->io::Result<usize>{
    let queue=OUTPUT.lock().unwrap_or_else(|e|e.into_inner()).clone();
    let Some(queue)=queue else{return Ok(0);};
    let mut queue=queue.lock().unwrap_or_else(|e|e.into_inner());let mut count=0;
    for pair in samples.chunks_exact_mut(2){
        let Some((l,r))=queue.next_pair()? else{break;};pair[0]=l;pair[1]=r;count+=2;
    }
    Ok(count)
}
