# librptadvsquelch

Portable MICOR discriminator-noise squelch and sample-clocked VOX hold for
rpt_advanced. The runtime is `librptadvsquelch.so.2`, with ABI 2 exposed
through `rptadv_squelch_descriptor` and
`include/rptadvsquelch/rptadvsquelch.h`.

The MICOR algorithm and tests come from `librptadvradio/src/micor_squelch.rs`
at commit `bfdc71738874188055169f4abb3024dafce44a67`. The VOX hold
advancement comes from `librptadvradio/src/receive_path.rs` at that same
commit. The library preserves those implementations behind a versioned
C-compatible descriptor; envelope measurement, PCM threshold conversion, and
receiver orchestration remain in the radio library.

Create one handle per receiver during setup and destroy it after callback use
stops. Validate the descriptor name, size and ABI before calling it. Pass
previous comparator status and squared discriminator-noise sample power to
`noise_update` once per native sample. For VOX, call `vox_advance` for each
increasing in-span native offset with caller-computed qualification, then call
`vox_finish` once with the span's native frame count; this also starts the next
span's offset count at zero. Neither update performs allocation or I/O.

ABI 2 uses `int32_t` status and an out-handle for construction, and
`uint32_t` booleans at the C boundary (zero is false; any nonzero input is
true; outputs are zero or one). There is no ABI 1 shim. Before upgrading,
remove the ABI 1 development package, install the ABI 2 development and
runtime packages, then rebuild and relink consumers against the ABI 2 header.
An ABI 1 descriptor must not be used as ABI 2.

Build with `make`; `make test` checks Rust behavior and the C descriptor.
`make platform-verify` adds staged install and Debian package checks. The
package names are `librptadvsquelch2` and `librptadvsquelch-dev`, version
`0.1.0~alpha2-1` for this release. The release tag is `v0.1.0-alpha.2`.
