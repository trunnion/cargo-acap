# `cargo-acap-build`

This folder contains a Docker image suitable for Rust development with the [AXIS Camera Application
Platform](https://www.axis.com/en-us/products/analytics/acap). The image contains a custom Rust build and associated
C/C++ cross-compilers supporting `--target=`:

* `x86_64-unknown-linux-gnu` (for build scripts)
* `aarch64-axis-linux-gnu`
* `armv5te-axis-linux-gnueabi`
* `arm-axis-linux-gnueabi`
* `armv7-axis-linux-gnueabi`
* `armv7-axis-linux-gnueabihf`
* `mipsel-axis-linux-gnu`

## Tags

This image is built from tagged releases of the Rust compiler in the `rust-lang/rust` GitHub repository. The resulting
Docker images are tagged to match, like `ghcr.io/trunnion/cargo-acap:1.68.0`.

## Details

### Docker assets

This folder contains a `Dockerfile`, a `rust-config.toml` for the Rust build, and a script which adds the
`mipsisa32r2el-axis-linux-gnu` target. Building this image requires a considerable amount of resources; see
[the `rustc-dev` docs](https://rustc-dev-guide.rust-lang.org/building/prerequisites.html#hardware) for more specific
guidance.

### GitHub Actions

`.github/workflows/` contains two associated GitHub Actions definitions:

* `build-rust-image.yml`: when invoked a `rustVersion` argument, it builds and pushes a `cargo-acap-build:<rustVersion>`
  Docker image.
* `automatically-build-rust-images.yml`: when invoked manually or by the scheduler, it compares the list of Docker
  images to the list of Rust releases and triggers `build-rust-image.yml` builds as needed.

### Custom MIPS target

Axis ARTPEC-4 and ARTPEC-5 chips contain MIPS32r2 cores, and the Axis firmware uses them in a little-endian
configuration. `/proc/cpuinfo`:

```
system type		: Axis Artpec-5
machine			: Unknown
processor		: 0
cpu model		: MIPS 1004Kc V2.12
BogoMIPS		: 265.42
wait instruction	: yes
microsecond timers	: yes
tlb_entries		: 32
extra interrupt vector	: yes
hardware watchpoint	: yes, count: 4, address/irw mask: [0x0ffc, 0x0ffc, 0x0ffb, 0x0ffb]
isa			: mips1 mips2 mips32r1 mips32r2
ASEs implemented	: mips16 dsp mt
shadow register sets	: 1
kscratch registers	: 0
package			: 0
core			: 0
VPE			: 0
VCED exceptions		: not available
VCEI exceptions		: not available

<3 more cores>
```

This processor corresponds to Rust targets `mipsel-*`, except that none of them quite fit:

* `mipsel-unknown-linux-gnu` use the incompatible `fpxx` floating point ABI
* `mipsel-unknown-linux-musl` use the compatible soft-float ABI but use `musl` as libc, which cannot be statically
   linked on MIPS due to interference with `libbacktrace-sys`
* `mipsel-unknown-linux-uclibc` is again compatible but again requires an alternate libc `.so`

`mipsel-axis-linux-gnu` is a custom target specifically for `cargo-acap`, configured as `mipsel-unknown-linux-gnu`
except with `+mips32r2,+soft-float` like `mipsel-unknown-linux-musl` and `mipsel-unknown-linux-uclibc`. Additionally,
this target is built with the Axis-provided GNU toolchain, since there is no commonly-distributed C toolchain with
exactly the right configuration.

### Custom other targets

The other Axis targets:

* `aarch64-axis-linux-gnu`
* `armv5te-axis-linux-gnueabi`
* `arm-axis-linux-gnueabi`
* `armv7-axis-linux-gnueabi`
* `armv7-axis-linux-gnueabihf`

…are 100% identical to their usual `*-unknown-linux-*` counterparts except for having `axis` in the name.
