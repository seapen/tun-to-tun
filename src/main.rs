use std::sync::Arc;
use tun_tap::{Iface, Mode};
use std::thread;
use std::process::Command;

fn cmd(cmd: &str, args: &[&str]) {
    let ecode = Command::new("ip").args(args).spawn().unwrap().wait().unwrap();
    assert!(ecode.success(), "Failed to execte {}", cmd);
}

fn create_tun(name: &str, ip: &str) -> (Arc<Iface>, Arc<Iface>) {
    let tun = Iface::new(name, Mode::Tun).unwrap();
    let tun = Arc::new(tun);
    cmd("ip", &["addr", "add", "dev", tun.name(), ip]);
    cmd("ip", &["link", "set", "up", "dev", tun.name()]);
    (Arc::clone(&tun), Arc::clone(&tun))
}

fn tun_to_tun(reader: Arc<Iface>, writer: Arc<Iface>) {
    let mut buf = vec![0; 1522];
    loop {
        let amount = reader.recv(&mut buf).unwrap();
        writer.send(&buf[0..amount]).unwrap();
    }
}

fn main() {
    let (tun1_writer, tun1_reader) = create_tun("tun_server", "10.0.0.2/24");
    let (tun2_writer, tun2_reader) = create_tun("tun_client", "10.0.0.3/24");

    let t1_t2 = thread::spawn(move || {tun_to_tun(tun1_reader, tun2_writer);});
    let t2_t1 = thread::spawn(move || {tun_to_tun(tun2_reader, tun1_writer);});

    t1_t2.join().unwrap();
    t2_t1.join().unwrap();
}