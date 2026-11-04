//! Wine can deliver a confirmed IME result after ending composition. Forward that
//! late result through winit's normal IME path for this command window only.
use std::{ffi::{c_char,c_void},ptr,sync::Arc};
use winit::{raw_window_handle::{HasWindowHandle,RawWindowHandle},window::Window};

type Hwnd=*mut c_void;
type SubclassProc=unsafe extern "system" fn(Hwnd,u32,usize,isize,usize,usize)->isize;
const WM_IME_STARTCOMPOSITION:u32=0x010d;
const WM_IME_ENDCOMPOSITION:u32=0x010e;
const WM_IME_COMPOSITION:u32=0x010f;
const WM_KILLFOCUS:u32=0x0008;
const WM_NCDESTROY:u32=0x0082;
const GCS_RESULTSTR:u32=0x0800;
#[link(name="comctl32")]
extern "system" {
    fn SetWindowSubclass(hwnd:Hwnd,proc:Option<SubclassProc>,id:usize,data:usize)->i32;
    fn RemoveWindowSubclass(hwnd:Hwnd,proc:Option<SubclassProc>,id:usize)->i32;
    fn DefSubclassProc(hwnd:Hwnd,msg:u32,wparam:usize,lparam:isize)->isize;
}
#[link(name="imm32")]
extern "system" {
    fn ImmGetContext(hwnd:Hwnd)->*mut c_void;
    fn ImmReleaseContext(hwnd:Hwnd,context:*mut c_void)->i32;
    fn ImmGetCompositionStringW(context:*mut c_void,index:u32,buffer:*mut c_void,len:u32)->i32;
}
#[link(name="kernel32")]
extern "system" {
    fn GetModuleHandleA(name:*const c_char)->*mut c_void;
    fn GetProcAddress(module:*mut c_void,name:*const c_char)->*mut c_void;
}

#[derive(Default)]
struct Composition { active:bool, result_seen:bool, awaiting_late:bool }
impl Composition {
    fn start(&mut self){*self=Self{active:true,..Self::default()};}
    fn end(&mut self,result_available:bool){
        self.awaiting_late=self.active && !self.result_seen && !result_available;
        self.active=false;
    }
    fn result(&mut self,nonempty:bool)->bool{
        if !nonempty{return false;}
        if self.awaiting_late {self.awaiting_late=false;return true;}
        if self.active {self.result_seen=true;}
        false
    }
    fn reset(&mut self){*self=Self::default();}
}

// Only query the IME's actual committed-result length; preedit is never used as
// a substitute, because an ended composition can also have been cancelled.
unsafe fn has_result(hwnd:Hwnd)->bool{
    let context=ImmGetContext(hwnd);
    if context.is_null(){return false;}
    let len=ImmGetCompositionStringW(context,GCS_RESULTSTR,ptr::null_mut(),0);
    ImmReleaseContext(hwnd,context);
    len>0
}
unsafe extern "system" fn subclass(hwnd:Hwnd,msg:u32,wparam:usize,lparam:isize,_id:usize,data:usize)->isize{
    let state=data as *mut Composition;
    let recover=match msg {
        WM_IME_STARTCOMPOSITION=>{(*state).start();false},
        WM_IME_ENDCOMPOSITION=>{let available=has_result(hwnd);(*state).end(available);false},
        WM_IME_COMPOSITION if (lparam as u32 & GCS_RESULTSTR)!=0=>{let available=has_result(hwnd);(*state).result(available)},
        WM_KILLFOCUS|WM_NCDESTROY=>{(*state).reset();false},
        _=>false,
    };
    if recover {
        // DefSubclassProc enters winit's window procedure directly, so these
        // synthetic boundaries do not reenter this subclass or duplicate text.
        DefSubclassProc(hwnd,WM_IME_STARTCOMPOSITION,0,0);
        let result=DefSubclassProc(hwnd,msg,wparam,lparam);
        DefSubclassProc(hwnd,WM_IME_ENDCOMPOSITION,0,0);
        result
    }else{DefSubclassProc(hwnd,msg,wparam,lparam)}
}

pub(crate) struct Shim { window:Arc<Window>, hwnd:Hwnd, state:*mut Composition }
impl Shim {
    pub fn install(window:Arc<Window>)->Option<Self>{
        // Wine exports this ntdll symbol; native Windows follows winit unchanged.
        unsafe {
            let module=GetModuleHandleA(b"ntdll.dll\0".as_ptr().cast());
            if module.is_null() || GetProcAddress(module,b"wine_get_version\0".as_ptr().cast()).is_null(){return None;}
            let handle=window.window_handle().ok()?;
            let RawWindowHandle::Win32(handle)=handle.as_raw() else{return None};
            let hwnd=handle.hwnd.get() as Hwnd;
            let state=Box::into_raw(Box::<Composition>::default());
            if SetWindowSubclass(hwnd,Some(subclass),state as usize,state as usize)==0 {
                drop(Box::from_raw(state));
                eprintln!("Command console: Wine IME compatibility hook unavailable.");
                return None;
            }
            Some(Self{window,hwnd,state})
        }
    }
    pub fn reset(&mut self){unsafe{(*self.state).reset();}}
}
impl Drop for Shim {
    fn drop(&mut self){
        // The Arc keeps the HWND alive until its subclass is removed.
        // If removal fails, retain the state so a possible callback cannot use
        // freed memory; window subclass removal normally runs on its UI thread.
        unsafe {if RemoveWindowSubclass(self.hwnd,Some(subclass),self.state as usize)!=0{drop(Box::from_raw(self.state));}}
        let _=&self.window;
    }
}

#[cfg(test)]
mod tests {
    use super::Composition;
    #[test]
    fn only_an_actual_late_result_recovers_once_per_generation(){
        let mut s=Composition::default();s.start();s.end(false);
        assert!(!s.result(false));assert!(s.result(true));assert!(!s.result(true));
        s.start();s.end(false);assert!(s.result(true));
    }
    #[test]
    fn cancellation_in_order_result_and_lifecycle_never_recover_stale_text(){
        let mut s=Composition::default();s.start();s.end(false);s.start();assert!(!s.result(true));s.end(true);assert!(!s.result(true));
        s.start();s.end(true);assert!(!s.result(true));
        s.start();s.end(false);s.reset();assert!(!s.result(true));
    }
}
