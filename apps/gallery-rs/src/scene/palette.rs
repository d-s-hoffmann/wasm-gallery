//! The colour palette shared by the pixel scenes.

/// One pixel: red, green, blue, alpha.
pub type Rgba = [u8; 4];

/// A bright HSV rainbow of opaque colours.
///
/// The compiler computes this table while building, so the finished
/// `.wasm` holds the 1 KiB of bytes and no code to make them.
pub static RAINBOW: [Rgba; 256] = rainbow();

const fn rainbow<const N: usize>() -> [Rgba; N] {
    let mut pal = [[0; 4]; N];
    // `for` loops call iterator methods, which a const fn can't use yet.
    let mut i = 0;
    while i < N {
        let h = i as f64 / N as f64 * 6.0;
        // `as u8` truncates toward zero, like Go's uint8(...).
        let x = (255.0 * (1.0 - (h % 2.0 - 1.0).abs())) as u8;
        pal[i] = match h as usize {
            0 => [255, x, 0, 255],
            1 => [x, 255, 0, 255],
            2 => [0, 255, x, 255],
            3 => [0, x, 255, 255],
            4 => [x, 0, 255, 255],
            _ => [255, 0, x, 255],
        };
        i += 1;
    }
    pal
}
