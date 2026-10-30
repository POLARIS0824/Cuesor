use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::{
    model::ImeMode,
    platform::{self, AppResult},
};

pub const DEFAULT_BRIDGE_PORT: u16 = 23333;
const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// Compute standard SHA-1 digest (RFC 3174).
pub fn sha1(data: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [
        0x67452301,
        0xEFCDAB89,
        0x98BADCFE,
        0x10325476,
        0xC3D2E1F0,
    ];

    let bit_len = (data.len() as u64) * 8;
    let mut msg = Vec::from(data);
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];

        for i in 0..80 {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
    }

    let mut out = [0u8; 20];
    for (i, val) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&val.to_be_bytes());
    }
    out
}

/// Base64 encoding according to RFC 4648.
pub fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        let idx0 = (b0 >> 2) as usize;
        let idx1 = (((b0 & 0x03) << 4) | (b1 >> 4)) as usize;
        let idx2 = (((b1 & 0x0F) << 2) | (b2 >> 6)) as usize;
        let idx3 = (b2 & 0x3F) as usize;

        out.push(TABLE[idx0] as char);
        out.push(TABLE[idx1] as char);
        if chunk.len() > 1 {
            out.push(TABLE[idx2] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[idx3] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Compute Sec-WebSocket-Accept key (RFC 6455).
pub fn compute_accept_key(sec_websocket_key: &str) -> String {
    let mut combined = String::with_capacity(sec_websocket_key.trim().len() + WS_GUID.len());
    combined.push_str(sec_websocket_key.trim());
    combined.push_str(WS_GUID);
    let digest = sha1(combined.as_bytes());
    base64_encode(&digest)
}

/// Create an unmasked WebSocket text frame for server-to-client transmission.
pub fn make_text_frame(text: &str) -> Vec<u8> {
    let payload = text.as_bytes();
    let mut frame = Vec::with_capacity(10 + payload.len());
    frame.push(0x81); // FIN (0x80) | Text Opcode (0x01)
    let len = payload.len();
    if len <= 125 {
        frame.push(len as u8);
    } else if len <= 65535 {
        frame.push(126);
        frame.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        frame.push(127);
        frame.extend_from_slice(&(len as u64).to_be_bytes());
    }
    frame.extend_from_slice(payload);
    frame
}

/// Create a Pong frame (RFC 6455).
pub fn make_pong_frame(payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(2 + payload.len().min(125));
    frame.push(0x8A); // FIN | Pong Opcode
    let len = payload.len().min(125);
    frame.push(len as u8);
    frame.extend_from_slice(&payload[..len]);
    frame
}

pub fn format_ime_json_with_colors(mode: ImeMode, colors: Option<(&str, &str)>) -> String {
    let mode_str = match mode {
        ImeMode::Chinese => "zh",
        ImeMode::Latin => "en",
        ImeMode::Unknown => "unknown",
    };
    if let Some((zh, en)) = colors {
        format!("{{\"mode\":\"{mode_str}\",\"source\":\"cuesor-bridge\",\"colors\":{{\"zh\":\"{zh}\",\"en\":\"{en}\"}}}}")
    } else {
        format!("{{\"mode\":\"{mode_str}\",\"source\":\"cuesor-bridge\"}}")
    }
}

pub fn format_ime_json(mode: ImeMode) -> String {
    format_ime_json_with_colors(mode, None)
}

pub struct BridgeServer {
    stop_signal: Arc<AtomicBool>,
    client_count: Arc<AtomicUsize>,
    thread: Option<thread::JoinHandle<()>>,
}

impl BridgeServer {
    pub fn start(port: u16) -> AppResult<Self> {
        Self::start_with_dir(port, None)
    }

    pub fn start_with_dir(port: u16, data_dir: Option<std::path::PathBuf>) -> AppResult<Self> {
        let listener = TcpListener::bind(("127.0.0.1", port))?;
        listener.set_nonblocking(true)?;

        let stop_signal = Arc::new(AtomicBool::new(false));
        let client_count = Arc::new(AtomicUsize::new(0));

        let stop_clone = Arc::clone(&stop_signal);
        let count_clone = Arc::clone(&client_count);

        let thread_handle = thread::Builder::new()
            .name("cuesor-obsidian-bridge".into())
            .spawn(move || {
                run_bridge_loop(listener, stop_clone, count_clone, data_dir);
            })?;

        Ok(Self {
            stop_signal,
            client_count,
            thread: Some(thread_handle),
        })
    }

    pub fn client_count(&self) -> usize {
        self.client_count.load(Ordering::Relaxed)
    }

    pub fn stop(&mut self) {
        self.stop_signal.store(true, Ordering::SeqCst);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for BridgeServer {
    fn drop(&mut self) {
        self.stop();
    }
}

struct ConnectedClient {
    stream: TcpStream,
    is_websocket: bool,
}

fn get_active_colors(data_dir: Option<&std::path::Path>) -> Option<(String, String)> {
    let dir = data_dir?;
    let settings = crate::settings::Settings::load(dir).ok()?;
    let active = settings.active_colors();
    Some((active.chinese_color.clone(), active.english_color.clone()))
}

fn run_bridge_loop(
    listener: TcpListener,
    stop: Arc<AtomicBool>,
    client_count: Arc<AtomicUsize>,
    data_dir: Option<std::path::PathBuf>,
) {
    let mut clients: Vec<ConnectedClient> = Vec::new();
    let mut last_mode = ImeMode::Unknown;

    while !stop.load(Ordering::Relaxed) {
        // Accept new connections
        match listener.accept() {
            Ok((stream, _)) => {
                let _ = stream.set_nonblocking(true);
                let _ = stream.set_nodelay(true);
                clients.push(ConnectedClient {
                    stream,
                    is_websocket: false,
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => {}
        }

        // Process client handshakes and messages
        let mut i = 0;
        while i < clients.len() {
            let client = &mut clients[i];
            let mut remove = false;

            if !client.is_websocket {
                let mut buffer = [0u8; 2048];
                match client.stream.read(&mut buffer) {
                    Ok(0) => {
                        remove = true;
                    }
                    Ok(n) => {
                        let req = String::from_utf8_lossy(&buffer[..n]);
                        let is_upgrade = req.to_ascii_lowercase().contains("upgrade: websocket");
                        if is_upgrade {
                            // Extract Sec-WebSocket-Key
                            let mut key = None;
                            for line in req.lines() {
                                if let Some((name, value)) = line.split_once(':') {
                                    if name.trim().eq_ignore_ascii_case("Sec-WebSocket-Key") {
                                        key = Some(value.trim().to_string());
                                        break;
                                    }
                                }
                            }
                            if let Some(key) = key {
                                let accept = compute_accept_key(&key);
                                let response = format!(
                                    "HTTP/1.1 101 Switching Protocols\r\n\
                                     Upgrade: websocket\r\n\
                                     Connection: Upgrade\r\n\
                                     Sec-WebSocket-Accept: {}\r\n\r\n",
                                    accept
                                );
                                if client.stream.write_all(response.as_bytes()).is_ok() {
                                    client.is_websocket = true;
                                    // Immediately deliver current mode and synchronize last_mode
                                    let current_mode = platform::sample_foreground_ime();
                                    last_mode = current_mode;
                                    let colors = get_active_colors(data_dir.as_deref());
                                    let colors_ref = colors.as_ref().map(|(zh, en)| (zh.as_str(), en.as_str()));
                                    let frame = make_text_frame(&format_ime_json_with_colors(current_mode, colors_ref));
                                    let _ = client.stream.write_all(&frame);
                                } else {
                                    remove = true;
                                }
                            } else {
                                remove = true;
                            }
                        } else if req.starts_with("GET ") {
                            // Standard HTTP health-check / status response
                            let current_mode = platform::sample_foreground_ime();
                            let json = format_ime_json(current_mode);
                            let response = format!(
                                "HTTP/1.1 200 OK\r\n\
                                 Content-Type: application/json; charset=utf-8\r\n\
                                 Access-Control-Allow-Origin: *\r\n\
                                 Content-Length: {}\r\n\
                                 Connection: close\r\n\r\n{}",
                                json.len(),
                                json
                            );
                            let _ = client.stream.write_all(response.as_bytes());
                            remove = true;
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(_) => {
                        remove = true;
                    }
                }
            } else {
                // Read WebSocket frames from client
                let mut buffer = [0u8; 512];
                match client.stream.read(&mut buffer) {
                    Ok(0) => {
                        remove = true;
                    }
                    Ok(n) => {
                        if n >= 2 {
                            let opcode = buffer[0] & 0x0F;
                            if opcode == 0x08 {
                                // Close frame
                                remove = true;
                            } else if opcode == 0x09 {
                                // Ping frame -> respond Pong with unmasked application data (RFC 6455)
                                let mask = (buffer[1] & 0x80) != 0;
                                let mut len = (buffer[1] & 0x7F) as usize;
                                let mut offset = 2;
                                if len == 126 && n >= 4 {
                                    len = u16::from_be_bytes([buffer[2], buffer[3]]) as usize;
                                    offset = 4;
                                }
                                let mask_key = if mask && n >= offset + 4 {
                                    let k = [
                                        buffer[offset],
                                        buffer[offset + 1],
                                        buffer[offset + 2],
                                        buffer[offset + 3],
                                    ];
                                    offset += 4;
                                    Some(k)
                                } else {
                                    None
                                };
                                let payload = if n >= offset {
                                    &buffer[offset..n.min(offset + len)]
                                } else {
                                    &[]
                                };
                                let unmasked_payload: Vec<u8> = if let Some(k) = mask_key {
                                    payload
                                        .iter()
                                        .enumerate()
                                        .map(|(idx, &b)| b ^ k[idx % 4])
                                        .collect()
                                } else {
                                    payload.to_vec()
                                };
                                let pong = make_pong_frame(&unmasked_payload);
                                let _ = client.stream.write_all(&pong);
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(_) => {
                        remove = true;
                    }
                }
            }

            if remove {
                clients.swap_remove(i);
            } else {
                i += 1;
            }
        }

        let ws_count = clients.iter().filter(|c| c.is_websocket).count();
        client_count.store(ws_count, Ordering::Relaxed);

        if ws_count > 0 {
            // Sample current foreground IME mode
            let current_mode = platform::sample_foreground_ime();
            if current_mode != last_mode {
                last_mode = current_mode;
                let colors = get_active_colors(data_dir.as_deref());
                let colors_ref = colors.as_ref().map(|(zh, en)| (zh.as_str(), en.as_str()));
                let frame = make_text_frame(&format_ime_json_with_colors(current_mode, colors_ref));
                let mut i = 0;
                while i < clients.len() {
                    if clients[i].is_websocket {
                        if clients[i].stream.write_all(&frame).is_err() {
                            clients.swap_remove(i);
                            continue;
                        }
                    }
                    i += 1;
                }
            }
            thread::sleep(Duration::from_millis(30));
        } else {
            // Idle when no clients connected to conserve resources
            thread::sleep(Duration::from_millis(200));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc6455_websocket_accept_key_matches_spec() {
        let key = "dGhlIHNhbXBsZSBub25jZQ==";
        let accept = compute_accept_key(key);
        assert_eq!(accept, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    }

    #[test]
    fn sha1_matches_known_hashes() {
        // Test vector: sha1("") = da39a3ee5e6b4b0d3255bfef95601890afd80709
        let empty = sha1(b"");
        let empty_hex: String = empty.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(empty_hex, "da39a3ee5e6b4b0d3255bfef95601890afd80709");

        // Test vector: sha1("The quick brown fox jumps over the lazy dog")
        let fox = sha1(b"The quick brown fox jumps over the lazy dog");
        let fox_hex: String = fox.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(fox_hex, "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12");
    }

    #[test]
    fn base64_encode_matches_standard() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn text_frame_format_length_variants() {
        let short = make_text_frame("hello");
        assert_eq!(short[0], 0x81);
        assert_eq!(short[1], 5);
        assert_eq!(&short[2..], b"hello");

        let medium_str = "a".repeat(200);
        let medium = make_text_frame(&medium_str);
        assert_eq!(medium[0], 0x81);
        assert_eq!(medium[1], 126);
        assert_eq!(u16::from_be_bytes([medium[2], medium[3]]), 200);
        assert_eq!(&medium[4..], medium_str.as_bytes());
    }

    #[test]
    fn bridge_server_starts_and_responds_to_http() {
        let port = 29933;
        let mut server = BridgeServer::start(port).expect("start server");
        assert_eq!(server.client_count(), 0);

        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
        stream
            .write_all(b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
            .expect("write GET");
        let mut response = String::new();
        stream.read_to_string(&mut response).expect("read response");
        assert!(response.contains("HTTP/1.1 200 OK"));
        assert!(response.contains("cuesor-bridge"));

        server.stop();
    }

    #[test]
    fn bridge_server_accepts_websocket_handshake_and_answers_masked_ping() {
        let port = 29934;
        let mut server = BridgeServer::start(port).expect("start server");

        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
        let client_key = "dGhlIHNhbXBsZSBub25jZQ==";
        let handshake = format!(
            "GET /cuesor HTTP/1.1\r\n\
             Host: 127.0.0.1:{port}\r\n\
             Upgrade: WebSocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Key: {client_key}\r\n\
             Sec-WebSocket-Version: 13\r\n\r\n"
        );
        stream.write_all(handshake.as_bytes()).expect("write handshake");

        // Read handshake response and initial IME mode frame
        let mut buffer = [0u8; 1024];
        let mut n = 0;
        for _ in 0..20 {
            if let Ok(bytes) = stream.read(&mut buffer[n..]) {
                n += bytes;
                let text = String::from_utf8_lossy(&buffer[..n]);
                if text.contains("101 Switching Protocols") && text.contains("s3pPLMBiTxaQ9kYGzzhZRbK+xOo=") {
                    break;
                }
            }
            thread::sleep(Duration::from_millis(20));
        }
        let text = String::from_utf8_lossy(&buffer[..n]);
        assert!(text.contains("101 Switching Protocols"));
        assert!(text.contains("s3pPLMBiTxaQ9kYGzzhZRbK+xOo="));

        // Now send a masked Ping frame with payload "ping"
        // Opcode: 0x89 (FIN | Ping)
        // Length + Mask: 0x84 (Mask bit set, len 4)
        // Mask key: [0x12, 0x34, 0x56, 0x78]
        // Payload: "ping" = [0x70, 0x69, 0x6e, 0x67]
        // Masked: "ping" ^ mask
        let mask = [0x12u8, 0x34, 0x56, 0x78];
        let original_payload = b"ping";
        let masked_payload: Vec<u8> = original_payload
            .iter()
            .enumerate()
            .map(|(i, b)| b ^ mask[i % 4])
            .collect();
        let mut ping_frame = vec![0x89, 0x84];
        ping_frame.extend_from_slice(&mask);
        ping_frame.extend_from_slice(&masked_payload);

        stream.write_all(&ping_frame).expect("write ping");

        // Server must reply with Pong (0x8A) containing the UNMASKED original payload "ping"
        let mut pong_buf = [0u8; 256];
        let mut pong_n = 0;
        for _ in 0..30 {
            if let Ok(bytes) = stream.read(&mut pong_buf[pong_n..]) {
                pong_n += bytes;
                // Look for pong frame: 0x8A opcode
                for i in 0..pong_n {
                    if pong_buf[i] == 0x8A && i + 2 <= pong_n {
                        let len = (pong_buf[i + 1] & 0x7F) as usize;
                        if i + 2 + len <= pong_n {
                            let pong_payload = &pong_buf[i + 2..i + 2 + len];
                            assert_eq!(pong_payload, b"ping");
                            server.stop();
                            return;
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!("Did not receive expected unmasked Pong frame");
    }
}
