use crate::dwm::*;
use fdmux::*;
use std::io::Write;

struct Stream {
    handle: Option<Window>,
    output: Option<NamedWritePipe>,
    name: String
}

fn prefix_eq(prefix: &String, s: &String) -> bool {
    s.len() >= prefix.len() && s[0..prefix.len()] == *prefix
}

impl Stream {
    fn new(name: String) -> Self {
        Stream { handle: None, output: None, name: name }
    }

    fn try_window(&mut self, dpy: Ptr, handle: Window, name: &String) -> Option<CLenStr> {
        if self.handle.is_none() && prefix_eq(&self.name, name) {
            unsafe { XGrabKey(dpy, ANY_KEY, ANY_MODIFIER, handle, true, GRAB_MODE_ASYNC, GRAB_MODE_ASYNC) };
            match NamedWritePipe::new(format!("/tmp/dwm-{}-{}.xev", self.name, handle)) {
                Ok(pipe) => {
                    let ptr = CLenStr::new(pipe.name.as_bytes());
                    self.output = Some(pipe);
                    self.handle = Some(handle);
                    Some(ptr)
                }, Err(_) => None
            }
        } else {
            None
        }
    }

    fn try_key(&mut self, handle: Window, key: KeySym, modkeys: u16) -> bool {
        self.handle.map(|my_handle| {
            if handle == my_handle {
                match &mut self.output {
                    Some(o) => {o.write(format!("key:{:x}/{:x}\n\0", key, modkeys).as_bytes()).unwrap();},
                    None => {}
                };

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

    pub fn remove(&mut self, handle: Window) {
        for i in (0..self.streams.len()).rev() {
            match &self.streams[i].handle {
                Some(h) => {
                    if *h == handle {
                        self.streams.remove(i);
                    }
                }, None => {}
            }
        }
    }

    fn try_window(&mut self, handle: Window, name: String) -> Option<CLenStr> {
        for s in self.streams.iter_mut() {
            match s.try_window(self.dpy, handle, &name) {
                Some(s) => {return Some(s);},
                None => {}
            }
        }
        
        None
    }

    fn try_key(&mut self, handle: Window, key: KeySym, modkeys: u16) -> bool {
        for s in self.streams.iter_mut() {
            if s.try_key(handle, key, modkeys) {
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
fn end_stream(s: &mut Streams, handle: Window) {
    s.remove(handle);
}

#[no_mangle]
extern "C"
fn win2stream(s: &mut Streams, handle: Window, name: CStr) -> CLenStr {
    s.try_window(handle, ptr2str(name))
     .unwrap_or(CLenStr::null())
}

#[no_mangle]
fn key2stream(s: &mut Streams, handle: Window, key: KeySym, modkeys: u16) {
    s.try_key(handle, key, modkeys);
}

#[no_mangle]
fn free_streams(_s: Box<Streams>) {}

