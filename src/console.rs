use fdmux::*;

use crate::dwm::*;
use crate::stream::Streams;

use std::collections::HashMap;
use std::io::Write;
use std::io::Error;
use std::str;

type CStr = *const u8;
type MCStr = *mut u8;

extern "C" {
    fn got_msg(buff: MCStr, buff_len: usize);
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

#[repr(C)]
struct WMCtx<'a> {
    cmdout: NamedWritePipe,
    ev_streams: &'a mut Streams
}

type DwmHandler = fn(&[u8], &mut WMCtx);
struct DwmCommand<'a> {
    ctx: WMCtx<'a>,
    handlers: HashMap<u8, DwmHandler>
}

impl<'a> Write for DwmCommand<'a> {
    fn write(&mut self, buff: &[u8]) -> Result<usize, Error> {
        let len = buff.len();
        if len == 0 { return Ok(0); }

        let cmd = buff[0];
        match self.handlers.contains_key(&cmd) {
            true => {
                (self.handlers[&cmd])(&buff[1..len], &mut self.ctx);
                Ok(len)
            },
            false => { Ok(0) }
        }
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a> DoCtrlD for DwmCommand<'a> {
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
    cmd: DestBuff<DwmCommand<'a>>,
    msg: DestBuff<BarMessager>,
    top: Topology<'a>
}

impl<'a> Console<'a> {
    fn new(streams: &'a mut Streams) -> Box<Self> {
        let mut ans = Box::new(Console {
            msg: DestBuff { pipe: NamedReadPipe::new("/tmp/dwm.in".to_string()).unwrap(),
                            call: BarMessager{} },
            cmd: DestBuff { pipe: NamedReadPipe::new("/tmp/dwm.cmd".to_string()).unwrap(),
                            call: DwmCommand{
                                ctx: WMCtx {
                                    cmdout: NamedWritePipe::new("/tmp/dwm.out".to_string()).unwrap(),
                                    ev_streams: streams
                                },
                                handlers: HashMap::from([
                                    (b'l', ccmd_ls as DwmHandler),
                                    (b'<', ccmd_focus_last as DwmHandler),
                                    (b'f', ccmd_fullscreen as DwmHandler),
                                    (b't', ccmd_trace_on as DwmHandler),
                                    (b'T', ccmd_trace_off as DwmHandler),
                                    (b'g', ccmd_grab_ev as DwmHandler)
                                ])
                            } },
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
extern "C" fn init_console<'a>(streams: &'a mut Streams) -> Box<Console<'a>> {
    Console::new(streams)
}

#[no_mangle]
extern "C" fn console_job(cons: &mut Console) {
    cons.top.tick(0);
}

#[no_mangle]
extern "C" fn close_console(_: Box<Console>) {}

fn ccmd_ls(_args: &[u8], ctx: &mut WMCtx) {
    let mut monn = 0;
    for mon in Monitors::all() {
        for client in Clients::all(&mon) {
            ctx.cmdout.write(&format!("{}: {}\n\0", monn, ptr2str(client.name.as_ptr() as CStr)).as_bytes())
                   .unwrap();
        }

        monn += 1;
    }
}

#[no_mangle]
extern "C" fn console_out(cons: &mut Console, buff: CLenStr) {
    cons.cmd.call.ctx.cmdout.write(buff.as_slice_ref())
                            .unwrap();
}

#[no_mangle]
extern "C" fn console_log_del(cons: &mut Console, name: CStr, wid: Window) {
    cons.cmd.call.ctx.cmdout.write(format!("Deleted: {}({})\n\0", ptr2str(name), wid).as_bytes())
                            .unwrap();
}

#[no_mangle]
extern "C" fn console_log_upd(cons: &mut Console, name: CStr, wid: Window) {
    cons.cmd.call.ctx.cmdout.write(format!("Updated: {}({})\n\0", ptr2str(name), wid).as_bytes())
                            .unwrap();
}

fn ccmd_focus_last (_args: &[u8], _ctx: &mut WMCtx) {
    unsafe {
        match lastc.as_ref() {
            Some(c) => {
                view(&c.tags);
                focus(lastc);
            },
            None => {}
        }
    }
}

fn ccmd_fullscreen (_pars: &[u8], _ctx: &mut WMCtx) {
    unsafe {
        setlayout(&mut layouts.offset(2) as *mut *mut Layout);
    }
}

fn ccmd_trace_on (_pars: &[u8], _ctx: &mut WMCtx) {
    unsafe{ trace_p = 1; }
}

fn ccmd_trace_off (_pars: &[u8], _ctx: &mut WMCtx) {
    unsafe{ trace_p = 0; }
}

fn ccmd_grab_ev (pars: &[u8], ctx: &mut WMCtx) {
    let s = String::from(str::from_utf8(&pars[0..pars.len()-1]).unwrap());
    ctx.ev_streams.add(s);
}

