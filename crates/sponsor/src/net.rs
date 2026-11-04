//! The sponsor on iroh: its endpoint (the key kept in `<data_dir>/sponsor.key`, so its address,
//! the endpoint id, stays the same across restarts) and the serving side of the protocol in
//! [`crate::client`]:
//!
//! - A DID request (the JSON of [`chiacraft_wallet::sponsor::Request`], at most
//!   [`MAX_REQUEST`] bytes) → `{"status": 200, "body": `[`DidReply`](crate::DidReply)`}`;
//!   refusals `400` (not a request, too large, malformed, bad signature, time more than
//!   `max_skew_seconds` off), `403` (identity already sponsored: the body also names its `did` and
//!   `launcher`; network not served), `429` (a request for the identity in progress, the asking
//!   endpoint's daily limit, the daily budget, a full queue), `503` (not synced, no free coin in
//!   time, the chain refused the spend), each with the body `{"error": "…"}`.
//! - `status` → `{"status": 200, "body": `[`StatusReply`](crate::StatusReply)`}`.
//!
//! Every read has a time limit ([`READ_WAIT`]); the service's blocking calls run on tokio's
//! blocking pool.
use crate::client::{ALPN, MAX_REQUEST, STATUS};
use crate::service::{Refusal, Sponsor};
use iroh::Endpoint;
use iroh::endpoint::{Connection, presets};
use serde_json::{Value, json};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// How long the sponsor waits for each step of an asker: the connection, its stream, its request.
pub const READ_WAIT: Duration = Duration::from_secs(10);
/// How long the sponsor waits, after answering, for the asker to close.
pub const CLOSE_WAIT: Duration = Duration::from_secs(5);
/// The endpoint's key file in the data folder.
pub const KEY_FILE: &str = "sponsor.key";

/// The endpoint's key: `<data_dir>/sponsor.key` (32 bytes), made from fresh randomness the first
/// time (only its owner can read it), so the sponsor's address stays the same across restarts.
pub fn load_key(data_dir: &Path) -> Result<iroh::SecretKey, String> {
    let path = data_dir.join(KEY_FILE);
    match std::fs::read(&path) {
        Ok(bytes) => {
            let bytes: [u8; 32] = bytes
                .try_into()
                .map_err(|_| format!("{}: not an endpoint key (32 bytes)", path.display()))?;
            Ok(iroh::SecretKey::from_bytes(&bytes))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => make_key(&path),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

fn make_key(path: &Path) -> Result<iroh::SecretKey, String> {
    use std::io::Write;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let key = iroh::SecretKey::generate();
    // Written beside it and renamed, so a crash never leaves half a key.
    let new = path.with_extension("key.new");
    let _ = std::fs::remove_file(&new);
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&new)
        .map_err(|e| format!("{}: {e}", new.display()))?;
    file.write_all(&key.to_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|e| format!("{}: {e}", new.display()))?;
    std::fs::rename(&new, path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(key)
}

/// The sponsor's endpoint on n0's preset (its relays and address lookup), so games find it by its
/// id alone. It accepts only [`ALPN`].
pub async fn endpoint(key: iroh::SecretKey) -> Result<Endpoint, String> {
    Endpoint::builder(presets::N0)
        .secret_key(key)
        .alpns(vec![ALPN.to_vec()])
        .bind()
        .await
        .map_err(|e| format!("network: {e}"))
}

/// Serve `sponsor` on `ep` until the endpoint closes. At most `max_queue` + 32 askers are served
/// at once; more are closed at once.
pub async fn serve(sponsor: Arc<Sponsor>, ep: Endpoint) {
    let busy = Arc::new(tokio::sync::Semaphore::new(sponsor.config().max_queue + 32));
    while let Some(incoming) = ep.accept().await {
        let Ok(permit) = busy.clone().try_acquire_owned() else {
            incoming.refuse();
            continue;
        };
        let sponsor = sponsor.clone();
        tokio::spawn(async move {
            let _permit = permit;
            let conn = match tokio::time::timeout(READ_WAIT, incoming).await {
                Ok(Ok(conn)) => conn,
                Ok(Err(e)) => return log::debug!("an asker's connection: {e}"),
                Err(_) => return log::debug!("an asker's connection timed out"),
            };
            if conn.alpn() != ALPN {
                return conn.close(1u32.into(), b"not the sponsor protocol");
            }
            if let Err(e) = answer(&sponsor, &conn).await {
                log::debug!("{}: {e}", conn.remote_id());
            }
        });
    }
}

/// Read one request from `conn` and answer it.
async fn answer(sponsor: &Arc<Sponsor>, conn: &Connection) -> Result<(), String> {
    let (mut send, mut recv) = tokio::time::timeout(READ_WAIT, conn.accept_bi())
        .await
        .map_err(|_| "no stream".to_owned())?
        .map_err(|e| e.to_string())?;
    let read = tokio::time::timeout(READ_WAIT, recv.read_to_end(MAX_REQUEST))
        .await
        .map_err(|_| "no request in time".to_owned())?;
    let (status, body) = match read {
        Ok(request) => handle(sponsor, &request, &conn.remote_id().to_string()).await,
        Err(iroh::endpoint::ReadToEndError::TooLong) => {
            let _ = recv.stop(0u32.into());
            (400, json!({ "error": format!("the request is larger than {MAX_REQUEST} bytes") }))
        }
        Err(e) => return Err(e.to_string()),
    };
    let text = json!({ "status": status, "body": body }).to_string();
    send.write_all(text.as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    send.finish().map_err(|e| e.to_string())?;
    // Wait for the asker to read it and close.
    let _ = tokio::time::timeout(CLOSE_WAIT, conn.closed()).await;
    Ok(())
}

/// A request's status and body. `client` is the asker's endpoint id.
async fn handle(sponsor: &Arc<Sponsor>, request: &[u8], client: &str) -> (u16, Value) {
    let sponsor = sponsor.clone();
    if request == STATUS {
        return match tokio::task::spawn_blocking(move || sponsor.status()).await {
            Ok(s) => (200, serde_json::to_value(s).unwrap_or_default()),
            Err(_) => (500, json!({ "error": "status failed" })),
        };
    }
    let req: chiacraft_wallet::sponsor::Request = match serde_json::from_slice(request) {
        Ok(r) => r,
        Err(e) => {
            let r = Refusal::new(400, format!("not a DID request: {e}"));
            return (r.status, r.body());
        }
    };
    let client = client.to_owned();
    match tokio::task::spawn_blocking(move || sponsor.request(&req, &client)).await {
        Ok(Ok(reply)) => (200, serde_json::to_value(reply).unwrap_or_default()),
        Ok(Err(r)) => (r.status, r.body()),
        Err(_) => (500, json!({ "error": "the request failed" })),
    }
}
