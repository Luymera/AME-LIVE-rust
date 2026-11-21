#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]
// AME LIVE! OVERDOSE - servidor em Rust.
// Entrega a página, faz o chat das salas e repassa os sinais WebRTC
// (mesmo protocolo do server.js antigo). A página vai embutida no executável.
mod tunnel;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse, Response},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

const PAGE: &str = include_str!("../ame-live.html");
static NEXT_ID: AtomicU64 = AtomicU64::new(1);
static PORT_NOW: AtomicU64 = AtomicU64::new(3000);

struct Client {
    tx: UnboundedSender<Message>,
    name: String,
    photo: String,
    status: String,
    presence: String,
    in_call: bool,
    mic: bool,
    cam: bool,
    screen: bool,
}
#[derive(Default)]
struct Room {
    clients: HashMap<String, Client>,
    history: VecDeque<Value>,
}
#[derive(Default)]
struct Hub {
    rooms: HashMap<String, Room>,
}
type Shared = Arc<Mutex<Hub>>;

fn clean_name(v: &str) -> String {
    let n: String = v.trim().chars().take(24).collect();
    if n.is_empty() { "anônima".into() } else { n }
}
fn clean_room(v: &str) -> String {
    let r: String = v
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
        .take(30)
        .collect();
    if r.is_empty() { "sala-de-stream".into() } else { r }
}
fn clean_status(v: &str) -> String {
    v.trim().chars().take(30).collect()
}
fn clean_pres(v: &str) -> String {
    if v == "away" || v == "busy" { v.to_string() } else { "online".to_string() }
}
fn clean_photo(v: &str) -> String {
    let ok = v.len() < 80_000
        && ["jpeg", "png", "webp"].iter().any(|k| {
            v.strip_prefix(&format!("data:image/{k};base64,"))
                .map_or(false, |b| {
                    !b.is_empty() && b.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'+' || c == b'/' || c == b'=')
                })
        });
    if ok { v.to_string() } else { String::new() }
}

fn pubc(id: &str, c: &Client) -> Value {
    json!({"id": id, "name": c.name, "photo": c.photo, "status": c.status, "presence": c.presence, "inCall": c.in_call, "mic": c.mic, "cam": c.cam, "screen": c.screen})
}
fn peer_msg(id: &str, c: &Client) -> Value {
    let mut p = pubc(id, c);
    p["t"] = json!("peer");
    p
}
fn send(c: &Client, v: &Value) {
    let _ = c.tx.send(Message::Text(v.to_string()));
}
fn bc(r: &Room, v: &Value, except: Option<&str>) {
    for (id, c) in &r.clients {
        if Some(id.as_str()) != except {
            send(c, v);
        }
    }
}
fn leave(hub: &mut Hub, room: &mut Option<String>, id: &str) {
    if let Some(name) = room.take() {
        if let Some(r) = hub.rooms.get_mut(&name) {
            r.clients.remove(id);
            bc(r, &json!({"t": "left", "id": id}), None);
            if r.clients.is_empty() {
                hub.rooms.remove(&name);
            }
        }
    }
}

fn handle(hub: &Shared, id: &str, tx: &UnboundedSender<Message>, room: &mut Option<String>, m: &Value) {
    let mut h = hub.lock().unwrap();
    let s = |k: &str| m[k].as_str().unwrap_or("").to_string();
    let t = s("t");

    if t == "join" {
        leave(&mut h, room, id);
        let rname = clean_room(&s("room"));
        let c = Client {
            tx: tx.clone(),
            name: clean_name(&s("name")),
            photo: clean_photo(&s("photo")),
            status: clean_status(&s("status")),
            presence: clean_pres(&s("presence")),
            in_call: false,
            mic: false,
            cam: false,
            screen: false,
        };
        let r = h.rooms.entry(rname.clone()).or_default();
        let peers: Vec<Value> = r.clients.iter().map(|(i, c)| pubc(i, c)).collect();
        let history: Vec<Value> = r.history.iter().cloned().collect();
        send(&c, &json!({"t": "welcome", "id": id, "peers": peers, "history": history}));
        bc(r, &peer_msg(id, &c), None);
        r.clients.insert(id.to_string(), c);
        *room = Some(rname);
        return;
    }

    let Some(rn) = room.clone() else { return };
    let Some(r) = h.rooms.get_mut(&rn) else { return };
    match t.as_str() {
        "profile" => {
            if let Some(c) = r.clients.get_mut(id) {
                c.name = clean_name(&s("name"));
                c.photo = clean_photo(&s("photo"));
                c.status = clean_status(&s("status"));
                c.presence = clean_pres(&s("presence"));
            }
            if let Some(c) = r.clients.get(id) {
                let p = peer_msg(id, c);
                bc(r, &p, Some(id));
            }
        }
        "chat" => {
            let text: String = s("text").trim().chars().take(500).collect();
            if text.is_empty() {
                return;
            }
            let Some(c) = r.clients.get(id) else { return };
            let time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let msg = json!({"name": c.name, "photo": c.photo, "text": text, "time": time});
            r.history.push_back(msg.clone());
            if r.history.len() > 50 {
                r.history.pop_front();
            }
            let mut out = msg;
            out["t"] = json!("chat");
            bc(r, &out, None);
        }
        "call" => {
            let on = m["on"].as_bool().unwrap_or(false);
            if let Some(c) = r.clients.get_mut(id) {
                c.in_call = on;
                if !on {
                    c.mic = false;
                    c.cam = false;
                    c.screen = false;
                }
            }
            bc(r, &json!({"t": "call", "id": id, "on": on}), Some(id));
        }
        "media" => {
            let (mic, cam, screen) = (
                m["mic"].as_bool().unwrap_or(false),
                m["cam"].as_bool().unwrap_or(false),
                m["screen"].as_bool().unwrap_or(false),
            );
            if let Some(c) = r.clients.get_mut(id) {
                c.mic = mic;
                c.cam = cam;
                c.screen = screen;
            }
            bc(r, &json!({"t": "media", "id": id, "mic": mic, "cam": cam, "screen": screen}), Some(id));
        }
        "signal" => {
            if let Some(to) = r.clients.get(&s("to")) {
                send(to, &json!({"t": "signal", "from": id, "data": m["data"]}));
            }
        }
        _ => {}
    }
}

