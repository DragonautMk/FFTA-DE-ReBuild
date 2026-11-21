use crate::engine::{self, Engine, ENGINE, KEYS, STATUS};
use jni_sys::*;
use std::{path::PathBuf, sync::atomic::Ordering};

fn guarded(action: impl FnOnce() -> Result<i32, String>) -> i32 {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(action)) {
        Ok(Ok(value)) => value,
        Ok(Err(error)) => { engine::status(format!("ERROR: {error}")); -1 },
        Err(_) => { engine::status("ERROR: native engine panicked; reload the ROM"); -1 },
    }
}

unsafe fn string(env: *mut JNIEnv, value: jstring) -> Result<String, String> {
    if value.is_null() { return Err("Missing path".into()); }
    let api = &(**env).v1_4;
    let len = (api.GetStringLength)(env,value) as usize;
    let data = (api.GetStringChars)(env,value,std::ptr::null_mut());
    if data.is_null() { return Err("Cannot read Java string".into()); }
    let text = String::from_utf16(std::slice::from_raw_parts(data,len)).map_err(|e|e.to_string());
    (api.ReleaseStringChars)(env,value,data);
    text
}

unsafe fn java_string(env: *mut JNIEnv, text: &str) -> jstring {
    let utf16: Vec<_> = text.encode_utf16().collect();
    ((**env).v1_4.NewString)(env,utf16.as_ptr(),utf16.len() as i32)
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_fftarecomp_port_NativeBridge_gbaLoadRom(
    env: *mut JNIEnv, _: jobject, bytes: jbyteArray, save_path: jstring,
) -> jint {
    guarded(|| {
        if bytes.is_null() { return Err("No ROM bytes".into()); }
        let api = &(**env).v1_4;
        let len = (api.GetArrayLength)(env,bytes);
        if !(192..=32*1024*1024).contains(&len) { return Err("Invalid ROM size".into()); }
        let mut rom = vec![0u8;len as usize];
        (api.GetByteArrayRegion)(env,bytes,0,len,rom.as_mut_ptr() as *mut _);
        if (api.ExceptionCheck)(env) { return Err("Cannot read ROM array".into()); }
        if !engine::verify_rom(&rom) { return Err("ROM SHA1/SHA256 mismatch: exact FFTA USA is required".into()); }
        let save = PathBuf::from(string(env,save_path)?);
        let mut loaded = Engine::new(rom);
        if save.is_file() {
            let bytes = std::fs::read(&save).map_err(|e|format!("Read save: {e}"))?;
            loaded.machine.bus.load_save_data(&bytes);
        }
        loaded.save_path = Some(save);
        loaded.cache_dir = loaded.save_path.as_ref().and_then(|path|path.parent()).map(|path|path.join("compiled"));
        *ENGINE.lock().map_err(|_|"Engine mutex poisoned")? = Some(loaded);
        KEYS.store(0x3ff,Ordering::Relaxed);
        engine::status("ROM verified and loaded; ready to compile");
        Ok(1)
    })
}

#[no_mangle]
pub extern "system" fn Java_com_fftarecomp_port_NativeBridge_gbaPrepare(_: *mut JNIEnv, _: jobject) -> jint {
    guarded(|| {
        let mut lock = ENGINE.lock().map_err(|_|"Engine mutex poisoned")?;
        lock.as_mut().ok_or("Load a ROM first")?.prepare()?;
        Ok(1)
    })
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_fftarecomp_port_NativeBridge_gbaFrame(
    env: *mut JNIEnv, _: jobject, keys: jint, extra_x: jint, extra_y: jint,
    frames: jint, sound: jboolean, audio_mode:jint, pixels: jobject, audio: jshortArray,
) -> jint {
    guarded(|| {
        if pixels.is_null() || audio.is_null() { return Err("Missing frame buffers".into()); }
        if !(0..=gba_core::mem::MAX_EXTRA as i32).contains(&extra_x)
            || !(0..=gba_core::mem::MAX_EXTRA_Y as i32).contains(&extra_y)
            || !(1..=8).contains(&frames) { return Err("Invalid render configuration".into()); }
        KEYS.store(keys as u32 & 0x3ff,Ordering::Relaxed);
        let api = &(**env).v1_4;
        let size = (240+2*extra_x)*(160+2*extra_y);
        if (api.GetDirectBufferCapacity)(env,pixels)<size as i64*4 || (api.GetArrayLength)(env,audio)<4096 {
            return Err("Frame buffer size mismatch".into());
        }
        let target = (api.GetDirectBufferAddress)(env,pixels) as *mut i32;
        if target.is_null() { return Err("Expected a direct pixel buffer".into()); }
        let mut lock = ENGINE.lock().map_err(|_|"Engine mutex poisoned")?;
        let engine = lock.as_mut().ok_or("ROM not loaded")?;
        engine.frame_audio(frames as usize, extra_x as usize, extra_y as usize,
            if !sound{0}else if audio_mode==2{2}else{1})?;
        std::ptr::copy_nonoverlapping(engine.pixels.as_ptr(),target,size as usize);
        let count = engine.audio.len().min(4096) as i32;
        if count > 0 { (api.SetShortArrayRegion)(env,audio,0,count,engine.audio.as_ptr()); }
        if (api.ExceptionCheck)(env) { return Err("Cannot copy frame data".into()); }
        Ok(count)
    })
}

#[no_mangle]
pub extern "system" fn Java_com_fftarecomp_port_NativeBridge_gbaSave(_: *mut JNIEnv, _: jobject) -> jint {
    guarded(|| {
        if let Some(engine) = ENGINE.lock().map_err(|_|"Engine mutex poisoned")?.as_ref() { engine.save()?; }
        Ok(1)
    })
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_fftarecomp_port_NativeBridge_gbaStatus(env: *mut JNIEnv, _: jobject) -> jstring {
    java_string(env,&STATUS.lock().unwrap_or_else(|e|e.into_inner()))
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_fftarecomp_port_NativeBridge_gbaDiagnostics(env: *mut JNIEnv, _: jobject) -> jstring {
    let text = match ENGINE.try_lock() {
        Ok(engine) => engine.as_ref().map_or_else(||"No ROM loaded".into(), Engine::diagnostics),
        Err(_) => "Engine busy compiling/running".into(),
    };
    java_string(env,&text)
}

#[no_mangle]
pub extern "system" fn Java_com_fftarecomp_port_NativeBridge_gbaCamera(_: *mut JNIEnv,_:jobject)->jlong{
    let lock=ENGINE.lock().unwrap_or_else(|e|e.into_inner());
    let Some(engine)=lock.as_ref()else{return -1;};
    let io=&engine.machine.bus.io;
    for bg in 0..4{
        let cnt=u16::from_le_bytes([io[8+bg*2],io[9+bg*2]]);
        if cnt==0x4c03 || cnt==0x4e02{
            let h=u16::from_le_bytes([io[16+bg*4],io[17+bg*4]])&511;
            let v=u16::from_le_bytes([io[18+bg*4],io[19+bg*4]])&511;
            return h as i64 | ((v as i64)<<9);
        }
    }
    -1
}

#[no_mangle]
pub extern "system" fn Java_com_fftarecomp_port_NativeBridge_gbaParallelZoom(_: *mut JNIEnv,_:jobject,enabled:jboolean){
    gba_core::ppu::set_parallel_zoom(enabled);
}
