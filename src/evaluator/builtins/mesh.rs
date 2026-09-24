// src/evaluator/builtins/mesh.rs

use super::Evaluator;
use crate::evaluator::value::Value;
use crate::parser::Node;

use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

struct ActiveMesh {
    socket: Arc<UdpSocket>,
    peer: Arc<Mutex<Option<SocketAddr>>>,
    inbox: Arc<Mutex<Vec<String>>>,
    mode: Mutex<String>,
}

static MESH: OnceLock<Arc<ActiveMesh>> = OnceLock::new();

fn get_mesh() -> Option<Arc<ActiveMesh>> {
    MESH.get().cloned()
}

fn spawn_listener(
    sock: Arc<UdpSocket>,
    peer: Arc<Mutex<Option<SocketAddr>>>,
    inbox: Arc<Mutex<Vec<String>>>,
) {
    thread::spawn(move || {
        let mut buf = [0u8; 65535];
        loop {
            match sock.recv_from(&mut buf) {
                Ok((len, src)) => {
                    let msg = String::from_utf8_lossy(&buf[..len]).to_string();

                    if let Ok(mut p) = peer.lock() {
                        *p = Some(src);
                    }

                    if msg == "PUNCH" {
                        let _ = sock.send_to(b"PUNCH_ACK", src);
                        continue;
                    }
                    if msg == "PUNCH_ACK" {
                        continue;
                    }

                    if let Ok(mut q) = inbox.lock() {
                        q.push(msg);
                    }
                }
                Err(_) => thread::sleep(Duration::from_millis(2)),
            }
        }
    });
}

fn start_mesh(sock: UdpSocket, initial_peer: Option<SocketAddr>, mode: &str) -> Result<(), String> {
    if MESH.get().is_some() {
        return Err("Mesh already on in this process. One host/connect per run.".into());
    }
    let _ = sock.set_nonblocking(true);
    let socket = Arc::new(sock);
    let peer = Arc::new(Mutex::new(initial_peer));
    let inbox = Arc::new(Mutex::new(Vec::new()));
    spawn_listener(socket.clone(), peer.clone(), inbox.clone());
    let state = Arc::new(ActiveMesh {
        socket,
        peer,
        inbox,
        mode: Mutex::new(mode.to_string()),
    });
    MESH.set(state)
        .map_err(|_| "Mesh already on.".to_string())?;
    Ok(())
}

impl Evaluator {
    /// mesh[bridge].host / .connect / .send / .recv / .peer
    pub fn eval_mesh_builtin(&mut self, mode: &str, method: &str, args: Vec<Node>) -> Value {
        let mode = if mode.is_empty() { "bridge" } else { mode };

        match method {
            "host" => {
                let port = if args.is_empty() {
                    8080u16
                } else {
                    match self.eval(args[0].clone()) {
                        Value::Integer(p) if p > 0 && p < 65536 => p as u16,
                        _ => 8080,
                    }
                };
                let sock = match UdpSocket::bind(format!("0.0.0.0:{port}")) {
                    Ok(s) => s,
                    Err(e) => return Value::Error(format!("mesh[{mode}].host({port}) failed: {e}")),
                };
                if let Err(e) = start_mesh(sock, None, mode) {
                    return Value::Error(e);
                }
                println!(
                    "\x1b[36m⚡ [MESH:{mode}] Host :{port} — 2-way when peer connects\x1b[0m"
                );
                Value::Boolean(true)
            }

            "connect" | "join" => {
                if args.is_empty() {
                    return Value::Error(
                        format!("mesh[{mode}].connect(\"127.0.0.1:8080\")").into(),
                    );
                }
                let addr_str = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    o => o.to_string(),
                };
                let remote: SocketAddr = match addr_str.parse() {
                    Ok(a) => a,
                    Err(e) => return Value::Error(format!("Bad address '{addr_str}': {e}")),
                };
                let sock = match UdpSocket::bind("0.0.0.0:0") {
                    Ok(s) => s,
                    Err(e) => return Value::Error(format!("bind failed: {e}")),
                };
                if let Err(e) = start_mesh(sock, Some(remote), mode) {
                    return Value::Error(e);
                }
                if let Some(m) = get_mesh() {
                    for _ in 0..5 {
                        let _ = m.socket.send_to(b"PUNCH", remote);
                    }
                }
                println!("\x1b[32m⚡ [MESH:{mode}] Connected → {remote}\x1b[0m");
                Value::Boolean(true)
            }

            "send" => {
                if args.is_empty() {
                    return Value::Error(format!("mesh[{mode}].send(\"text\")"));
                }
                let payload = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    o => o.to_string(),
                };
                let Some(m) = get_mesh() else {
                    return Value::Error("Mesh off. mesh[bridge].host or .connect first.".into());
                };
                let mut dest = None;
                for _ in 0..50 {
                    dest = *m.peer.lock().unwrap();
                    if dest.is_some() {
                        break;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                let Some(addr) = dest else {
                    return Value::Error("No peer locked yet.".into());
                };
                match m.socket.send_to(payload.as_bytes(), addr) {
                    Ok(_) => Value::Boolean(true),
                    Err(e) => Value::Error(format!("send failed: {e}")),
                }
            }

            "recv" => {
                let Some(m) = get_mesh() else {
                    return Value::Null;
                };
                let mut q = m.inbox.lock().unwrap();
                if q.is_empty() {
                    Value::Null
                } else {
                    Value::StringVal(q.remove(0))
                }
            }

                        "peer" => {
                let Some(m) = get_mesh() else {
                    return Value::Null;
                };
                let addr = {
                    let guard = m.peer.lock().unwrap();
                    *guard
                };
                match addr {
                    Some(a) => Value::StringVal(a.to_string()),
                    None => Value::Null,
                }
            }

            _ => Value::Error(format!(
                "mesh[{mode}].{method} unknown. Use host, connect, send, recv, peer"
            )),
        }
    }
}