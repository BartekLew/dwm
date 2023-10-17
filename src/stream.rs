use crate::dwm::*;
use fdmux::*;
use std::io::Write;

pub enum StreamType { Trace, Grab }

struct Stream {
    handle: Option<Window>,
    output: Option<NamedWritePipe>,
    typ: StreamType,
    name: String
}

fn prefix_eq(prefix: &String, s: &String) -> bool {
    s.len() >= prefix.len() && s[0..prefix.len()] == *prefix
}

impl Stream {
    fn new_grab(name: String) -> Self {
        Stream { handle: None, typ: StreamType::Grab, output: None, name: name }
    }

    fn new_trace(name:String, handle: Window) -> Self {
        unsafe { XGrabKey(dpy, ANY_KEY, ANY_MODIFIER, handle, true, GRAB_MODE_SYNC, GRAB_MODE_SYNC) };
        Stream { handle: Some(handle), name, typ: StreamType::Trace, output: None }
    }

    fn try_window(&mut self, disp: Ptr, handle: Window, name: &String) -> Option<CLenStr> {
        if self.handle.is_none() && prefix_eq(&self.name, name) {
            unsafe { XGrabKey(disp, ANY_KEY, ANY_MODIFIER, handle, true, GRAB_MODE_ASYNC, GRAB_MODE_ASYNC) };
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

    fn try_key(&mut self, ev: &XKeyEvent, key: KeySym) -> bool {
        self.handle.map(|my_handle| {
            if ev.window == my_handle {
                match &mut self.output {
                    Some(o) => {
                        o.write(format!("key:{:x}/{:x}\n\0", key, ev.state).as_bytes()).unwrap();
                    },
                    None => {
                        println!("key:{:x}/{:x} @ {:#x}", key, ev.state, ev.window);
                    }
                };

                match self.typ {
                    StreamType::Trace => {
                        unsafe {
                            XAllowEvents(dpy, 5, ev.time);// ReplayKeyboard
                            XFlush(dpy);
                        }
                    },
                    _ => {}
                }

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
    pub fn new(disp: Ptr) -> Self {
        Streams { streams: Vec::with_capacity(5), dpy: disp }
    }

    pub fn add_grab(&mut self, prefix: String) {
        self.streams.push(Stream::new_grab(prefix));        
    }

    pub fn add_trace(&mut self, client: &Client) {
        self.streams.push(Stream::new_trace(client.name_str(), client.win));
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

    fn try_key(&mut self, ev: &XKeyEvent, key: KeySym) -> bool {
        for s in self.streams.iter_mut() {
            if s.try_key(ev, key) {
                return true;
            }
        }
        
        false
    }

}

#[no_mangle]
extern "C"
fn init_streams(disp: Ptr) -> Box<Streams> {
    Box::new(Streams::new(disp))
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
fn key2stream(s: &mut Streams, ev: &XKeyEvent, key: KeySym) {
    s.try_key(ev, key);
}

#[no_mangle]
fn free_streams(_s: Box<Streams>) {}

