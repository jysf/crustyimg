//! Target-independent container sniffing (SPEC-072).
//!
//! AVIF DETECTION has to compile everywhere; AVIF DECODE does not. The decoder
//! (`super::avif`, via `re_rav1d`) is native-only — it is the one crate in the
//! tree that cannot compile to `wasm32-unknown-unknown` (DEC-064). But the wasm
//! build must still *recognize* an AVIF input so it can answer with a typed
//! `ImageError::CodecUnavailableOnTarget` instead of the vague
//! "unsupported or undetectable image format" the generic guesser would give.
//!
//! So the sniff lives here, apart from the decoder it dispatches to: a pure byte
//! predicate over the container header, no codec crate behind it. `decode_with_limits`
//! calls it on **both** targets and only the *branch it takes* is target-gated.
//!
//! (The HEIC sniff already had to work this way for the same reason — detection in
//! every build, decode behind the `heic` feature, DEC-056 — but it could keep its
//! sniff next to its decoder because `libheif-rs` is only pulled in by the feature.
//! `re_rav1d` is a default, non-optional dep, so its module has to leave the wasm
//! build entirely and the sniff has to move out from under it.)

/// Whether `bytes` is an ISOBMFF file whose `ftyp` box advertises an AVIF brand.
///
/// Detection is by container brand (not the `image` guesser) so dispatch does
/// not depend on `image`'s optional avif feature. Scans the major brand and the
/// compatible-brands list of the leading `ftyp` box for `avif`/`avis`.
pub(crate) fn is_avif(bytes: &[u8]) -> bool {
    // ftyp box: [size:u32][b"ftyp"][major:4][minor:4][compatible brands: 4*n].
    if bytes.len() < 12 || &bytes[4..8] != b"ftyp" {
        return false;
    }
    let box_size = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    // Clamp the brand scan to the declared box size and the actual buffer.
    let end = box_size.clamp(8, bytes.len());
    // Major brand at 8..12, then compatible brands every 4 bytes from 16.
    if is_avif_brand(&bytes[8..12]) {
        return true;
    }
    let mut i = 16;
    while i + 4 <= end {
        if is_avif_brand(&bytes[i..i + 4]) {
            return true;
        }
        i += 4;
    }
    false
}

fn is_avif_brand(brand: &[u8]) -> bool {
    matches!(brand, b"avif" | b"avis")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_avif_detects_ftyp_avif_major_brand() {
        // size(0) + "ftyp" + "avif" + minor + compat
        let mut b = Vec::new();
        b.extend_from_slice(&0x20u32.to_be_bytes());
        b.extend_from_slice(b"ftypavif");
        b.extend_from_slice(&0u32.to_be_bytes());
        b.extend_from_slice(b"avifmif1miafMA1B");
        assert!(is_avif(&b));
    }

    #[test]
    fn is_avif_detects_compatible_brand() {
        let mut b = Vec::new();
        b.extend_from_slice(&0x1cu32.to_be_bytes());
        b.extend_from_slice(b"ftypmif1"); // major = mif1
        b.extend_from_slice(&0u32.to_be_bytes());
        b.extend_from_slice(b"mif1avif"); // compatible brands include avif
        assert!(is_avif(&b));
    }

    #[test]
    fn is_avif_rejects_png_and_short() {
        assert!(!is_avif(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]));
        assert!(!is_avif(b"ftyp")); // too short
        let mut heic = Vec::new();
        heic.extend_from_slice(&0x18u32.to_be_bytes());
        heic.extend_from_slice(b"ftypheic");
        heic.extend_from_slice(&0u32.to_be_bytes());
        heic.extend_from_slice(b"heicmif1");
        assert!(!is_avif(&heic));
    }
}
