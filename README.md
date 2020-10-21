# `cargo-acap`

A tool to build Rust programs for the [AXIS Camera Application Platform](https://www.axis.com/en-us/products/analytics/acap).

## Example

```console
$ cargo install cargo-acap
…
   Installed package `cargo-acap v0.1.0` (executable `cargo-acap`)
$ cargo new --bin your-app
     Created binary (application) `your-app` package
$ cd your-app/
$ cargo acap build
cargo-acap: building ACAP package `your-app` using Docker image trunnion/cargo-acap-build:1.47.0
…
    Finished release [optimized] target(s) in 0.56s
$ ls target/acap/*.* | cat
 target/acap/your-app_0.1.0_aarch64.eap
 target/acap/your-app_0.1.0_aarch64.elf
 target/acap/your-app_0.1.0_armv5tej.eap
 target/acap/your-app_0.1.0_armv5tej.elf
 target/acap/your-app_0.1.0_armv6.eap
 target/acap/your-app_0.1.0_armv6.elf
 target/acap/your-app_0.1.0_armv7.eap
 target/acap/your-app_0.1.0_armv7.elf
 target/acap/your-app_0.1.0_armv7hf.eap
 target/acap/your-app_0.1.0_armv7hf.elf
 target/acap/your-app_0.1.0_mips.eap
 target/acap/your-app_0.1.0_mips.elf
```

`cargo-acap` builds your application using a Docker image specialized for compiling Rust for AXIS devices. It therefore
requires [Docker](https://docs.docker.com/get-docker/) to be installed and running in order to build.

## Application organization

```text
foo/
    Cargo.toml                      Rust project configuration + `[package.metadata.acap]`
    data/                           Copied to `/usr/local/packages/foo/data/`, i.e. `./data/`
    src/
        main.rs                     `fn main()`
    target/
        acap/
            foo_0.1.0_aarch64.eap   An ACAP package suitable for AArch64 devices
            foo_0.1.0_aarch64.elf   The executable inside foo_0.1.0_aarch64.eap prior to stripping symbols
            …4 more…
            foo_0.1.0_mips.eap      An ACAP package suitable for MIPS devices
            foo_0.1.0_mips.elf      The executable inside foo_0.1.0_mips.eap prior to stripping symbols
```

## Packaging

`cargo acap` builds your application's executable and then packages it into an `.eap` application package. The package
contains metadata which is set to sensible defaults but can be overridden via `Cargo.toml`.

```toml
[package.metadata.acap]
# The machine-friendly name of the package. Used for:
#
# * Installation path: `/usr/local/packages/<app_name>`
# * Executable path: `/usr/local/packages/<app_name>/<app_name>`
# * Generated package names: `<app_name>_1_2_3_arch.eap`
# * Myriad related files
#
# app_name = ""

# A user-friendly package name. The name will be displayed in the Axis product's web pages.
# display_name = ""

# The application name that will be displayed in the web pages' left-hand side menu.
# menu_name = ""

# The name of the vendor that created the application.
# vendor = ""

# The URL of the vendor's home page, to be linked in the product's web pages.
# vendor_homepage_url = ""

# The command line arguments to pass when the application is launched normally.
# launch_arguments = ""

# The command line arguments to pass when the application is executed to perform a custom license check, if using custom
# licensing.
# license_check_arguments = ""

# The Axis-assigned application ID, if using Axis licensing.
# axis_application_id = ""

# The start mode to use for this application.
# (One of: "respawn", "once", "never")
# start_mode = ""
```

## Targets

Different AXIS products use different [SoCs](https://en.wikipedia.org/wiki/System_on_a_chip) which contain different
processors. `cargo acap` provides a Docker build environment containing everything needed to build software for various
AXIS devices. These targets are described in `cargo acap targets table`:

| `cargo acap` `target` | Rust `--target`              |
| --------------------- | ---------------------------- |
| `aarch64`             | `aarch64-axis-linux-gnu`     |
| `armv5tej`            | `armv5te-axis-linux-gnueabi` |
| `armv6`               | `arm-axis-linux-gnueabi`     |
| `armv7`               | `armv7-axis-linux-gnueabi`   |
| `armv7hf`             | `armv7-axis-linux-gnueabihf` |
| `mips`                | `mipsel-axis-linux-gnu`      |

`cargo acap` is specialized for AXIS products, so it uses abbreviated target names on the left. The Rust target triples
defined by the `cargo-acap-build` environment are presented here for completeness.

These targets correspond to the system-on-chips listed in `cargo acap targets soc_table`:

| SOC           | Year | `cargo acap` `target` | Rust `--target`              |
| ------------- | ---- | --------------------- | ---------------------------- |
| Axis ARTPEC-1 | 1999 | (unsupported)         | (unsupported)                |
| Axis ARTPEC-2 | 2003 | (unsupported)         | (unsupported)                |
| Axis ARTPEC-3 | 2007 | (unsupported)         | (unsupported)                |
| Ambarella A5S | 2010 | `armv6`               | `arm-axis-linux-gnueabi`     |
| Axis ARTPEC-4 | 2011 | `mips`                | `mipsel-axis-linux-gnu`      |
| Ambarella S2  | 2012 | `armv7`               | `armv7-axis-linux-gnueabi`   |
| Ambarella S2E | 2012 | `armv7hf`             | `armv7-axis-linux-gnueabihf` |
| Ambarella S2L | 2012 | `armv7hf`             | `armv7-axis-linux-gnueabihf` |
| Axis ARTPEC-5 | 2013 | `mips`                | `mipsel-axis-linux-gnu`      |
| NXP i.MX 8 QP | 2013 | `aarch64`             | `aarch64-axis-linux-gnu`     |
| Ambarella S3L | 2014 | `armv7hf`             | `armv7-axis-linux-gnueabihf` |
| Ambarella S5  | 2016 | `aarch64`             | `aarch64-axis-linux-gnu`     |
| Ambarella S5L | 2016 | `aarch64`             | `aarch64-axis-linux-gnu`     |
| Hi3516C V300  | 2016 | `armv5tej`            | `armv5te-axis-linux-gnueabi` |
| Hi3719C V100  | 2016 | `armv7hf`             | `armv7-axis-linux-gnueabihf` |
| Axis ARTPEC-6 | 2017 | `armv7hf`             | `armv7-axis-linux-gnueabihf` |
| Axis ARTPEC-7 | 2019 | `armv7hf`             | `armv7-axis-linux-gnueabihf` |

The [AXIS product interface guide](https://www.axis.com/en-us/developer-community/product-interface-guide) describes
many hardware configurations in detail, though it is incomplete. Check your device's [`root.Properties.System.Soc` and
`.Architecture` parameter](http://0.0.0.0/axis-cgi/param.cgi?action=list&group=root.Properties.System) to see what your
specific device uses.

`cargo acap build` enables every target by default. This can be overridden in `Cargo.toml`:

```toml
[package.metadata.acap]
targets = ["aarch64", "armv7", "armv7hf"]
```
