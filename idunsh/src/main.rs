// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Brian Holdsworth
#[macro_use] extern crate failure;

use std::env;
use std::result;
use std::process;
use std::fs;
use std::str;
use std::thread;
use std::time::Duration;
use bstr::BString;
use nix::unistd;
use std::path::Path;
use std::io::{Read, Write, stdout};
use std::os::unix::net::{UnixListener, UnixStream};
use clap::{Parser,Subcommand,ArgGroup};
use shell_words::split;
mod util;
use util::PetString;
mod c64ultimate;
use c64ultimate::C64Ultimate;

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
#[command(version, about, long_about=None, arg_required_else_help=true,
    group(
        ArgGroup::new("command").required(true).args(&["cmd", "rest"])
    )
)]
struct Cli {
    #[arg(short)]
    /// Synchronize idun shell current directory with linux
    syncdir: bool,
    #[arg(short)]
    /// Redirect program output to terminal
    output: bool,
    #[arg(short)]
    /// Use the C64 Ultimate runner to load content
    ultimate: bool,
    #[arg(short, long, value_name="flags")]
    /// Add flag arguments to the command
    xarg: Option<String>,
    #[arg(short, long, value_name="cmdline")]
    /// Pass sub-command as a single argument (for shell wrappers)
    cmd: Option<String>,
    #[arg(trailing_var_arg=true, value_name="COMMAND", help="Subcommand with arguments")]
    /// Pass sub-command as additional args (for normal CLI usage)
    rest: Vec<String>,
    // TODO: Run idunsh in interactive mode
    // #[arg(short)]
    // interactive: bool,
}

#[derive(Parser)]
struct Syscommand {
    #[command(subcommand)]
    cmd: Syscommands,
}

#[derive(Subcommand)]
enum Syscommands {
    /// Launch an application on the Commodore
    Go { app:String},
    /// Launch a native program on the Commodore
    Load { prg:String },
    /// Launch content on the C64 Ultimate
    Run { prg:String },
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
fn parse_sys_command(cli: &Cli) -> Syscommand {
    let mut argv = vec!["idunsh".to_string()];

    if let Some(cmdline) = &cli.cmd {
        argv.extend(
            split(&cmdline).unwrap_or_else(|e| {
                eprintln!("Invalid --cmd syntax: {e}");
                std::process::exit(2);
            }),
        );
    } else {
        argv.extend(cli.rest.clone());
    }

    Syscommand::parse_from(argv)
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

    // Extract the sub-command
    let syscmd = parse_sys_command(&cli);

    // Check for C64-Ultimate commands first, since they circumvent chrir and redirect processing
    if cli.ultimate || matches!(syscmd.cmd, Syscommands::Run{..}) {
        // Check that we have access to the C64 Ultimate web service
        let c64u = C64Ultimate::new();
        if c64u.ip().is_none() {
            bail!("C64 Ultimate loads require $C64_ULTIMATE_IP set!")
        }

        match syscmd.cmd {
            Syscommands::Load { prg } |
            Syscommands::Run  { prg } =>
                return c64u.load(&prg),
            Syscommands::Mount { dev, dimage } =>
                return c64u.mount(&dev, &dimage),
            Syscommands::Drives { dev } => {
                match c64u.getdrv(&dev) {
                    Ok(ultid) => {
                        for entry in ultid.drives {
                            let (drive, settings) = entry.devices.into_iter().next().unwrap();
                            if drive.len()==1 {     // Just listing a:, b:
                                if settings.enabled {
                                    println!("{}", format!("{}:={}", drive, settings.image_file.unwrap()));
                                } else {
                                    println!("{}", format!("{}:={}", drive, "<Disabled>"));
                                }
                            }
                        }
                        return Ok(())
                    }
                    Err(e) => bail!("C64 Ultimate drive settings Error: {}", e)
                }
                // Idun virtual drives handled below...
            },
            _ => bail!("Command not supported for the C64 Ultimate")
        }
    }

    // 'cd' commands as needed
    if cli.syncdir {
        let path = env::current_dir().unwrap();
        let cmd = format!("sys.chdir(\"{}\")", path.to_string_lossy());

        luasend(cmd)?;
        // TESTING - pause here to allow first NMI to complete
        thread::sleep(Duration::from_millis(500));
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
    let ojoin = match cli.output {
        true => {
            // Create listening socket for response
            let respath = format!("/run/user/{}/{}", unistd::getuid(), process::id());
            let resport = UnixListener::bind(Path::new(&respath))?;
            Some(thread::spawn(move || -> Result<()> {
                // Wait on response
                match resport.accept()? {
                    (mut s, _) => {
                        let mut buf = [0u8; 4096];
                        loop {
                            match s.read(&mut buf)? {
                                0 => break,
                                n => {
                                    let pet = PetString::new(&BString::new(buf[..n].to_vec()));
                                    let pets = String::from(pet).replace('\r', "\n");
                                    print!("{}", pets);
                                },
                            }
                        }
                    }
                }
                // Cleanup
                println!();
                stdout().flush()?;
                fs::remove_file(&respath)?;
                Ok(())
            }))
        },
        false => None
    };

    // Assign `proc` variable if output needs to be redirected to this process.
    let proc = if ojoin.is_some() {process::id()} else {0};

    // Handle commands
    match syscmd.cmd {
        Syscommands::Go { app } => return shell(GO_CMD, &app, 0),
        Syscommands::Load { prg } => return shell(LOAD_CMD, &prg, 0),
        Syscommands::Reboot => return reboot_cmd(0),
        Syscommands::Stop   => return stop_cmd(),
        Syscommands::Dir { dev } => shell(DIR_CMD, &dev, proc)?,
        Syscommands::Catalog { dev } => {
            let argstr = format!("{}{}", xargs, dev);
            shell(CATALOG_CMD, &argstr, proc)?
        },
        Syscommands::Drives { dev} => {
            let argstr = dev.clone().unwrap_or_default();
            shell(DRIVES_CMD, &argstr, proc)?
        },
        Syscommands::Mount { dev, dimage } => {
            let mut argstr = String::from(dev);
            argstr.push(' ');
            argstr.push_str(&dimage);
            shell(MOUNT_CMD, &argstr, proc)?
        }
        Syscommands::Assign { dev, path } => {
            let mut argstr = String::from(dev);
            argstr.push(' ');
            argstr.push_str(&path);
            shell(ASSIGN_CMD, &argstr, proc)?
        }
        Syscommands::Exec { cmd, args} =>
        {
            let argstr = args.join(" ");
            let mut exe = cmd.to_owned();

            exe.push(' ');
            exe.push_str(&xargs);
            exe.push_str(&argstr);
            shell(EXEC_CMD, &exe, proc)?
        },
        Syscommands::Run { .. } => return Ok(()),   //not used, handled above
    }
    
    // Rejoin thread
    match ojoin {
        Some(oj) => {
            match oj.join() {
                Ok(_) => Ok(()),
                Err(e) => bail!("Failed receiving redirected output E:{:?}", e)
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
