use std::str;
use std::slice;
use std::ffi::c_void;

type Window = u64;
type KeySym = u64;
type Ptr = *const c_void;
type CStr = *const u8;

const ANY_KEY : i32 = 0;
const ANY_MODIFIER : u32 = 1 << 15;
const GRAB_MODE_ASYNC : i32 = 1;

extern "C" {
    fn strlen(cstr: CStr) -> usize;
    fn printf(fmt: CStr, ...) -> usize;
    fn XGrabKey(dpy: Ptr, key: i32, mods: u32, tgt: Window, owner_events: bool,
                ptr_mode: i32, key_mode: i32) -> i32;
}

struct Stream {
    handle: Option<Window>,
    name: String
}

fn prefix_eq(prefix: &String, s: &String) -> bool {
    s.len() >= prefix.len() && s[0..prefix.len()] == *prefix
}

impl Stream {
    fn new(name: String) -> Self {
        Stream { handle: None, name: name }
    }

    fn try_window(&mut self, dpy: Ptr, handle: Window, name: &String) -> bool {
        if self.handle.is_none() && prefix_eq(&self.name, name) {
            self.handle = Some(handle);
            unsafe { XGrabKey(dpy, ANY_KEY, ANY_MODIFIER, handle, true, GRAB_MODE_ASYNC, GRAB_MODE_ASYNC) };
            true
        } else {
            false
        }
    }

    fn try_key(&mut self, handle: Window, key: KeySym) -> bool {
        self.handle.map(|my_handle| {
            if handle == my_handle {
                unsafe { printf("Hit '%c' for 0x%x\n\0".as_ptr(), key as usize, handle) };
                true
            } else {
                false
            }
        }).unwrap_or(false)
    }
}

struct Streams {
    streams: Vec<Stream>,
    dpy: Ptr
}

impl Streams {
    fn new(dpy: Ptr) -> Self {
        Streams { streams: Vec::with_capacity(5), dpy: dpy }
    }

    fn add(&mut self, prefix: String) {
        self.streams.push(Stream::new(prefix));        
    }

    fn try_window(&mut self, handle: Window, name: String) -> bool{
        for s in self.streams.iter_mut() {
            if s.try_window(self.dpy, handle, &name) {
                return true;
            }
        }
        
        false
    }

    fn try_key(&mut self, handle: Window, key: KeySym) -> bool {
        for s in self.streams.iter_mut() {
            if s.try_key(handle, key) {
                return true;
            }
        }
        
        false
    }

}

fn ptr2str(ptr: CStr) -> String {
    unsafe {
        String::from(str::from_utf8(slice::from_raw_parts(ptr, strlen(ptr))).unwrap())
    }
}

#[no_mangle]
extern "C"
fn init_streams(dpy: Ptr) -> Box<Streams> {
    Box::new(Streams::new(dpy))
}

#[no_mangle]
extern "C"
fn new_stream(s: &mut Streams, name: CStr) {
    s.add(String::from(ptr2str(name)));
}

#[no_mangle]
extern "C"
fn win2stream(s: &mut Streams, handle: Window, name: CStr) -> bool {
    s.try_window(handle, ptr2str(name))
}

#[no_mangle]
fn key2stream(s: &mut Streams, handle: Window, key: KeySym) {
    s.try_key(handle, key);
}

#[no_mangle]
fn free_streams(_s: Box<Streams>) {}

