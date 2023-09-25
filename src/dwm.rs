use std::ffi::c_void;
use std::str;
use std::slice;
use std::fmt;
use std::io::Write;

pub type Window = u64;
pub type KeySym = u64;
pub type Ptr = *const c_void;
pub type CStr = *const u8;

pub const ANY_KEY : i32 = 0;
pub const ANY_MODIFIER : u32 = 1 << 15;
pub const GRAB_MODE_ASYNC : i32 = 1;

extern "C" {
    // libc:
    pub fn strlen(cstr: CStr) -> usize;

    // xlib:
    pub fn XGrabKey(dpy: Ptr, key: i32, mods: u32, tgt: Window, owner_events: bool,
                ptr_mode: i32, key_mode: i32) -> i32;

    pub fn XMoveWindow(dpy: Ptr, win: Window, x: i32, y: i32);

    // dwm:
    pub static mut mons: *mut Monitor;
    pub static mut lastc: *mut Client;
    pub static mut trace_p: i32;
    pub static mut layouts: *mut Layout;
    pub static dpy : Ptr;
    pub static selmon : *mut Monitor;
    
    pub fn focus(c: *mut Client);
    pub fn setlayout(l: *mut *mut Layout);
    pub fn resize(c: *mut Client, x: i32, y:i32, w: i32, h:i32, interact: i32);
    pub fn arrange(m: *mut Monitor);
}

#[repr(C)]
pub struct CLenStr {
    buff: CStr,
    len: usize
}

impl CLenStr {
    pub fn new(buff: &[u8]) -> Self {
        CLenStr { buff: buff.as_ptr(), len: buff.len() }
    }

    pub fn null() -> Self {
        CLenStr { buff: 0 as CStr, len: 0 }
    }

    pub fn as_slice_ref<'a>(self) -> &'a [u8] {
        unsafe { slice::from_raw_parts(self.buff, self.len) }
    }

    pub fn as_str<'a>(self) -> &'a str {
        match std::str::from_utf8(self.as_slice_ref()) {
            Ok(str) => str,
            Err(_) => "???"
        }
    }
}

#[repr(C)]
pub struct Client {
	pub name: [u8;256],
	_mina: f32, _maxa: f32,
	x: i32, y: i32, w: i32, h: i32,
	_oldx: i32, _oldy: i32, _oldw: i32, _oldh: i32,
	_basew: i32, _baseh: i32, _incw: i32, _inch: i32,
    _maxw: i32, _maxh: i32, minw: i32, minh: i32,
	_bw: i32, _oldbw: i32,
	pub tags: u32,
	_isfixed: i32, isfloating: i32, _isurgent: i32, 
    _neverfocus: i32, _oldstate: i32, isfullscreen: i32,
	next: *mut Client,
	snext: *mut Client,
	mon: *const Monitor,
	pub win: Window
}

impl Client {
    fn visible(&self) -> bool {
        unsafe {
            self.tags & (*self.mon).tags != 0
        }
    }

    fn apply_possize(&mut self) {
        unsafe {
            if self.visible() {
                XMoveWindow(dpy, self.win, self.x, self.y);
                if ((*self.mon).lt[(*self.mon).sellt as usize].arrange as usize == 0
                    || self.isfloating != 0) && self.isfullscreen == 0 {
                        resize(self, self.x, self.y, self.w, self.h, 0);
                }
            } else {
                XMoveWindow(dpy, self.win, -2*self.w, 0);
            }
        }
    }

    fn from_ptr<'a>(c: *mut Client) -> Option<&'a mut Client> {
        match c as usize {
            0 => None,
            _ => Some(unsafe { &mut *c })
        }
    }

    fn null() -> *mut Client {
        0 as *mut Client
    }
}

impl fmt::Display for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}: {}x{}@{}:{}({})\n  {}",
               self.win, self.w, self.h, self.x, self.y, self.tags,
               str::from_utf8(&self.name).unwrap())
    }
}

#[repr(C)]
pub struct Monitor {
    _ltsym: [u8;16],
	_mfact: f32,
	_nmaster: i32,
	_num: i32,
	_by: i32,               /* bar geometry */
	_mx: i32, _my: i32, _mw: i32, _mh: i32,   /* screen size */
	_wx: i32, _wy: i32, _ww:i32, _wh: i32,   /* window area  */
	tags: u32,
	sellt: u32,
	_showbar: i32,
	_topbar: i32,
	clients: *mut Client,
	_sel: *const Client,
	_stack: *const Client,
	next: *mut Monitor,
	_barwin: Window,
	lt: [Layout; 2]
}

impl Monitor {
    pub fn from_ptr<'a>(ptr: *mut Monitor) -> Option<&'a mut Monitor> {
        unsafe {
            match ptr as usize {
                0 => None,
                _ => Some(&mut *ptr)
            }
        }
    }

    pub fn view_tags(&mut self, tags: u32) {
        unsafe {
            if self.tags != tags {
                self.tags = tags;
                focus(Client::null());
                arrange(self);
            }
        }
    }
}

#[repr(C)]
pub struct Layout {
    _name: *const u8,
    arrange: extern "C" fn(*mut Monitor)
}

pub struct Monitors<'a> {
    cur: Option<&'a Monitor>
}

impl<'a> Monitors<'a> {
    pub fn new(val: *mut Monitor) -> Self { Monitors { cur: unsafe { val.as_ref() } } }
    pub fn all() -> Self { unsafe { Self::new(mons) } }
}

impl <'a> Iterator for Monitors <'a> {
    type Item = &'a Monitor;

    fn next(&mut self) -> Option<Self::Item> {
        match self.cur {
            Some(val) => {
                self.cur = unsafe { val.next.as_ref() };

                Some(val)
            },
            None => None
        }
    }
}

pub struct Clients<'a> {
    cur: Option<&'a Client>
}

impl<'a> Clients<'a> {
    pub fn new(val: *mut Client) -> Self { Clients { cur: unsafe {val.as_ref()} } }
    pub fn all(mon: &Monitor) -> Self { 
        Self::new(mon.clients)
    }
}

impl <'a> Iterator for Clients<'a>{
    type Item = &'a Client;

    fn next(&mut self) -> Option<Self::Item> {
        match self.cur {
            Some(val) => {
                self.cur = unsafe {val.next.as_ref()};

                Some(val)
            },
            None => None
        }
    }
}

pub fn ptr2str(ptr: CStr) -> String {
    unsafe {
        String::from(str::from_utf8(slice::from_raw_parts(ptr, strlen(ptr))).unwrap())
    }
}

#[no_mangle]
extern "C" fn set_term_title(title: CLenStr) {
    let mut o = std::io::stdout();
    o.write(format!("\x1b]0;{}\x07", title.as_str()).as_bytes()).unwrap();
    o.flush().unwrap();
}

#[no_mangle]
extern "C" fn showhide(cptr: *mut Client) {
    match Client::from_ptr(cptr) {
        Some(c) => {
            c.apply_possize();
            showhide(c.snext);
        }, None => ()
    }
}

#[no_mangle]
pub extern "C" fn view(tags: &u32) {
    unsafe {
        Monitor::from_ptr(selmon)
                .map(|m| m.view_tags(*tags));
    }
}

