# librptadvsquelch development rules

Follow the shared rpt_advanced baseline in `../rpt_advanced/AGENTS.md` and
`../rpt_advanced/QUALITY.md`. This component owns only MICOR discriminator-noise
squelch and sample-clocked VOX hold state. Qualification, envelope measurement,
PCM conversion, and receiver orchestration remain with librptadvradio.

The exported interface is a versioned C-compatible descriptor with one opaque
handle. Do not allocate, lock, block, log, or perform I/O in update functions.
Use warning-free Rustdoc for Rust source and Doxygen for the C header.
