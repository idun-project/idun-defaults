#[macro_use] extern crate failure;

use std::env;
use std::result;
use std::process;
use std::fs;
use std::str;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use bstr::BString;
use nix::unistd;
use std::path::Path;
use std::io::{Read, Write, ErrorKind, stdout};
use std::os::unix::net::UnixStream;
use mio::net::UnixListener;
use mio::{Events, Interest, Poll, Token};
use clap::{Parser,Subcommand};
mod util;
use util::PetString;

const RESPLISTEN: Token      = Token(65535);
const LUAPORT: &str          = "/tmp/idunmm-lua";

// Supported shell command constants
const EXEC_CMD: u8      = 0;
const GO_CMD: u8        = 1;
const LOAD_CMD: u8      = 2;
const DIR_CMD: u8       = 3;
const CATALOG_CMD: u8   = 4;
const DRIVES_CMD: u8    = 5;
const MOUNT_CMD: u8     = 6;
const ASSIGN_CMD: u8    = 7;

#[derive(Parser)]
#[command(version, about, long_about=None, arg_required_else_help=true)]
struct Cli {
    #[command(subcommand)]
    syscmd: Option<Syscommands>,
    #[arg(short)]
    /// Synchronize idun shell current directory with linux
    syncdir: bool,
    #[arg(short)]
    /// Redirect program output to terminal
    output: bool,
    #[arg(short, long, value_name="flags")]
    /// Add flag arguments to the command
    xarg: Option<String>,
    // TODO: Run idunsh in interactive mode
    // #[arg(short)]
    // interactive: bool,
}

#[derive(Subcommand)]
enum Syscommands {
    /// Launch an application on the Commodore
    Go { app:String},
    /// Launch a native program on the Commodore
    Load { prg:String },
    /// Execute remote idun command/program with arguments
    Exec { cmd:String, args: Vec<String> },
    /// Get file list from Idun device using short format
    Dir { dev:String },
    /// Get file list from Idun device using long format
    Catalog { dev:String },
    /// Show list of the active virtual drives and mounts
    Drives { dev:Option<String> },
    /// Mount a virtual floppy image
    Mount { dev:String, dimage:String },
    /// Assign local path to a virtual drive
    Assign { dev:String, path:String },
    /// Fully reboot the idun cartridge and Commodore
    Reboot,
    /// Stop a running program (sends "STOP" key)
    Stop,
}

// Simpler error handling
type Result<T> = result::Result<T, failure::Error>;

fn luasend(message: String) -> Result<()> {
    let mut s = UnixStream::connect(LUAPORT)?;
    let mut r: Vec<u8> = Vec::new();

    s.write_all(message.as_bytes())?;
    s.write(&['\n' as u8])?;
    s.read_to_end(&mut r)?;
    if r.len()>0 && r[0]>0 {
        let emsg = str::from_utf8(&r[1..])?;
        eprintln!("Remote sys.shell() fail: {}", emsg);
    }
    Ok(())
}

fn shell(cmd: u8, args: &String, proc: u32) -> Result<()> {
    let cmd = format!("sys.shell({}, \"{}\", {})", cmd, args, proc);
    luasend(cmd)
}

