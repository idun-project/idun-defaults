use std::net::{UdpSocket, SocketAddr};
use std::time::Duration;

/// Detect if there is a C64 Ultimate on the LAN and return its IP address.
pub fn detect() -> Option<String> {
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
