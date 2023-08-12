use crate::dwm::*;

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

pub struct Streams {
    streams: Vec<Stream>,
    dpy: Ptr
}

impl Streams {
    pub fn new(dpy: Ptr) -> Self {
        Streams { streams: Vec::with_capacity(5), dpy: dpy }
    }

    pub fn add(&mut self, prefix: String) {
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

