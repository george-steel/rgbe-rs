use bytemuck::{Pod, Zeroable};
use half::f16;

// Aligned representation of Radiance RGBE8 pixel.
// r, g, and b are subnormal mantissas and e (taking the place of the alpha channel) is a common exponent.
// This is commonly loaded from Radiance pictures (.hdr).
//
// As it uses 4 u8 components, this firmat can also be used with ONG compression,
// with the exponent taking the place of the alpha channel.
// This will preserve chroma but distort luminance if loaded as a normal PNG,
// making thumbnailers somewhat useful for image identification.
#[repr(C, align(4))]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Pod, Zeroable)]
pub struct RGBE8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub e: u8, // bias of 128
}

// Aligned epresentation of rgb9e5ufloat texel.
// Field order is from LSB to MSB with 9 bits each of subnormal R, G, and B mantissa
// then 5 bits of a common exponent.
#[repr(transparent)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Pod, Zeroable)]
pub struct RGB9E5(pub u32);

// Aligned representation of rgba16float texel
#[repr(C, align(8))]
#[derive(PartialEq, Clone, Copy, Debug, Pod, Zeroable)]
pub struct RGBA16F {
    pub r: f16,
    pub g: f16,
    pub b: f16,
    pub a: f16,
}

impl RGB9E5 {
    // Clamp and pack a triple of RGB float values into an RGB9E5 value.
    //
    // Ported from the C++ example in the DirectX docs (MIT licensed)
    // https://github.com/microsoft/DirectX-Graphics-Samples/blob/master/MiniEngine/Core/Color.cpp
    pub fn pack(rgb: [f32;3]) -> Self {
        const MAX_F14:f32 = (0x1FF << 7) as f32;
        const MIN_NORM_F14:f32 = 1.0 / ((1 << 16) as f32);
        let r = rgb[0].clamp(0.0, MAX_F14);
        let g = rgb[1].clamp(0.0, MAX_F14);
        let b = rgb[2].clamp(0.0, MAX_F14);

        // Compute the maximum channel, no less than 1.0*2^-15
        let max_channel =  MIN_NORM_F14.max(r).max(g).max(b);

        // Take the exponent of the maximum channel (rounding up the 9th bit) and
        // add 15 to it.  When added to the channels, it causes the implicit '1.0'
        // bit and the first 8 mantissa bits to be shifted down to the low 9 bits
        // of the mantissa, rounding the truncated bits.

        // Calculate the shared exponent
        // Add 15 to the exponent and 0x4000 (half an f14 ULP) to the mantissa
        // so that rounding effects the final exponent before clearing the mantissa.
        let bias_bits = (max_channel.to_bits() + 0x07804000) & 0x7F800000;
        let bias = f32::from_bits(bias_bits);

        // This shifts the 9-bit values we need into the lowest bits, rounding as
        // needed.  Note that if the channel has a smaller exponent than the max
        // channel, it will shift even more.  This is intentional.
        let R = (r + bias).to_bits() & 0x1ff;
        let G = (g + bias).to_bits() & 0x1ff;
        let B = (b + bias).to_bits() & 0x1ff;

        // Convert the Bias to the correct exponent in the upper 5 bits.
        let E = (bias_bits << 4) + 0x10000000;

        // Combine the fields.  RGB floats have unwanted data in the upper 9
        // bits.  Only red needs to mask them off because green and blue shift
        // it out to the left.
        RGB9E5(E | (B << 18) | (G << 9) | R)
    }

    // Convert a packed color to individual floats
    pub fn unpack(self) -> [f32;3] {
        let bias = (((self.0 & 0xf8000000) >> 27) as f32 - 15.0).exp2();
        let r = ((self.0) & 0x000001ff) as f32 * bias / 512.0;
        let g = ((self.0 >> 9) & 0x000001ff) as f32 * bias / 512.0;
        let b = ((self.0 >> 18) & 0x000001ff) as f32 * bias / 512.0;
        [r, g, b]
    }
}

impl RGBE8 {
    // Pack a triple of RGB float values into an RGBE8.
    // This is not as optimized as RGB9E5::pack since it is designed for use in tooling instead of asset loading.
    pub fn pack(rgb: [f32;3]) -> Self {
        let max_channel = f32::MIN_POSITIVE.max(rgb[0]).max(rgb[1]).max(rgb[2]);
        // round to 8 bits of precision than take the next power of 2.
        let bias = f32::from_bits((max_channel.to_bits() + 0x00808000) & 0x7F800000);

        let r = ((rgb[0] / bias) * 256.0).round().clamp(0.0,255.0) as u8;
        let g = ((rgb[1] / bias) * 256.0).round().clamp(0.0,255.0) as u8;
        let b = ((rgb[2] / bias) * 256.0).round().clamp(0.0,255.0) as u8;
        let e = ((bias.to_bits() >> 23) + 1).clamp(0,255) as u8;

        RGBE8{r, g, b, e}
    }

    pub fn unpack(self) -> [f32;3] {
        let bias = ((self.e as f32) - 128.0).exp2();
        let r = ((self.r as f32) / 256.0) * bias;
        let g = ((self.g as f32) / 256.0) * bias;
        let b = ((self.b as f32) / 256.0) * bias;
        [r,g,b]
    }

    // Repack 
    pub fn repack_rgb9e5(self) -> RGB9E5 {
        let e = (self.e as i32) - 128;
        if e <= 15 && e >= -15 {
            // Simple case where we can leave mantissas alone.
            let e5 = (e + 15) as u32;
            let r = self.r as u32;
            let g = self.g as u32;
            let b = self.b as u32;
            return RGB9E5((e5 << 27) | (b << 19) | (g << 10) | (r << 1));
        } else {
            return RGB9E5::pack(self.unpack());
        }
    }
}

impl RGBA16F {
    fn into_rgb9e5(self) -> RGB9E5 {
        RGB9E5::pack([self.r.to_f32(), self.g.to_f32(), self.b.to_f32()])
    }

    fn into_rgbe8(self) -> RGBE8 {
        RGBE8::pack([self.r.to_f32(), self.g.to_f32(), self.b.to_f32()])
    }
}
