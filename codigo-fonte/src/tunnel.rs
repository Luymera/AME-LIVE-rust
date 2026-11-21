// Controla o cloudflared (túnel "quick", sem conta) para gerar um link https público.
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

static GEN: AtomicU64 = AtomicU64::new(0); // invalida leitores de túneis antigos

struct Tun {
    status: String, // "", "starting", "on", "stopping", "error"
    url: String,
    msg: String,
    child: Option<Child>,
}
static TUN: Mutex<Tun> = Mutex::new(Tun { status: String::new(), url: String::new(), msg: String::new(), child: None });

fn exe_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("AME_CLOUDFLARED") {
        let p = PathBuf::from(p);
        return p.exists().then_some(p);
    }
    let p = std::env::current_exe().ok()?.parent()?.join("cloudflared.exe");
    p.exists().then_some(p)
}

fn find_url(line: &str) -> Option<String> {
    let i = line.find("https://")?;
    let u: String = line[i..].chars().take_while(|c| !c.is_whitespace() && *c != '|').collect();
    u.ends_with(".trycloudflare.com").then_some(u)
}

pub fn snapshot() -> (String, String, String, bool) {
    let t = TUN.lock().unwrap();
    let st = if t.status.is_empty() { "off".to_string() } else { t.status.clone() };
    (st, t.url.clone(), t.msg.clone(), exe_path().is_some())
}

pub fn start(port: u16) {
    let mut t = TUN.lock().unwrap();
    if t.child.is_some() {
        return;
    }
    let Some(exe) = exe_path() else {
        t.status = "error".into();
        t.msg = "O arquivo cloudflared.exe não está ao lado do app.".into();
        return;
    };
    let mut cmd = Command::new(exe);
    cmd.args(["tunnel", "--url", &format!("http://127.0.0.1:{port}"), "--no-autoupdate"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // sem janela preta
    }
    match cmd.spawn() {
        Ok(mut child) => {
            let err = child.stderr.take();
            let gen = GEN.fetch_add(1, Ordering::SeqCst) + 1;
            t.child = Some(child);
            t.status = "starting".into();
            t.url.clear();
            t.msg.clear();
            std::thread::spawn(move || {
                if let Some(err) = err {
                    for line in BufReader::new(err).lines().map_while(Result::ok) {
                        if let Some(u) = find_url(&line) {
                            let mut t = TUN.lock().unwrap();
                            if GEN.load(Ordering::SeqCst) != gen {
                                return;
                            }
                            t.url = u;
                            t.status = "on".into();
                        }
                    }
                }
                let mut t = TUN.lock().unwrap();
                if GEN.load(Ordering::SeqCst) != gen {
                    return;
                }
                if t.status == "on" {
                    t.status = "off".into();
                    t.url.clear();
                } else {
                    t.status = "error".into();
                    t.msg = "O túnel fechou antes de gerar o link. Confira a internet ou se o antivírus bloqueou.".into();
                }
                t.child = None;
            });
        }
        Err(e) => {
            t.status = "error".into();
            t.msg = format!("Não consegui abrir o cloudflared: {e}");
        }
    }
}

pub fn stop() {
    let mut t = TUN.lock().unwrap();
    GEN.fetch_add(1, Ordering::SeqCst);
    if let Some(c) = t.child.as_mut() {
        let _ = c.kill();
    }
    t.child = None;
    t.status = "off".into();
    t.url.clear();
    t.msg.clear();
}

pub fn kill() {
    if let Ok(mut t) = TUN.lock() {
        if let Some(c) = t.child.as_mut() {
            let _ = c.kill();
        }
    }
}

