# Small deterministic image fixtures

The red still images were created locally using installed ImageMagick, not a live service or the production Rust encoder. All are 2x2 red pixels: PNG/WebP exact RGBA [255,0,0,255]; JPEG quality100 decodes [254,0,0,255]. Commands from repository root:

```sh
convert -size 2x2 xc:red -strip PNG32:crates/gateway/tests/fixtures/codex_images/red.png
convert -size 2x2 xc:red -strip -sampling-factor 1x1 -quality 100 crates/gateway/tests/fixtures/codex_images/red.jpg
convert -size 2x2 xc:red -strip -define webp:lossless=true crates/gateway/tests/fixtures/codex_images/red.webp
```

`pixel-bomb.png`, `axis-bomb.png`, `raster-bomb.png` are 77-byte adversarial variants of red.png. A local Python script replaced IHDR width/height/depth with (5000,5000,8), (20000,1,8), (4096,4096,16), respectively, and recomputed that chunk's CRC32 using zlib. Color is RGBA. The original small IDAT/tail is intentionally retained: these are oversized advertised-header rejection fixtures, not usable large images or a normal-capacity claim. The tests establish the separate dimension/pixel/raster guards before allocating the declared raster. Malformed/truncated/wrong-format tests separately establish genuine decode failure; red image tests establish actual decoded pixels and original-byte preservation.

Boundary tests extend a real small PNG with inert trailing file bytes to exactly32MiB. The existing PNG is genuinely decoded and the complete original compressed/base64 payload is preserved. This exercises wire/file/aggregate accounting without expensive generated full-size rasters; it is not empirical normal workload sizing.


IMG-R1 remediation fixtures:

- `inner-vp8-bomb.webp` is only48bytes: literal RIFF/VP8X2x2 plus an inner VP8 header. It is tested only with a panic sentinel immediately before decoder construction, preventing any dangerous raster allocation if the safety guard regresses.
- `red-lossy.webp` and `red-alpha.webp` were created by ImageMagick with `-size 2x2 xc:red -strip -quality 95`, and `-size 2x2 xc:rgba(255,0,0,0.5) -strip -quality 95`. Independent ImageMagick pixel enumeration gives RGBA[255,1,0,255] and[255,1,0,128]. The alpha variant is VP8X+uncompressed ALPH+VP8.
- `red-compressed-alpha.webp` uses the same half-alpha command at16x16. Its ALPH compression bits are1 (lossless alpha with implicit frame dimensions), independent first pixel[255,1,0,128], with small spatial RGB variations from lossy compression. The complete independent ImageMagick decode is retained as1024byte `red-compressed-alpha.rgba` using `convert red-compressed-alpha.webp -depth 8 RGBA:red-compressed-alpha.rgba`; all256 pixels are compared exactly, rather than assuming uniform reconstructed color. `red-alpha-lossless.webp` uses2x2 plus `-define webp:lossless=true`, yielding VP8L with alpha and pixel[255,0,0,128].
- `red-extended.webp` independently wraps the original real VP8L data with a VP8X2x2 header. `red-animation.webp` wraps the real ALPH+VP8 data in one ANMF2x2 with ANIM/VP8X. Their real first-raster pixels remain[255,0,0,255] and[255,1,0,128], respectively. The test is first-raster preservation, not full animation fidelity.

Additional small test-only chunk fixtures exercise geometry disagreement, simple/VP8L/ANMF/ALPH-related guards, ambiguous carriers, checked sizes/padding and truncations. None of those negative cases is allowed to enter the mature decoder. A matching header with invalid entropy separately must enter that decoder and fail, showing that the safety preflight cannot substitute for genuine decoding.
