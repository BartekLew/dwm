use fdmux::*;
use va_list::*;
use std::collections::HashMap;
use std::io::Write;
use std::io::Error;

type CStr = *const u8;
type MCStr = *mut u8;

extern "C" {
    fn vsnprintf(buff: MCStr, buff_len: usize, fmt: CStr, ...) -> usize;

    fn got_msg(buff: MCStr, buff_len: usize);

    fn ccmd_ls(buff: CStr, buff_len: usize);
    fn ccmd_focus_last(buff: CStr, buff_len: usize);
    fn ccmd_fullscreen(buff: CStr, buff_len: usize);
    fn ccmd_trace_on(buff: CStr, buff_len: usize);
    fn ccmd_trace_off(buff: CStr, buff_len: usize);
    fn ccmd_grab_ev(buff: CStr, buff_len: usize);
}

struct BarMessager {}
impl Write for BarMessager {
    fn write(&mut self, buff: &[u8]) -> Result<usize, Error> {
        unsafe { got_msg(buff.as_ptr() as *mut u8, buff.len()); }
        Ok(buff.len())
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl DoCtrlD for BarMessager {
    fn ctrl_d(&mut self) -> bool {
        false
    }
}

type DwmHandler = unsafe extern "C" fn(CStr, usize);
struct DwmCommand {
    handlers: HashMap<u8, DwmHandler>
}

impl Write for DwmCommand {
    fn write(&mut self, buff: &[u8]) -> Result<usize, Error> {
        let len = buff.len();
        if len == 0 { return Ok(0); }

        let cmd = buff[0];
        match self.handlers.contains_key(&cmd) {
            true => {
                unsafe { (self.handlers[&cmd])(buff[1..len].as_ptr() as *const u8, len-1); }
                Ok(len)
            },
            false => { Ok(0) }
        }
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl DoCtrlD for DwmCommand {
    fn ctrl_d(&mut self) -> bool {
        false
    }
}

pub struct DestBuff<C:DoCtrlD> {
    pipe: NamedReadPipe,
    call: C
}

impl<C:DoCtrlD> DestBuff<C> {
    fn destination (&mut self) -> Destination {
        Destination::new(&mut self.call, vec![&mut self.pipe])
    }
}

#[repr(C)]
pub struct Console<'a> {
    cmd: DestBuff<DwmCommand>,
    msg: DestBuff<BarMessager>,
    out: NamedWritePipe,
    top: Topology<'a>
}

impl<'a> Console<'a> {
    fn new() -> Box<Self> {
        let mut ans = Box::new(Console {
            msg: DestBuff { pipe: NamedReadPipe::new("/tmp/dwm.in".to_string()).unwrap(),
                            call: BarMessager{} },
            cmd: DestBuff { pipe: NamedReadPipe::new("/tmp/dwm.cmd".to_string()).unwrap(),
                            call: DwmCommand{
                                handlers: HashMap::from([
                                    (b'l', ccmd_ls as DwmHandler),
                                    (b'<', ccmd_focus_last as DwmHandler),
                                    (b'f', ccmd_fullscreen as DwmHandler),
                                    (b't', ccmd_trace_on as DwmHandler),
                                    (b'T', ccmd_trace_off as DwmHandler),
                                    (b'g', ccmd_grab_ev as DwmHandler)
                                ])
                            } },
            out: NamedWritePipe::new("/tmp/dwm.out".to_string()).unwrap(),
            top: Topology::new(2)
        });

        unsafe {
            let m = &mut ans.msg as *mut DestBuff<BarMessager>;
            ans.top.insert((*m).destination());
            let c = &mut ans.cmd as *mut DestBuff<DwmCommand>;
            ans.top.insert((*c).destination());

            ans
        }
    }
}

#[no_mangle]
extern "C" fn init_console<'a>() -> Box<Console<'a>> {
    Console::new()
}

#[no_mangle]
extern "C" fn console_job(cons: &mut Console) {
    cons.top.tick(0);
}

#[no_mangle]
extern "C" fn close_console(_: Box<Topology>) {}

#[no_mangle]
extern "C" fn console_log(cons: *mut Console, fmt: *const u8, va: VaList) {
    let buff = [0;256];
    unsafe {
        let n = vsnprintf(buff.as_ptr() as MCStr, 256, fmt, va);
        (*cons).out.write(&buff[0..n]).unwrap();
    }
}
