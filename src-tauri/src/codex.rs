use std::{
    env,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

const PET_ADDR: &str = "127.0.0.1:9527";

/// Handles the lightweight CLI mode used by Codex, Claude Code and Hermes hooks.
///
/// Hook input is supplied on stdin. Legacy `notify` input is normally appended
/// as the final command-line argument, so both sources are supported.
pub fn handle_cli_event() -> bool {
    let args: Vec<String> = env::args().collect();
    let Some(mode) = args.get(1).map(String::as_str) else {
        return false;
    };

    let (provider, event, payload_index) = match mode {
        "--codex-hook" => (
            "codex",
            args.get(2).map(String::as_str).unwrap_or("unknown"),
            3,
        ),
        "--claude-hook" => (
            "claude",
            args.get(2).map(String::as_str).unwrap_or("unknown"),
            3,
        ),
        "--hermes-hook" => (
            "hermes",
            args.get(2).map(String::as_str).unwrap_or("unknown"),
            3,
        ),
        "--provider-hook" => (
            args.get(2).map(String::as_str).unwrap_or("local"),
            args.get(3).map(String::as_str).unwrap_or("unknown"),
            4,
        ),
        "--codex-notify" => ("codex", "stop", 2),
        _ => return false,
    };

    let payload = args
        .get(payload_index)
        .cloned()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(read_stdin);
    let payload = if payload.trim().is_empty() {
        "{}".to_string()
    } else {
        payload
    };

    // Hooks must never own the desktop application's lifecycle. In particular,
    // restarting the pet here means that quitting it is only temporary and a
    // burst of concurrent hooks can race to create multiple windows. If the pet
    // is not running, silently drop the visual-only event instead.
    let _ = send_event(provider, event, &payload);

    true
}

fn read_stdin() -> String {
    let mut input = String::new();
    let _ = std::io::stdin().read_to_string(&mut input);
    input
}

fn send_event(provider: &str, event: &str, payload: &str) -> std::io::Result<()> {
    let safe_provider: String = provider
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
        .collect();
    let safe_event: String = event
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
        .collect();
    let address: SocketAddr = PET_ADDR.parse().expect("valid loopback address");
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_millis(180))?;
    stream.set_write_timeout(Some(Duration::from_millis(300)))?;
    stream.set_read_timeout(Some(Duration::from_millis(300)))?;

    let request = format!(
        "POST /api/event/{safe_provider}/{safe_event} HTTP/1.1\r\nHost: {PET_ADDR}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
        payload.len()
    );
    stream.write_all(request.as_bytes())?;

    let mut response = [0_u8; 64];
    let read = stream.read(&mut response)?;
    let status = String::from_utf8_lossy(&response[..read]);
    if status.starts_with("HTTP/1.1 2") || status.starts_with("HTTP/1.0 2") {
        Ok(())
    } else {
        Err(std::io::Error::other("pet rejected hook event"))
    }
}
