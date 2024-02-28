rgbe: A library for handling RGBE-format HDR images
===================================================

This crate contains types which correspond to the RGBE8 and RGB9E5 common-exponent unsigned float formats used to represent high dynamic range images.
RGBE8 images can be loaded from Radiance HDR files and can also be loaded from and saved to PNG files which save the exponent in the alpha channel
(compatible with [hdrpng.js](https://enkimute.github.io/hdrpng.js/)) for significantly smaller file sizes.
RGBE8 images (as well as floating point images) can also be converted to RGB9E5 for use as GPU textures.

As well as the library, this package also contasins a command-line tool `hdr2rgbe-png`
to compress Radiance HDR images into RGBE8 PNG, which results in much smaller file sizes.
