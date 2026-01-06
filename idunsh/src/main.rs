#[macro_use] extern crate failure;

use std::collections::HashMap;
use std::env;
use std::result;
use std::process;
use std::fs;
use std::str;
use std::thread;
use std::io;
use std::time::Duration;
use bstr::BString;
use nix::unistd;
use std::path::Path;
use std::io::{Read, Write, stdout};
use std::os::unix::net::{UnixListener, UnixStream};
use clap::{Parser,Subcommand};
use ureq;
use serde;
use serde::Deserialize;
mod util;
use util::PetString;

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
    #[arg(short)]
    /// Use the C64 Ultimate runner to load content
    ultimate: bool,
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

/// Types used for deserializing the C64 Ultimate Drives
#[allow(dead_code)]
#[derive(Deserialize)]
struct Device {
    enabled: bool,
    bus_id: u8,
    #[serde(rename = "type")]
    device_type: Option<String>, // not all devices have it
    rom: Option<String>,
    image_file: Option<String>,
    image_path: Option<String>,
}
#[derive(Deserialize)]
struct DriveEntry {
    #[serde(flatten)]
    devices: HashMap<String, Device>,
}
#[derive(Deserialize)]
struct UltiDrives {
    drives: Vec<DriveEntry>,
}

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

fn post(ip: &String, url: &String, file: &String) -> io::Result<()> {
    let path = Path::new(file);
    let mut buf: Vec<u8> = vec![];
    fs::File::open(path)?.read_to_end(&mut buf)?;
    
    let mut req = String::from("http://");
    req.push_str(ip);
    req.push_str(url);

    ureq::post(req)
        .send(buf)
        .map(|_| ())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}

fn meta(filename: &str) -> io::Result<(u64, u16)> {
    let path = Path::new(filename);

    // Open file
    let mut file = fs::File::open(path)?;

    // Get file size
    let size = file.metadata()?.len();

    // Read first 2 bytes
    let mut buf = [0u8; 2];
    file.read_exact(&mut buf)?;

    // Convert to little-endian u16
    let addr = u16::from_le_bytes(buf);

    Ok((size, addr))
}

fn ultiload(ip: &String, filenm: &String) -> Result<()> {
    let url: Option<String>;
    let lcase = filenm.to_lowercase();
    let ext = Path::new(&lcase)
                            .extension()
                            .and_then(|s| s.to_str());
    let (size, start) = meta(filenm)?;

    if ext == None {
        if (size + (start as u64)) < 65536 {
            url = Some(String::from("/v1/runners:run_prg"));
        } else {
            bail!("PRG file is too large")
        }
    } else {
        url = match ext.unwrap() {
            "crt" => Some(String::from("/v1/runners:run_crt")),
            "sid" => Some(String::from("/v1/runners:sidplay")),
            "mod" => Some(String::from("/v1/runners:modplay")),
            "prg" => if (size + (start as u64)) < 65536 {
                Some(String::from("/v1/runners:run_prg"))
            } else {
                bail!("PRG file is too large")
            }
            _ => None,
        };
    }
    
    if let Some(u) = url {
        match post(ip, &u, filenm) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Error: {}", e);
                bail!("C64 Ultimate web request fail: {}{}", ip, u)
            }
        }
    } else {
        bail!("File extension not recognized")
    }
}

fn ultimount(ip: &String, device: &String, dimage: &String) -> Result<()> {
    let lcase = dimage.to_lowercase();
    let ext = Path::new(&lcase)
                            .extension()
                            .and_then(|s| s.to_str());

    // Disk image name must have a recognized file extension
    if ext.is_none() { bail!("Unrecognized disk image file type/") }
    let url = match ext.unwrap() {
        "d64" | "g64" | "d71" | "g71" | "d81" => {
            format!("/v1/drives/{}mount?type={}", device, ext.unwrap())
        },
        _ => bail!("Unrecognized disk image file type/")
    };

    match post(ip, &url, dimage) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("Error: {}", e);
            bail!("C64 Ultimate web request fail: {}{}", ip, &url)
        }
    }
}

fn ultidrv(ip: &String, _device: &Option<String>) -> io::Result<UltiDrives> {
    let url = format!("http://{}/v1/drives", ip);
    let mut resp = ureq::get(&url)
        .call()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    resp.body_mut()
        .read_json::<UltiDrives>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut xargs = String::new();

    // Check for C64-Ultimate commands first, since they circumvent additional processing below.
    if cli.ultimate {
        // Check that we have access to the web services
        if let Ok(ip) = std::env::var("C64_ULTIMATE_IP") {
            match &cli.syscmd {
                Some(Syscommands::Load { prg }) =>
                    return ultiload(&ip, prg),
                Some(Syscommands::Mount { dev, dimage }) =>
                    return ultimount(&ip, dev, dimage),
                Some(Syscommands::Drives { dev }) => {
                    match ultidrv(&ip, dev) {
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
        } else {
            bail!("C64 Ultimate loads require $C64_ULTIMATE_IP set!")
        }
    }

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
            shell(DRIVES_CMD, &argstr, proc)?
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
