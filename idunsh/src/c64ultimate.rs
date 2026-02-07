// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Brian Holdsworth
use std::result;
use std::net::{UdpSocket, SocketAddr};
use std::time::Duration;
use std::path::Path;
use std::fs;
use std::io;
use std::io::Read;
use std::collections::HashMap;
use serde;
use serde::Deserialize;
use ureq;

// Simpler error handling
type Result<T> = result::Result<T, failure::Error>;

/// Types used for deserializing the C64 Ultimate Drives
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Device {
    pub enabled: bool,
    pub bus_id: u8,
    #[serde(rename = "type")]
    pub device_type: Option<String>, // not all devices have it
    pub rom: Option<String>,
    pub image_file: Option<String>,
    pub image_path: Option<String>,
}
#[derive(Deserialize)]
pub struct DriveEntry {
    #[serde(flatten)]
    pub devices: HashMap<String, Device>,
}
#[derive(Deserialize)]
pub struct UltiDrives {
    pub drives: Vec<DriveEntry>,
}

/// Access to a C64U on the LAN using its network service API.
/// For this to work, the "Web Remote Control Service" and the
/// "Ident Service" must be enabled in the C64U configuration.
pub struct C64Ultimate {
    service_ip: Option<String>,
}

impl C64Ultimate {
    /// If the "C64_ULTIMATE_IP" env var is set, then it is assumed that
    /// a C64U has been previously detected and available at that IP.
    /// Otherwise, attempt to detect a C64U on the LAN.
    pub fn new() -> Self {
        match std::env::var("C64_ULTIMATE_IP") {
            Ok(v) => C64Ultimate { service_ip: Some(v) },
            Err(_) => if let Some(detect) = Self::detect() {
                C64Ultimate { service_ip: Some(detect) }
            } else {
                C64Ultimate { service_ip: None }
            }
        }
    }
    /// Returns the IP of the C64U as a String, or None if it is
    /// not detected.
    pub fn ip(&self) -> &Option<String> {
        &self.service_ip
    }
    /// Loads content file using network service. Currently supports
    /// PRG, CRT, SID, and MOD files.
    pub fn load(&self, filenm: &String) -> Result<()> {
        let url: Option<String>;
        let lcase = filenm.to_lowercase();
        let ext = Path::new(&lcase)
                                .extension()
                                .and_then(|s| s.to_str());
        let (size, start) = Self::meta(filenm)?;

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
            match self.post(&u, filenm) {
                Ok(_) => Ok(()),
                Err(e) => {
                    println!("Error: {}", e);
                    bail!("C64 Ultimate web request fail: {}", u)
                }
            }
        } else {
            bail!("File extension not recognized")
        }
    }
    /// Mounts disk image file to selected floppy device [a | b]. Supports
    /// most disk image types and the C64U will change the drive type based
    /// on the filename extension.
    pub fn mount(&self, device: &String, dimage: &String) -> Result<()> {
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

        match self.post(&url, dimage) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Error: {}", e);
                bail!("C64 Ultimate web request fail: {}", &url)
            }
        }
    }
    /// Get the vital information about the available IEC devices
    pub fn getdrv(&self, _device: &Option<String>) -> io::Result<UltiDrives> {
        let url = format!("http://{}/v1/drives", self.service_ip.as_ref().unwrap());
        let mut resp = ureq::get(&url)
            .call()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        resp.body_mut()
            .read_json::<UltiDrives>()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }
    /// Detect if there is a C64 Ultimate on the LAN and return its IP address.
    fn detect() -> Option<String> {
        const MESSAGE: &[u8] = b"ping";
        const BROADCAST_ADDR: &str = "255.255.255.255:64";
        const TIMEOUT: Duration = Duration::from_millis(500);

        // Bind to an ephemeral local port
        let socket = UdpSocket::bind("0.0.0.0:0").ok()?;

        // Enable broadcast (best effort)
        let _ = socket.set_broadcast(true);

        // Set receive timeout
        socket.set_read_timeout(Some(TIMEOUT)).ok()?;

        // Send discovery packet
        socket.send_to(MESSAGE, BROADCAST_ADDR).ok()?;

        // Receive exactly one response
        let mut buf = [0u8; 2048];
        let (len, src): (usize, SocketAddr) = socket.recv_from(&mut buf).ok()?;

        let payload = std::str::from_utf8(&buf[..len]).ok()?;

        // Match:
        // "*** C64 Ultimate (V1.47) 3.14 ***"
        let matches = payload
            .split("C64 Ultimate")
            .nth(1)
            .and_then(|s| s.split(')').nth(1))
            .map(|s| s.trim_start())
            .and_then(|s| s.split_whitespace().next())
            .filter(|v| v.chars().all(|c| c.is_ascii_digit() || c == '.'));

        if matches.is_some() {
            Some(src.ip().to_string())
        } else {
            None
        }
    }
    fn post(&self, url: &String, file: &String) -> io::Result<()> {
        let path = Path::new(file);
        let mut buf: Vec<u8> = vec![];
        fs::File::open(path)?.read_to_end(&mut buf)?;
        
        let mut req = String::from("http://");
        req.push_str(self.service_ip.as_ref().unwrap().as_str());
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
}