async fn conn(socket: WebSocket, hub: Shared) {
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst).to_string();
    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = unbounded_channel::<Message>();
    let writer = tokio::spawn(async move {
        let mut iv = tokio::time::interval(Duration::from_secs(25));
        loop {
            tokio::select! {
                m = rx.recv() => match m {
                    Some(m) => if sink.send(m).await.is_err() { break },
                    None => break,
                },
                _ = iv.tick() => if sink.send(Message::Ping(vec![])).await.is_err() { break },
            }
        }
    });
    let mut room: Option<String> = None;
    while let Some(Ok(msg)) = stream.next().await {
        match msg {
            Message::Text(t) => {
                if let Ok(v) = serde_json::from_str::<Value>(&t) {
                    handle(&hub, &id, &tx, &mut room, &v);
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
    leave(&mut hub.lock().unwrap(), &mut room, &id);
    writer.abort();
}

async fn root(ws: Option<WebSocketUpgrade>, State(hub): State<Shared>) -> Response {
    match ws {
        Some(ws) => ws.max_message_size(256 * 1024).on_upgrade(move |s| conn(s, hub)),
        None => Html(PAGE).into_response(),
    }
}


// ----- convite: liga/desliga o túnel (só o app pode mandar; quem vem de fora do túnel não) -----
fn from_app(h: &HeaderMap) -> bool {
    h.contains_key("x-ame") && !h.contains_key("cf-connecting-ip") && !h.contains_key("x-forwarded-for") && !h.contains_key("cf-ray")
}
fn tunnel_json() -> axum::Json<Value> {
    let (status, url, msg, available) = tunnel::snapshot();
    axum::Json(json!({"status": status, "url": url, "msg": msg, "available": available}))
}
async fn tunnel_get() -> axum::Json<Value> {
    tunnel_json()
}
async fn tunnel_start(headers: HeaderMap) -> Response {
    if !from_app(&headers) {
        return StatusCode::FORBIDDEN.into_response();
    }
    tunnel::start(PORT_NOW.load(Ordering::SeqCst) as u16);
    tunnel_json().into_response()
}
async fn tunnel_stop(headers: HeaderMap) -> Response {
    if !from_app(&headers) {
        return StatusCode::FORBIDDEN.into_response();
    }
    tunnel::stop();
    tunnel_json().into_response()
}

fn start_server() -> u16 {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
        rt.block_on(async move {
            let l = match tokio::net::TcpListener::bind(("127.0.0.1", 3000)).await {
                Ok(l) => l,
                Err(_) => tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.unwrap(),
            };
            let port = l.local_addr().unwrap().port();
            PORT_NOW.store(port as u64, Ordering::SeqCst);
            let _ = tx.send(port);
            let app = Router::new()
                .route("/", get(root))
                .route("/tunnel", get(tunnel_get))
                .route("/tunnel/start", post(tunnel_start))
                .route("/tunnel/stop", post(tunnel_stop))
                .with_state(Shared::default());
            let _ = axum::serve(l, app).await;
        });
    });
    rx.recv().unwrap()
}

// servidor.txt ao lado do .exe: usa um servidor online em vez do interno
fn remote_url() -> Option<String> {
    let dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let t = std::fs::read_to_string(dir.join("servidor.txt")).ok()?;
    let t = t.trim().to_string();
    (t.starts_with("http://") || t.starts_with("https://")).then_some(t)
}

fn main() {
    use tao::{
        dpi::LogicalSize,
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Icon, WindowBuilder},
    };
    use wry::{WebContext, WebViewBuilder};

    let port = start_server();
    let url = remote_url().unwrap_or_else(|| format!("http://localhost:{port}"));
    let event_loop = tao::event_loop::EventLoopBuilder::<()>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let icon = Icon::from_rgba(include_bytes!("../icon64.rgba").to_vec(), 64, 64).ok();
    let window = WindowBuilder::new()
        .with_title("AME LIVE! OVERDOSE")
        .with_inner_size(LogicalSize::new(1280.0, 800.0))
        .with_window_icon(icon)
        .build(&event_loop)
        .unwrap();
    let data = std::env::var_os("LOCALAPPDATA").map(|p| std::path::PathBuf::from(p).join("AME-LIVE"));
    let mut ctx = WebContext::new(data);
    let _webview = WebViewBuilder::with_web_context(&mut ctx)
        .with_url(&url)
        .with_new_window_req_handler(|u: String| {
            if cfg!(windows) {
                let _ = std::process::Command::new("cmd").args(["/C", "start", "", &u]).spawn();
            }
            false
        })
        .with_ipc_handler(move |req: wry::http::Request<String>| {
            if req.body() == "quit" {
                let _ = proxy.send_event(());
            }
        })
        .build(&window)
        .unwrap();

    event_loop.run(move |event, _, flow| {
        *flow = ControlFlow::Wait;
        if matches!(event, Event::UserEvent(()) | Event::WindowEvent { event: WindowEvent::CloseRequested, .. }) {
            tunnel::kill();
            *flow = ControlFlow::Exit;
        }
    });
}

