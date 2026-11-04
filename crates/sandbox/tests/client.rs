//! Client halves (API 1.2) against a fake player's game: the bell's committed `code/client.wasm`
//! registers, builds its HUD as parts, answers its bindings, takes a packet and a key, and may
//! not act from a binding.
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chiacraft_sandbox::client::*;
use chiacraft_sandbox::{Arg, ItemStack};

fn bell() -> Arc<ClientMod> {
    let wasm = std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/packages/bell/code/client.wasm")).unwrap();
    Arc::new(ClientMod::load("bell", &wasm).unwrap())
}

#[derive(Default)]
struct Seen {
    sent: Vec<(String, Vec<u8>)>,
    shown: Vec<String>,
}

struct Fake {
    seen: Arc<Mutex<Seen>>,
    act: bool,
}

impl ClientGame for Fake {
    fn me(&mut self) -> Option<([f64; 3], f32, f32)> {
        Some(([0.0, 64.0, 0.0], 20.0, 20.0))
    }
    fn day_time(&mut self) -> u64 {
        6000
    }
    fn raining(&mut self) -> bool {
        false
    }
    fn creative(&mut self) -> bool {
        false
    }
    fn component(&mut self, _: &str) -> Option<Vec<u8>> {
        None
    }
    fn inventory_slot(&mut self, _: u32) -> ItemStack {
        ItemStack { item: 0, count: 0, data: 0 }
    }
    fn screen(&mut self) -> String {
        String::new()
    }
    fn can_act(&self) -> bool {
        self.act
    }
    fn send(&mut self, channel: &str, data: &[u8]) {
        self.seen
            .lock()
            .unwrap()
            .sent
            .push((channel.into(), data.to_vec()));
    }
    fn open_screen(&mut self, _: &str) {}
    fn close_screen(&mut self) {}
    fn show(&mut self, key: &str, _: &[Arg]) {
        self.seen.lock().unwrap().shown.push(key.into());
    }
    fn block_name(&mut self, _: u32) -> String {
        String::new()
    }
    fn block_state(&mut self, _: &str) -> Option<u32> {
        None
    }
    fn prop(&mut self, _: u32, _: &str) -> Option<String> {
        None
    }
    fn with_prop(&mut self, _: u32, _: &str, _: &str) -> Option<u32> {
        None
    }
    fn item_name(&mut self, _: u32) -> String {
        String::new()
    }
    fn item_id(&mut self, _: &str) -> Option<u32> {
        None
    }
    fn entity_kind(&mut self, _: u32) -> String {
        String::new()
    }
    fn entity_kind_id(&mut self, _: &str) -> Option<u32> {
        None
    }
}

#[test]
fn registers_its_hud_packet_and_key() {
    assert_eq!(
        bell().hooks(),
        &[
            ClientHook::Hud { name: "bell:rings".into(), anchor: "top_right".into(), order: 0 },
            ClientHook::Packet("bell:rings".into()),
            ClientHook::Key { name: "bell:ping".into(), default: "N".into() },
        ]
    );
}

#[test]
fn builds_binds_and_acts() {
    let seen: Arc<Mutex<Seen>> = Arc::default();
    let game = |act| Box::new(Fake { seen: seen.clone(), act }) as Box<dyn ClientGame>;
    let mut inst = ClientInstance::new(bell());
    let parts = inst.build(game(false), 0);
    assert!(well_formed(&parts));
    let kinds: Vec<PartKind> = parts.iter().map(|p| p.kind).collect();
    assert_eq!(kinds, [PartKind::Panel, PartKind::TextBind, PartKind::Container, PartKind::Progress, PartKind::TextBind]);
    assert_eq!(parts[0].class, "hud_badge");
    assert!(parts[4].style.at.is_some(), "the count sits inside the bar");
    // Nothing heard yet: the bar hides, the line says so.
    assert_eq!(inst.value(game(false), parts[3].bind), -1.0);
    assert_eq!(inst.text(game(false), parts[1].bind).0, "bell.hud_none");
    // The server half says 13 rings.
    inst.packet(game(true), 1, &13u32.to_le_bytes());
    assert_eq!(inst.text(game(false), parts[1].bind).0, "bell.hud");
    assert!((inst.value(game(false), parts[3].bind) - 0.3).abs() < 1e-6);
    // The key sends a ping and shows a line; a binding could do neither.
    inst.key(game(true), 2);
    {
        let s = seen.lock().unwrap();
        assert_eq!(s.sent, [("bell:ping".to_string(), b"ping".to_vec())]);
        assert_eq!(s.shown, ["bell.pinged"]);
    }
    inst.key(game(false), 2);
    assert_eq!(seen.lock().unwrap().sent.len(), 1, "a call that may not act sends nothing");
    assert!(inst.stopped().is_none());
}

#[test]
fn malformed_parts_are_refused() {
    let part = |children| Part {
        kind: PartKind::Container,
        style: Style {
            row: false,
            width: None,
            height: None,
            gap: 0.0,
            pad: 0.0,
            grow: 0.0,
            align: 0,
            justify: 0,
            at: None,
            scale: 1.0,
            flags: 0,
            class: String::new(),
        },
        class: String::new(),
        text: String::new(),
        args: Vec::new(),
        children,
        bind: 0,
        count: 0,
        item: 0,
        size: (0.0, 0.0),
    };
    assert!(well_formed(&[part(1), part(0)]));
    // A child that isn't there.
    assert!(!well_formed(&[part(2), part(0)]));
    // A size that isn't a number.
    let mut nan = part(0);
    nan.style.width = Some(f32::NAN);
    assert!(!well_formed(&[nan]));
    assert!(!well_formed(&vec![part(0); PARTS + 1]));
}
