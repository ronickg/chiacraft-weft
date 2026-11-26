//! A mod's storage calls, as bytes: each op is a kind, a key and a value size. The quotas hold,
//! refused keys never become files, and the folder holds only valid keys.
#![no_main]

use chiacraft_sandbox::Storage;
use chiacraft_sandbox::storage::{MAX_BYTES, MAX_KEYS, key_ok};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let dir = std::env::temp_dir().join(format!("chiacraft-sandbox-fuzz-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut s = Storage::open(dir.clone()).unwrap();
    let mut rest = data;
    while rest.len() >= 3 {
        let (op, klen, size) = (rest[0], (rest[1] % 70) as usize, rest[2] as usize * 4096);
        rest = &rest[3..];
        let take = klen.min(rest.len());
        let key = String::from_utf8_lossy(&rest[..take]).into_owned();
        rest = &rest[take..];
        match op % 3 {
            0 => {
                let ok = s.set(&key, vec![0; size]).is_ok();
                assert!(!ok || key_ok(&key));
            }
            1 => s.remove(&key),
            _ => {
                let _ = s.get(&key);
            }
        }
        let keys = s.keys();
        assert!(keys.len() <= MAX_KEYS);
        assert!(keys.iter().all(|k| key_ok(k)));
        assert!(
            keys.iter()
                .map(|k| s.get(k).unwrap().len() as u64)
                .sum::<u64>()
                <= MAX_BYTES
        );
    }
    for e in std::fs::read_dir(&dir).unwrap().flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        assert!(key_ok(&name), "stray file {name}");
    }
    let _ = std::fs::remove_dir_all(&dir);
});