fn stop_cmd() -> Result<()> {
    let cmd = String::from(r#"sys.stop()"#);
    luasend(cmd)
}

fn reboot_cmd(mode: u8) -> Result<()> {
    let cmd = format!("sys.reboot({})", mode);
    luasend(cmd)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut xargs = String::new();

    // 'cd' commands as needed
    if cli.syncdir {
        let path = env::current_dir().unwrap();
        let cmd = format!("sys.chdir(\"{}\")", path.to_string_lossy());

        luasend(cmd)?;
        // TESTING - pause here to allow first NMI to complete
        thread::sleep(Duration::from_millis(250));
    }
    if let Some(flags)=cli.xarg {
        // Create a switch style flag for each of the characters in xarg.
        for c in flags.chars() {
            xargs.push('/');
            xargs.push(c);
            xargs.push(' ');
        }
    }
    // If output is redirected, create a thread to handle this...
    let mut ojoin: Option<JoinHandle<Result<()>>> = None;
    if cli.output {
        // Create listening socket for response
        let mut poll = Poll::new()?;
        let mut events = Events::with_capacity(128);
        let respath = format!("/run/user/{}/{}", unistd::getuid(), process::id());
        let resport = match UnixListener::bind(Path::new(&respath)) {
            Ok(mut s) => {
                poll.registry().register(&mut s, RESPLISTEN, Interest::READABLE)?;
                s
            },
            Err(e) => bail!("Redirect socket create failed, error: {}", e)
        };
        ojoin = Some(thread::spawn(move || -> Result<()> {
'outer: loop {
            // Wait on response
            poll.poll(&mut events, None)?;

            // Recv responses on socket
            if let Some(event) = events.iter().next() {
                match event.token() {
                    RESPLISTEN => {
                        match resport.accept() {
                            Ok((mut s, _)) => {
                                loop {
                                    let mut buf = [0u8; 80];
                                    match s.read(&mut buf) {
                                        Ok(len) if len>0 => {
                                            let pet = PetString::new(&BString::new(buf[..len].to_vec()));
                                            let pets = String::from(pet);
                                            let pets = pets.replace('\r', "\n");
                                            print!("{}", pets);
                                        },
                                        Ok(_) => break 'outer,
                                        Err(e) if e.kind()==ErrorKind::WouldBlock => continue,
                                        Err(e) if e.kind()==ErrorKind::BrokenPipe => break 'outer,
                                        Err(e) => { return Err(e.into()) }
                                    };
                                }
                            },
                            Err(e) => { return Err(e.into()) }
                        };
                    },
                    Token(tok) => bail!("Mio token error: {}", tok)
                }
            };
        }
        // Cleanup
        println!();
        stdout().flush()?;
        fs::remove_file(&respath)?;
        Ok(())
        }));
    }

    // Assign `proc` variable if output needs to be redirected to this process.
    let proc = if ojoin.is_some() {process::id()} else {0};

    // Handle commands
    match &cli.syscmd {
        Some(Syscommands::Go { app }) => return shell(GO_CMD, app, 0),
        Some(Syscommands::Load { prg }) => return shell(LOAD_CMD, prg, 0),
        Some(Syscommands::Reboot) => return reboot_cmd(0),
        Some(Syscommands::Stop)   => return stop_cmd(),
        Some(Syscommands::Dir { dev }) => shell(DIR_CMD, dev, proc)?,
        Some(Syscommands::Catalog { dev }) => {
            let argstr = format!("{}{}", xargs, dev);
            shell(CATALOG_CMD, &argstr, proc)?
        },
        Some(Syscommands::Drives { dev}) => {
            let argstr = dev.clone().unwrap_or_default();
            shell(DRIVES_CMD, 
            &argstr,
            proc)?
        },
        Some(Syscommands::Mount { dev, dimage }) => {
            let mut argstr = String::from(dev);
            argstr.push(' ');
            argstr.push_str(dimage);
            shell(MOUNT_CMD, &argstr, proc)?
        }
        Some(Syscommands::Assign { dev, path }) => {
            let mut argstr = String::from(dev);
            argstr.push(' ');
            argstr.push_str(path);
            shell(ASSIGN_CMD, &argstr, proc)?
        }
        Some(Syscommands::Exec { cmd, args}) =>
        {
            let argstr = args.join(" ");
            let mut exe = cmd.to_owned();

            exe.push(' ');
            exe.push_str(&xargs);
            exe.push_str(&argstr);
            shell(EXEC_CMD, &exe, proc)?
        },
        None => return Ok(())
    }
    
    // Rejoin thread
    match ojoin {
        Some(oj) => {
            match oj.join() {
                Ok(_) => Ok(()),
                Err(_) => bail!("Failed receiving redirected output")
            }
        },
        None => Ok(())
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
