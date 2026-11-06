#[cfg(not(target_os = "android"))]
use arboard::Clipboard;
use std::sync::{Mutex, OnceLock};

/// Android: no system clipboard reached from here (it is Java's); copy and paste work within
/// the game.
#[cfg(target_os = "android")]
#[derive(Default)]
struct Clipboard(String);

#[cfg(target_os = "android")]
impl Clipboard {
    fn new() -> Result<Clipboard, ()> {
        Ok(Clipboard::default())
    }
    fn get_text(&mut self) -> Result<String, ()> {
        Ok(self.0.clone())
    }
    fn set_text(&mut self, text: &str) -> Result<(), ()> {
        self.0 = text.to_string();
        Ok(())
    }
}

static CLIPBOARD: OnceLock<Mutex<Clipboard>> = OnceLock::new();

/// Returns the clipboard, creating it on first success.
/// If creation fails, the next call tries again.
fn clipboard() -> Option<&'static Mutex<Clipboard>> {
    if let Some(c) = CLIPBOARD.get() {
        return Some(c);
    }

    let new = Clipboard::new().ok()?;
    Some(CLIPBOARD.get_or_init(|| Mutex::new(new)))
}

pub fn get() -> Option<String> {
    clipboard()?.lock().ok()?.get_text().ok()
}

pub fn set(text: &str) -> bool {
    clipboard()
        .and_then(|c| c.lock().ok())
        .map(|mut c| c.set_text(text).is_ok())
        .unwrap_or(false)
}
