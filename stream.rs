
use std::str;
use std::slice;

#[repr(C)]
struct Window {}

extern "C" {
    fn strlen(cstr: *const u8) -> usize;
    fn printf(fmt: *const u8, ...) -> usize;
}

struct Stream {
    handle: Option<*const Window>,
    name: String
}

fn prefix_eq(prefix: &String, s: &String) -> bool {
    s.len() >= prefix.len() && s[0..prefix.len()] == *prefix
}

impl Stream {
    fn new(name: String) -> Self {
        Stream { handle: None, name: name }
    }

    fn try_window(&mut self, handle: *const Window, name: &String) -> bool {
        if self.handle.is_none() && prefix_eq(&self.name, name) {
            self.handle = Some(handle);
            true
        } else {
            false
        }
    }

    fn try_key(&mut self, handle: *const Window, key: u8) -> bool {
        self.handle.map(|my_handle| {
            if handle == my_handle {
                unsafe { printf("Hit '%c' for %x\n\0".as_ptr(), key as usize, handle) };
                true
            } else {
                false
            }
        }).unwrap_or(false)
    }
}

struct Streams {
    streams: Vec<Stream>
}

impl Streams {
    fn new() -> Self {
        Streams { streams: Vec::with_capacity(5) }
    }

    fn add(&mut self, prefix: String) {
        self.streams.push(Stream::new(prefix));        
    }

    fn try_window(&mut self, handle: *const Window, name: String) -> bool{
        for s in self.streams.iter_mut() {
            if s.try_window(handle, &name) {
                return true;
            }
        }
        
        false
    }

    fn try_key(&mut self, handle: *const Window, key: u8) -> bool {
        for s in self.streams.iter_mut() {
            if s.try_key(handle, key) {
                return true;
            }
        }
        
        false
    }

}

fn ptr2str(ptr: *const u8) -> String {
    unsafe {
        String::from(str::from_utf8(slice::from_raw_parts(ptr, strlen(ptr))).unwrap())
    }
}

#[no_mangle]
extern "C"
fn init_streams() -> Box<Streams> {
    Box::new(Streams::new())
}

#[no_mangle]
extern "C"
fn new_stream(s: &mut Streams, name: *const u8) {
    s.add(String::from(ptr2str(name)));
}

#[no_mangle]
extern "C"
fn win2stream(s: &mut Streams, handle: *const Window, name: *const u8) -> bool {
    s.try_window(handle, ptr2str(name))
}

#[no_mangle]
fn key2stream(s: &mut Streams, handle: *const Window, key: u8) {
    s.try_key(handle, key);
}

#[no_mangle]
fn free_streams(_s: Box<Streams>) {}
