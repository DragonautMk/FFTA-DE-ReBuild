use crate::compiler::{self, NativeCode};
use gba_core::{Machine, cpu::FLAG_I, mem};
use sha1::{Digest, Sha1};
use std::{collections::BTreeSet, path::PathBuf, sync::{Mutex, atomic::{AtomicU32, Ordering}}, time::Instant};
pub static ENGINE: Mutex<Option<Engine>> = Mutex::new(None);
pub static STATUS: Mutex<String> = Mutex::new(String::new());
pub static KEYS: AtomicU32 = AtomicU32::new(0x3ff);
pub const MAX_STEPS: u64 = 5_000_000;
pub fn status(message: impl Into<String>) { *STATUS.lock().unwrap_or_else(|e| e.into_inner()) = message.into(); }
pub struct Engine {
    pub machine: Box<Machine>,
    code: Option<NativeCode>,
    extensions: Vec<NativeCode>,
    pending: Option<std::sync::mpsc::Receiver<Result<NativeCode,String>>>,
    missed: BTreeSet<u32>,
    adaptive: bool,
    pub pixels: Vec<i32>,
    pub audio: Vec<i16>,
    pcm: crate::audio::Audio,
    pub native_blocks: u64,
    pub fallback_steps: u64,
    pub frame_ns: u64,
    pub prepare_ms: u64,
    pub save_path: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
    pub cache_hit: bool,
}
pub fn verify_rom(bytes: &[u8]) -> bool {
    let sha1: String = Sha1::digest(bytes).iter().map(|b| format!("{b:02x}")).collect();
    sha1 == "4ac05441f4de70a4ec3dd932116346c61b8783d9"
        && gba_core::ffta_rom::digest(bytes) == gba_core::ffta_rom::SHA256
}
impl Engine {
    pub fn new(rom: Vec<u8>) -> Self {
        Self { machine: Box::new(Machine::new(rom)), code: None, extensions: Vec::new(), pending: None,
            missed: BTreeSet::new(), adaptive: cfg!(target_os="android"), pixels: Vec::new(),
            audio: Vec::with_capacity(4096), pcm: crate::audio::Audio::default(), native_blocks: 0,
            fallback_steps: 0, frame_ns: 0, prepare_ms: 0, save_path: None, cache_dir: None, cache_hit: false }
    }
    pub fn prepare(&mut self) -> Result<(), String> {
        let start = Instant::now();
        if let Some(path)=self.cache_path("base") {
            if valid_cache(&path) {
                if let Ok(code)=compiler::load_object(&path) {
                    self.code=Some(code);
                    self.cache_hit=true;
                    for index in 0..16 {
                        if let Some(hot)=self.cache_path(&format!("hot-{index}")) {
                            if valid_cache(&hot) {
                                if let Ok(code)=compiler::load_object(&hot) {
                                    self.code.as_mut().unwrap().blocks.extend(code.blocks.iter().map(|(k,v)|(*k,*v)));
                                    self.extensions.push(code);
                                }
                            }
                        }
                    }
                    self.prepare_ms=start.elapsed().as_millis() as u64;
                    status("Ready (cached game)");
                    return Ok(());
                }
            }
        }
        status("Profiling boot code (90 frames)...");
        mem::set_extra(0);
        mem::set_extra_y(0);
        // Separate machine: preparation never advances the user's game/save.
        let mut profile = Machine::new(self.machine.bus.rom.clone());
        if let Some(save) = self.machine.bus.save_data() { profile.bus.load_save_data(save); }
        let mut seeds = BTreeSet::from([0x08000000]);
        for frame in 0..90 {
            let before = profile.bus.frames;
            let mut previous_end = u32::MAX;
            for _ in 0..MAX_STEPS {
                let pc = profile.cpu.regs[15];
                if matches!(pc >> 24, 2 | 3 | 8) && pc != previous_end {
                    seeds.insert(pc | profile.cpu.thumb() as u32);
                }
                previous_end = pc.wrapping_add(if profile.cpu.thumb() {2} else {4});
                profile.step();
                if profile.bus.frames != before { break; }
            }
            profile.bus.audio_buf.clear();
            if frame % 15 == 0 { status(format!("Profiling boot: {frame}/90 frames")); }
        }
        status("Discovering native ARM/Thumb blocks...");
        let view = recomp_core::analyze::View { rom: &profile.bus.rom, ewram: Some(&profile.bus.ewram), iwram: Some(&profile.bus.iwram), bios: None };
        let analysis = recomp_core::analyze::analyze(&view, &seeds.into_iter().collect::<Vec<_>>());
        let object=self.cache_path("base");
        self.code = Some(compiler::compile(&analysis, &view, object.as_deref(), |done, total| {
            status(format!("Compiling native blocks: {done}/{total}"));
        })?);
        if let Some(path)=&object { seal_cache(path)?; }
        self.prepare_ms = start.elapsed().as_millis() as u64;
        status(format!("Ready: {} native blocks, {} ms", self.code.as_ref().unwrap().blocks.len(), self.prepare_ms));
        Ok(())
    }
    pub fn frame(&mut self, count: usize, extra_x: usize, extra_y: usize, sound: bool) -> Result<(), String> {
        self.frame_audio(count,extra_x,extra_y,if sound{1}else{0})
    }
    pub fn frame_audio(&mut self,count:usize,extra_x:usize,extra_y:usize,mode:u32)->Result<(),String>{
        let start = Instant::now();
        self.collect_compilation();
        mem::set_extra(extra_x);
        mem::set_extra_y(extra_y);
        self.audio.clear();
        self.pcm.begin(count.clamp(1,8),mode,
            self.cache_dir.as_deref().unwrap_or(&std::env::temp_dir()))?;
        for _ in 0..count.clamp(1, 8) {
            let m = &mut *self.machine;
            m.bus.keys = (KEYS.load(Ordering::Relaxed) as u16) & 0x3ff;
            m.bus.frame_ready = false;
            let mut previous_end = u32::MAX;
            for _ in 0..MAX_STEPS {
                if m.bus.frame_ready { break; }
                let pc = m.cpu.regs[15];
                let native = if m.bus.halted
                    || (pc == gba_core::machine::IRQ_RETURN_ADDR && m.cpu.mode() == gba_core::Mode::Irq)
                    || (m.bus.irq_pending() && !m.cpu.flag(FLAG_I))
                    || m.ffta_sprite_interpret(pc) { None }
                    else { self.code.as_ref().and_then(|code| code.blocks.get(&(pc | m.cpu.thumb() as u32))).copied() };
                if let Some(function) = native {
                    if let Some(h) = m.bus.mp2k.as_deref() {
                        let key = pc | m.cpu.thumb() as u32;
                        if h.active && h.hook_match(key) { m.bus.mp2k_frame_hook(key); }
                    }
                    function(&gba_core::capi::RT_API, m as *mut _ as *mut _);
                    self.native_blocks += 1;
                    previous_end = u32::MAX;
                } else {
                    if self.adaptive && pc != previous_end && matches!(pc>>24,2|3|8) && self.missed.len()<2048
                        && self.code.as_ref().is_some_and(|code|!code.blocks.contains_key(&(pc|m.cpu.thumb() as u32))) {
                        self.missed.insert(pc | m.cpu.thumb() as u32);
                    }
                    previous_end = pc.wrapping_add(if m.cpu.thumb(){2}else{4});
                    m.step(); self.fallback_steps += 1;
                }
            }
            if !m.bus.frame_ready { return Err(format!("Frame stalled at PC {:08x}", m.cpu.regs[15])); }
            self.pcm.feed(&m.bus.audio_buf);
            m.bus.audio_buf.clear();
        }
        self.pcm.finish(&mut self.audio)?;
        let size = mem::render_width() * mem::render_height();
        self.pixels.resize(size, 0);
        let expanded_rows=mem::extra_y()>0 && (self.machine.bus.ffta_movement.enabled || self.machine.bus.ffta_stage.enabled)
            && (0..4).any(|bg|gba_core::ppu::ffta_terrain(&self.machine.bus,bg)
                || gba_core::ffta_stage::layer(&self.machine.bus,bg).is_some());
        let padded=mem::extra_y()>0 && !expanded_rows;
        if padded { self.pixels.fill(0xff000000u32 as i32); }
        let source = if mem::extra() == 0 && mem::extra_y() == 0 {
            &self.machine.bus.framebuffer[..]
        } else { &self.machine.bus.wide_framebuffer[..if padded{mem::render_width()*160}else{size}] };
        let offset=if padded{mem::extra_y()*mem::render_width()}else{0};
        let colors=rgba_colors();
        for (out,&pixel) in self.pixels[offset..].iter_mut().zip(source){
            *out=colors[(pixel&0x7fff) as usize];
        }
        self.frame_ns = start.elapsed().as_nanos() as u64;
        if self.adaptive && self.machine.bus.frames % 120 < count as u64 { self.start_compilation(); }
        Ok(())
    }
    fn collect_compilation(&mut self) {
        let result = self.pending.as_ref().and_then(|rx|rx.try_recv().ok());
        if let Some(result) = result {
            self.pending = None;
            match result {
                Ok(code) => {
                    if let Some(primary) = &mut self.code { primary.blocks.extend(code.blocks.iter().map(|(k,v)|(*k,*v))); }
                    self.extensions.push(code);
                }
                Err(error) => status(format!("Background compiler: {error}; interpreter remains available")),
            }
        }
    }
    fn start_compilation(&mut self) {
        if self.pending.is_some() || self.code.is_none() || self.missed.is_empty() || self.extensions.len()>=16 { return; }
        let seeds=std::mem::take(&mut self.missed);
        let rom=self.machine.bus.rom.clone();
        let ewram=self.machine.bus.ewram.clone();
        let iwram=self.machine.bus.iwram.clone();
        let object=self.cache_path(&format!("hot-{}",self.extensions.len()));
        let (tx,rx)=std::sync::mpsc::channel();
        self.pending=Some(rx);
        std::thread::spawn(move || {
            #[cfg(target_os="android")]
            unsafe { libc::setpriority(libc::PRIO_PROCESS,0,10); }
            let view=recomp_core::analyze::View{rom:&rom,ewram:Some(&ewram),iwram:Some(&iwram),bios:None};
            let mut analysis=recomp_core::analyze::analyze(&view,&seeds.iter().copied().collect::<Vec<_>>());
            // Only observed misses are compiled; generated RAM blocks keep content guards.
            analysis.blocks.retain(|block|seeds.contains(&block.key()));
            let result=compiler::compile(&analysis,&view,object.as_deref(),|_,_|{}).and_then(|code|{
                if let Some(path)=&object{seal_cache(path)?;}
                Ok(code)
            });
            let _=tx.send(result);
        });
    }
    fn cache_path(&self,kind:&str)->Option<PathBuf> {
        let directory=self.cache_dir.as_ref()?;
        if std::fs::create_dir_all(directory).is_err(){return None;}
        Some(directory.join(format!("{kind}-{}.o",env!("FFTA_ANDROID_CACHE_TAG"))))
    }
    pub fn save(&self) -> Result<(), String> {
        if let (Some(path), Some(data)) = (&self.save_path, self.machine.bus.save_data()) {
            let temp = path.with_extension("sav.tmp");
            std::fs::write(&temp, data).map_err(|e| e.to_string())?;
            std::fs::rename(temp, path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub fn diagnostics(&self) -> String {
        let hash = self.pixels.iter().fold(0xcbf29ce484222325u64, |h,p| (h ^ *p as u32 as u64).wrapping_mul(0x100000001b3));
        let oam_hash=self.machine.bus.oam.iter().fold(0xcbf29ce484222325u64,|h,&v|(h^v as u64).wrapping_mul(0x100000001b3));
        format!("OAM FNV64: {oam_hash:016x}\nBackend: {}\nNative blocks: {}\nNative dispatches: {}\nInterpreter fallback instructions: {}\nFrames: {}\nLast frame CPU: {:.2} ms\nPreparation: {} ms\nRender: {}x{}\nFramebuffer FNV64: {hash:016x}\nPC: {:08x}\nGame audio: real GBA PCM 65536 -> 48000 Hz\nCache hit: {}\n", 
            if self.code.is_some() {"libtcc in-process native blocks"} else {"cached interpreter (not recompiled)"},
            self.code.as_ref().map_or(0,|c|c.blocks.len()),self.native_blocks,self.fallback_steps,self.machine.bus.frames,
            self.frame_ns as f64/1e6,self.prepare_ms,mem::render_width(),mem::render_height(),self.machine.cpu.regs[15],self.cache_hit)
    }
}
fn rgba_colors()->&'static[i32]{
    static COLORS:std::sync::OnceLock<Box<[i32]>>=std::sync::OnceLock::new();
    COLORS.get_or_init(||(0u32..32768).map(|pixel|{
        let r=(pixel&31)*255/31;
        let g=((pixel>>5)&31)*255/31;
        let b=((pixel>>10)&31)*255/31;
        (0xff000000|(r<<16)|(g<<8)|b) as i32
    }).collect::<Vec<_>>().into_boxed_slice())
}
fn seal_cache(path:&std::path::Path)->Result<(),String>{
    let data=std::fs::read(path).map_err(|e|e.to_string())?;
    std::fs::write(path.with_extension("sha256"),gba_core::ffta_rom::digest(&data)).map_err(|e|e.to_string())
}
fn valid_cache(path:&std::path::Path)->bool{
    if !std::fs::metadata(path).is_ok_and(|meta|meta.len()>0 && meta.len()<64*1024*1024){return false;}
    match (std::fs::read(path),std::fs::read_to_string(path.with_extension("sha256"))){
        (Ok(data),Ok(hash))=>gba_core::ffta_rom::digest(&data)==hash,
        _=>false,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cached_color_conversion_matches_every_hardware_color(){
        for pixel in 0u32..65536{
            let r=(pixel&31)*255/31;let g=((pixel>>5)&31)*255/31;let b=((pixel>>10)&31)*255/31;
            assert_eq!(rgba_colors()[(pixel&0x7fff) as usize],(0xff000000|(r<<16)|(g<<8)|b) as i32);
        }
    }
    static VIEW_TEST:std::sync::Mutex<()>=std::sync::Mutex::new(());
    #[test]
    fn rejects_large_arbitrary_file() { assert!(!verify_rom(&vec![0; 8*1024*1024])); }
    #[test]
    fn real_bus_keys_and_ppu_match_desktop_for_600_frames() {
        let _lock=VIEW_TEST.lock().unwrap();
        let mut rom = vec![0; 192];
        rom[..4].copy_from_slice(&0xeafffffeu32.to_le_bytes());
        let mut mobile = Engine::new(rom.clone());
        let mut reference = Machine::new(rom);
        for frame in 0..600 {
            let keys = if frame % 2 == 0 {0x3fe} else {0x3ff};
            KEYS.store(keys, Ordering::Relaxed);
            reference.bus.keys = keys as u16;
            reference.run_frame(MAX_STEPS);
            mobile.frame(1,0,0,false).unwrap();
            assert_eq!(mobile.machine.bus.framebuffer, reference.bus.framebuffer);
            assert_eq!(mobile.machine.bus.oam, reference.bus.oam);
            assert_eq!(mobile.machine.bus.keys, reference.bus.keys);
            assert_eq!(mobile.machine.cpu.regs, reference.cpu.regs);
        }
    }
    #[test]
    fn square_view_does_not_show_stale_rows_outside_battle() {
        let _lock=VIEW_TEST.lock().unwrap();
        let mut rom=vec![0;192];
        rom[..4].copy_from_slice(&0xeafffffeu32.to_le_bytes());
        let mut engine=Engine::new(rom);
        engine.machine.bus.io[0]=0x80;
        engine.frame(1,8,48,false).unwrap();
        assert_eq!(engine.pixels.len(),256*256);
        assert_eq!(engine.pixels[0],0xff000000u32 as i32);
        assert_eq!(engine.pixels[48*256],0xffffffffu32 as i32);
        assert_eq!(engine.pixels[255*256],0xff000000u32 as i32);
        mem::set_extra(0);mem::set_extra_y(0);
    }
    #[test]
    #[ignore = "requires a native TinyCC library and the user's exact ROM"]
    fn recompiled_ffta_matches_desktop_for_600_frames() {
        let rom = std::fs::read(std::env::var_os("FFTA_TEST_ROM").expect("FFTA_TEST_ROM")).unwrap();
        assert!(verify_rom(&rom));
        let mut mobile = Engine::new(rom.clone());
        let mut reference = Machine::new(rom);
        mobile.adaptive = true;
        let cache = std::env::var_os("FFTA_ANDROID_CACHE_TEST").map(PathBuf::from);
        mobile.cache_dir = cache.clone();
        println!("FFTA_ANDROID test: preparing native code");
        mobile.prepare().unwrap();
        if cache.is_some() {
            let mut warm=Engine::new(mobile.machine.bus.rom.clone());
            warm.cache_dir=cache;
            warm.adaptive=true;
            warm.prepare().unwrap();
            assert!(warm.cache_hit,"warm start must load objects without profiling/recompiling");
            println!("FFTA_ANDROID warm cache hit: {} ms",warm.prepare_ms);
            mobile=warm;
        }
        println!("FFTA_ANDROID test: prepared {} blocks", mobile.code.as_ref().unwrap().blocks.len());
        for _ in 0..600 {
            KEYS.store(0x3ff,Ordering::Relaxed);
            reference.run_frame(MAX_STEPS);
            reference.bus.audio_buf.clear();
            mobile.frame(1,0,0,false).unwrap();
            assert_eq!(mobile.machine.bus.framebuffer,reference.bus.framebuffer,"frame {}",reference.bus.frames);
            assert_eq!(mobile.machine.bus.oam,reference.bus.oam,"OAM frame {}",reference.bus.frames);
        }
        println!("FFTA_ANDROID 600 frames framebuffer/OAM IDENTICAL\n{}",mobile.diagnostics());
    }
}
