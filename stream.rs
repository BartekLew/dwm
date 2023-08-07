
use std::str;
use std::slice;

#[repr(C)]
struct Window {}

extern "C" {
    fn strlen(cstr: *const u8) -> usize;
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

    fn try_window(&mut self, handle: *const Window, name: String) -> bool {
        if self.handle.is_none() && prefix_eq(&self.name, &name) {
            self.handle = Some(handle);
            true
        } else {
            false
        }
    }

    fn try_key(&mut self, handle: *const Window, key: u8) {
        self.handle.map(|my_handle| {
            if handle == my_handle {
                println!("hit {} for {:?}", str::from_utf8(&[key]).unwrap(), handle);
            }
        });
    }
}

fn ptr2str(ptr: *const u8) -> String {
    unsafe {
        String::from(str::from_utf8(slice::from_raw_parts(ptr, strlen(ptr))).unwrap())
    }
}

#[no_mangle]
extern "C"
fn init_stream(name: *const u8) -> Box<Stream> {
    Box::new(Stream::new(String::from(ptr2str(name))))
}

#[no_mangle]
extern "C"
fn win2stream(s: &mut Stream, handle: *const Window, name: *const u8) -> bool {
    s.try_window(handle, ptr2str(name))
}

#[no_mangle]
fn key2stream(s: &mut Stream, handle: *const Window, key: u8) {
    s.try_key(handle, key);
}

#[no_mangle]
fn free_streams(_s: Box<Stream>) {}
