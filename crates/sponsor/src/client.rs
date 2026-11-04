//! The game's side: ask a sponsor, over iroh, for a DID or for its status.
//!
//! The protocol, on one bidirectional stream of a connection with the ALPN [`ALPN`]:
//!
//! 1. The asker writes its request and finishes its side: the JSON of a
//!    [`chiacraft_wallet::sponsor::Request`] (at most [`MAX_REQUEST`] bytes), or the 6 bytes
//!    [`STATUS`] (`status`).
//! 2. The sponsor answers with one JSON object and finishes: `{"status": <code>, "body": <JSON>}`.
//!    `status` is the code the HTTP API gave (200 a DID launched or the status; 400 malformed, a
//!    bad signature or time; 403 already sponsored, with its `did` and `launcher`, or another
//!    network; 429 busy or the budget spent; 503 not synced, no free coin, the chain refused), and
//!    `body` the body it gave (`{"error": "…"}` for a refusal). A world DID's request (a `Request`
//!    with `world`, [`chiacraft_wallet::sponsor::Request::sign_world`]) is answered the same way:
//!    403 also when the owner's DID isn't held by the address, and 403 naming the world's DID when
//!    its key already has one; 429 also past the identity's world DIDs for the day.
//! 3. The asker closes the connection.
//!
//! The game then reads a DID answer with [`chiacraft_wallet::sponsor::read_answer`]`(status,
//! &body)`. A sponsor's address is its endpoint id (the setting `chiacraft:sponsors` lists them).
use chiacraft_wallet::sponsor::Request;
use iroh::{Endpoint, EndpointAddr};
use std::time::Duration;

/// The sponsor's ALPN.
pub const ALPN: &[u8] = b"chiacraft/sponsor/1";
/// The request for the sponsor's status (`/v1/status` over HTTP).
pub const STATUS: &[u8] = b"status";
/// The most a request may be (the sponsor reads no more).
pub const MAX_REQUEST: usize = 4 << 10;
/// The most an answer may be (an answer is a few hundred bytes).
pub const MAX_ANSWER: usize = 16 << 10;
/// How long finding and reaching the sponsor may take.
pub const CONNECT_WAIT: Duration = Duration::from_secs(20);
/// How long a DID request may wait for its answer: the sponsor holds a request until its bundle is
/// pushed, which can wait for a free coin (`max_wait_seconds`, 120 by default) and the chain.
pub const ANSWER_WAIT: Duration = Duration::from_secs(200);
/// How long the status may take.
pub const STATUS_WAIT: Duration = Duration::from_secs(10);

/// Ask the sponsor at `sponsor` (its endpoint id, or an address with its paths) for a DID: the
/// answer's status and body, for [`chiacraft_wallet::sponsor::read_answer`]. An `Err` is a
/// network problem (no answer), never a refusal.
pub async fn ask(ep: &Endpoint, sponsor: impl Into<EndpointAddr>, request: &Request) -> Result<(u16, String), String> {
    exchange(ep, sponsor, request.to_json().as_bytes(), ANSWER_WAIT).await
}

/// Ask the sponsor for its status: `200` and the status JSON (network, phase, height, funding
/// address, balance, budget left today, …).
pub async fn status(ep: &Endpoint, sponsor: impl Into<EndpointAddr>) -> Result<(u16, String), String> {
    exchange(ep, sponsor, STATUS, STATUS_WAIT).await
}

/// Send `request` as it is and read the answer (within `wait` once connected).
pub async fn exchange(ep: &Endpoint, sponsor: impl Into<EndpointAddr>, request: &[u8], wait: Duration) -> Result<(u16, String), String> {
    let conn = tokio::time::timeout(CONNECT_WAIT, ep.connect(sponsor, ALPN))
        .await
        .map_err(|_| "the sponsor can't be reached".to_owned())?
        .map_err(|e| format!("the sponsor can't be reached: {e}"))?;
    let answer = tokio::time::timeout(wait, async {
        let (mut send, mut recv) = conn.open_bi().await.map_err(|e| e.to_string())?;
        // A sponsor that stops reading (a request too large) still answers: read it either way.
        let wrote = send
            .write_all(request)
            .await
            .map_err(|e| e.to_string())
            .and_then(|()| send.finish().map_err(|e| e.to_string()));
        match recv.read_to_end(MAX_ANSWER).await {
            Ok(answer) => Ok(answer),
            Err(e) => Err(wrote.err().unwrap_or_else(|| e.to_string())),
        }
    })
    .await
    .map_err(|_| "the sponsor didn't answer in time".to_owned())
    .and_then(|r| r.map_err(|e| format!("the sponsor's answer: {e}")));
    conn.close(0u32.into(), b"thanks");
    read(&answer?)
}

/// An answer's status and body (the body as JSON text).
pub fn read(answer: &[u8]) -> Result<(u16, String), String> {
    let v: serde_json::Value = serde_json::from_slice(answer).map_err(|e| format!("the sponsor's answer can't be read: {e}"))?;
    let status = v
        .get("status")
        .and_then(serde_json::Value::as_u64)
        .and_then(|s| u16::try_from(s).ok())
        .ok_or("the sponsor's answer has no status")?;
    let body = v.get("body").ok_or("the sponsor's answer has no body")?;
    Ok((status, body.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answers_read() {
        let (status, body) = read(br#"{"status":429,"body":{"error":"busy"}}"#).unwrap();
        assert_eq!((status, body.as_str()), (429, r#"{"error":"busy"}"#));
        let e = chiacraft_wallet::sponsor::read_answer(status, &body).unwrap_err();
        assert!(e.to_string().contains("busy"), "{e}");
        assert!(read(b"{\"body\":{}}").is_err());
        assert!(read(b"{\"status\":200}").is_err());
        assert!(read(b"<html>").is_err());
    }
}
