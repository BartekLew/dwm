use fdmux::*;

use crate::dwm::*;
use crate::stream::*;

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

type ReplHandler = for <'a> fn(args: Vec<&'a str>);
struct Repl<'a> {
    input: std::io::Stdin,
    handlers: HashMap<&'a str, ReplHandler>
}

impl<'a> Repl<'a> {
    fn new(handlers: HashMap<&'a str, ReplHandler>) -> Self {
        Repl::prompt();
        Repl { input: std::io::stdin(), handlers }
    }

    fn prompt() {
        std::io::stdout().write("> ".as_bytes()).unwrap();
        std::io::stdout().flush().unwrap();
    }
}

impl<'a> Write for Repl<'a> {
    fn write<'b>(&mut self, buff: &'b [u8]) -> Result<usize, Error> {
        let len = buff.len();
        if len <= 1 { 
            Repl::prompt();
            return Ok(1);
        }

        let args : Vec<&'b str> =
                   buff.split(|c| char::from(*c).is_whitespace())
                       .filter(|s| s.len() > 0)
                       .map(|s| std::str::from_utf8(s).unwrap())
                       .collect();

        if self.handlers.contains_key(args[0]) {
            self.handlers[args[0]](args[1..].to_vec());
        } else {
            println!("Unknown command: {}", args[0]);
        }

        Repl::prompt();
        Ok(buff.len())
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a> DoCtrlD for Repl<'a> {
    fn ctrl_d(&mut self) -> bool {
        false
    }
}

#[repr(C)]
struct WMCtx<'a, T: Write> {
    cmdout: T,
    ev_streams: &'a mut Streams
}

type DwmHandler<T> = fn(&[u8], &mut WMCtx<T>);
struct DwmCommand<'a, T: Write> {
    ctx: WMCtx<'a,T>,
    handlers: HashMap<u8, DwmHandler<T>>
}

impl<'a, T:Write> Write for DwmCommand<'a, T> {
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

impl<'a, T: Write> DoCtrlD for DwmCommand<'a, T> {
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
    cmd: DestBuff<DwmCommand<'a, NamedWritePipe>>,
    msg: DestBuff<BarMessager>,
    repl: Repl<'a>,
    top: Topology<'a>
}

impl<'a> Console<'a> {
    fn new(streams: &'a mut Streams) -> Box<Self> {
        let mut ans = Box::new(Console {
            msg: DestBuff { pipe: NamedReadPipe::new("/tmp/dwm.in".to_string()).unwrap(),
                            call: BarMessager{} },
            repl: Repl::new(HashMap::from([
                    ("ls", repl_ls as ReplHandler),
                    ("show", repl_show as ReplHandler)
                ])),
            cmd: DestBuff { pipe: NamedReadPipe::new("/tmp/dwm.cmd".to_string()).unwrap(),
                            call: DwmCommand{
                                ctx: WMCtx {
                                    cmdout: NamedWritePipe::new("/tmp/dwm.out".to_string()).unwrap(),
                                    ev_streams: streams
                                },
                                handlers: HashMap::from([
                                    (b'l', ccmd_ls as DwmHandler<NamedWritePipe>),
                                    (b'<', ccmd_focus_last as DwmHandler<NamedWritePipe>),
                                    (b'f', ccmd_fullscreen as DwmHandler<NamedWritePipe>),
                                    (b't', ccmd_trace_on as DwmHandler<NamedWritePipe>),
                                    (b'T', ccmd_trace_off as DwmHandler<NamedWritePipe>),
                                    (b'g', ccmd_grab_ev as DwmHandler<NamedWritePipe>)
                                ])
                            } },
            top: Topology::new(3)
        });

        unsafe {
            let m = &mut ans.msg as *mut DestBuff<BarMessager>;
            ans.top.insert((*m).destination());
            let c = &mut ans.cmd as *mut DestBuff<DwmCommand<NamedWritePipe>>;
            ans.top.insert((*c).destination());
            let r = &mut ans.repl as *mut Repl;
            ans.top.insert(Destination::new(&mut *r, vec![&mut (*r).input]));

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

fn ccmd_ls<T:Write>(_args: &[u8], ctx: &mut WMCtx<T>) {
    let mut monn = 0;
    for mon in Monitors::all() {
        for client in Clients::all(&mon) {
            ctx.cmdout.write(&format!("{}: {}\n\0", monn, ptr2str(client.name.as_ptr() as CStr)).as_bytes())
                   .unwrap();
        }

        monn += 1;
    }
}

fn repl_ls(_args: Vec<&str>) {
    Monitors::all()
             .for_each(|mon| Clients::all(mon)
                                     .for_each(|win| println!("{:#x} : {}",
                                                        win.win, str::from_utf8(&win.name).unwrap())));
}

fn for_client_args<F: FnMut(&Client)>(args: Vec<&str>, mut act: F) {
    let wins = args.iter().flat_map(|arg| match arg.parse::<u64>() {
                                            Ok(w) => vec![w],
                                            Err(_) => vec![]
                                        })
                          .collect::<Vec<Window>>();
    Monitors::all()
             .for_each(|mon| Clients::all(mon)
                                     .filter(|win| wins.contains(&win.win))
                                     .for_each(|c| act(c)));
}

fn repl_show(args: Vec<&str>) {
    let wins = args.iter().flat_map(|arg| match arg.parse::<u64>() {
                                            Ok(w) => vec![w],
                                            Err(_) => vec![]
                                        })
                          .collect::<Vec<Window>>();
    Monitors::all()
             .for_each(|mon| Clients::all(mon)
                                     .filter(|win| wins.contains(&win.win))
                                     .for_each(|win| println!("{}", win)));
}

fn repl_trace(streams: &mut Streams, args: Vec<&str>) {
    for_client_args(args, |client| streams.add_trace(client));
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

fn ccmd_focus_last<T: Write> (_args: &[u8], _ctx: &mut WMCtx<T>) {
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

fn ccmd_fullscreen<T: Write> (_pars: &[u8], _ctx: &mut WMCtx<T>) {
    unsafe {
        setlayout(&mut layouts.offset(2) as *mut *mut Layout);
    }
}

fn ccmd_trace_on<T: Write> (_pars: &[u8], _ctx: &mut WMCtx<T>) {
    unsafe{ trace_p = 1; }
}

fn ccmd_trace_off<T: Write> (_pars: &[u8], _ctx: &mut WMCtx<T>) {
    unsafe{ trace_p = 0; }
}

fn ccmd_grab_ev<T: Write> (pars: &[u8], ctx: &mut WMCtx<T>) {
    let s = String::from(str::from_utf8(&pars[0..pars.len()-1]).unwrap());
    ctx.ev_streams.add_grab(s);
}

