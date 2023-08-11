use fdmux::*;
use va_list::*;
use std::io::Write;
use std::io::Error;

type CStr = *const u8;
type MCStr = *mut u8;

extern "C" {
    fn vsnprintf(buff: MCStr, buff_len: usize, fmt: CStr, ...) -> usize;
}

struct CCall {
    call: extern "C" fn(*const u8, usize)
}

impl CCall {
    pub fn new(call: extern "C" fn(*const u8, usize)) -> Self {
        CCall{call : call}
    }
}

impl Write for CCall {
    fn write(&mut self, buff: &[u8]) -> Result<usize, Error> {
        (self.call)(buff.as_ptr(), buff.len());
        Ok(buff.len())
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl DoCtrlD for CCall {
    fn ctrl_d(&mut self) -> bool {
        false
    }
}

pub struct DestBuff {
    pipe: NamedReadPipe,
    call: CCall
}

impl DestBuff {
    fn destination (&mut self) -> Destination {
        Destination::new(&mut self.call, vec![&mut self.pipe])
    }
}

#[repr(C)]
pub struct Console<'a> {
    cmd: DestBuff,
    msg: DestBuff,
    out: NamedWritePipe,
    top: Topology<'a>
}

impl<'a> Console<'a> {
    fn new(msg: CCall, cmd: CCall) -> Box<Self> {
        let mut ans = Box::new(Console {
            msg: DestBuff { pipe: NamedReadPipe::new("/tmp/dwm.in".to_string()).unwrap(), call: msg },
            cmd: DestBuff { pipe: NamedReadPipe::new("/tmp/dwm.cmd".to_string()).unwrap(), call: cmd },
            out: NamedWritePipe::new("/tmp/dwm.out".to_string()).unwrap(),
            top: Topology::new(2)
        });

        unsafe {
            let m = &mut ans.msg as *mut DestBuff;
            ans.top.insert((*m).destination());
            let c = &mut ans.cmd as *mut DestBuff;
            ans.top.insert((*c).destination());

            ans
        }
    }
}

type ConsHandler = extern "C" fn(*const u8, usize);

#[no_mangle]
extern "C" fn init_console<'a>(msg: ConsHandler, cmd: ConsHandler) -> Box<Console<'a>> {
    Console::new(CCall::new(msg), CCall::new(cmd))
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
