//! A 24 MP 16-bit RGB TIFF written as one uncompressed strip (as Photoshop and Lightroom write
//! them) is 144 MB. The `tiff` crate's default intermediate-buffer limit is 128 MiB, so such a
//! file was refused as "decoder limits exceeded" even though its pixels are within the decode
//! limits. Same layout as the photo that failed in the phase 0 baseline (4000 × 6000).

use lightcraft_codecs::{DecodeOptions, decode};

/// A minimal little-endian baseline TIFF: RGB, 16 bits per sample, uncompressed, one strip.
fn single_strip_rgb16(width: u32, height: u32) -> Vec<u8> {
    let strip_len = width as usize * height as usize * 3 * 2;
    let entries: u16 = 10;
    let ifd_off = 8u32;
    let ifd_len = 2 + entries as u32 * 12 + 4;
    let bps_off = ifd_off + ifd_len; // three SHORTs for BitsPerSample
    let data_off = bps_off + 6;

    let mut b = Vec::with_capacity(data_off as usize + strip_len);
    b.extend_from_slice(b"II");
    b.extend_from_slice(&42u16.to_le_bytes());
    b.extend_from_slice(&ifd_off.to_le_bytes());
    b.extend_from_slice(&entries.to_le_bytes());
    let mut entry = |tag: u16, ty: u16, count: u32, value: u32| {
        b.extend_from_slice(&tag.to_le_bytes());
        b.extend_from_slice(&ty.to_le_bytes());
        b.extend_from_slice(&count.to_le_bytes());
        b.extend_from_slice(&value.to_le_bytes());
    };
    const SHORT: u16 = 3;
    const LONG: u16 = 4;
    entry(256, LONG, 1, width); // ImageWidth
    entry(257, LONG, 1, height); // ImageLength
    entry(258, SHORT, 3, bps_off); // BitsPerSample
    entry(259, SHORT, 1, 1); // Compression: none
    entry(262, SHORT, 1, 2); // Photometric: RGB
    entry(273, LONG, 1, data_off); // StripOffsets
    entry(277, SHORT, 1, 3); // SamplesPerPixel
    entry(278, LONG, 1, height); // RowsPerStrip: the whole image
    entry(279, LONG, 1, strip_len as u32); // StripByteCounts
    entry(284, SHORT, 1, 1); // PlanarConfiguration: chunky
    b.extend_from_slice(&0u32.to_le_bytes()); // no next IFD
    for _ in 0..3 {
        b.extend_from_slice(&16u16.to_le_bytes());
    }
    // A horizontal ramp, so the decoded pixels can be checked.
    let row: Vec<u8> = (0..width)
        .flat_map(|x| {
            let v = ((x as u64 * 65535) / u64::from(width - 1)) as u16;
            [v, v, v].into_iter().flat_map(u16::to_le_bytes)
        })
        .collect();
    for _ in 0..height {
        b.extend_from_slice(&row);
    }
    b
}

#[test]
fn decodes_a_144_mb_single_strip_16_bit_tiff() {
    let (w, h) = (4000, 6000);
    let bytes = single_strip_rgb16(w, h);
    assert!(bytes.len() > 128 << 20, "the strip must exceed the tiff crate's default 128 MiB intermediate buffer");

    let decoded = decode(&bytes, DecodeOptions::default()).expect("a 144 MB 16-bit TIFF decodes");
    assert_eq!((decoded.image.width, decoded.image.height), (w as usize, h as usize));
}
