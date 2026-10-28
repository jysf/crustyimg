use wasm_bindgen::prelude::*;
/// Decode an AVIF to tightly packed RGBA8; empty on error.
#[wasm_bindgen]
pub fn decode_rgba(bytes: &[u8]) -> Vec<u8> {
    oxideav_avif::decode(bytes).map(|i| i.to_rgba8()).unwrap_or_default()
}
