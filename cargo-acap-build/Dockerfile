# We're going to want the AXIS SDKv2 for the MIPS toolchain
# Pull in an image that contains it
FROM slawlor/axis_acap@sha256:c8390b3ef1028f2a8b3bfd690b4c9ddd495473362135095b832ab3deb1851022 AS axis-sdk2

# Start a Rust builder stage
FROM ubuntu:20.04 AS rust

# Start to build Rust, per:
#   https://rustc-dev-guide.rust-lang.org/building/how-to-build-and-run.html

# Install all the Ubuntu dependencies
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y \
  build-essential \
  clang \
  cmake \
  curl \
  g++-aarch64-linux-gnu \
  g++-arm-linux-gnueabihf \
  g++-arm-linux-gnueabi \
  git \
  libc6-dev-arm64-cross \
  libc6-dev-armel-cross \
  libc6-dev-armhf-cross \
  libssl-dev \
  ninja-build \
  pkg-config \
  python3 \
  && rm -rf /var/lib/apt/lists/* \
  && update-alternatives --install /usr/bin/python python /usr/bin/python3 2

# Copy the mipsisa32r2el-axis-linux-gnu toolchain from the v2 SDK
COPY --from=axis-sdk2 /usr/local/mipsisa32r2el /usr/local/mipsisa32r2el

# Clone the selected Rust version
ARG RUST_VERSION
RUN git clone --depth=1 --branch ${RUST_VERSION:-master} --recurse-submodules --shallow-submodules https://github.com/rust-lang/rust.git

# Copy in the Rust config
WORKDIR /rust
ADD rust-config.toml /rust/config.toml

# Define our custom targets
ADD add_targets.sh /tmp
RUN /tmp/add_targets.sh

# Build and install to /usr/local
# (This will take some time.)
# Additionally, clean up our work directory, since we made a lot of files which we don't need any more.
RUN ./x.py install && \
  rm -r /rust

# Prepare the final stage
FROM ubuntu:20.04

# Install some host tools and all the relevant target toolchains
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y \
  build-essential \
  cmake \
  curl \
  g++-aarch64-linux-gnu \
  g++-arm-linux-gnueabihf \
  g++-arm-linux-gnueabi \
  libc6-dev-arm64-cross \
  libc6-dev-armel-cross \
  libc6-dev-armhf-cross \
  pkg-config \
  && rm -rf /var/lib/apt/lists/*

# Grab both Rust and the mipsisa32r2el-axis-linux-gnu toolchain
COPY --from=rust /usr/local/ /usr/local/

# Set up the environment
ENV \
  CC_aarch64_axis_linux_gnu="aarch64-linux-gnu-gcc" \
  CXX_aarch64_axis_linux_gnu="aarch64-linux-gnu-g++" \
  CARGO_TARGET_AARCH64_AXIS_LINUX_GNU_LINKER="aarch64-linux-gnu-gcc" \
  CC_armv5te_axis_linux_gnueabi="arm-linux-gnueabi-gcc" \
  CXX_armv5te_axis_linux_gnueabi="arm-linux-gnueabi-g++" \
  CARGO_TARGET_ARMV5TE_AXIS_LINUX_GNUEABI_LINKER="arm-linux-gnueabi-gcc" \
  CC_arm_axis_linux_gnueabi="arm-linux-gnueabi-gcc" \
  CXX_arm_axis_linux_gnueabi="arm-linux-gnueabi-g++" \
  CARGO_TARGET_ARM_AXIS_LINUX_GNUEABI_LINKER="arm-linux-gnueabi-gcc" \
  CC_armv7_axis_linux_gnueabi="arm-linux-gnueabi-gcc" \
  CXX_armv7_axis_linux_gnueabi="arm-linux-gnueabi-g++" \
  CARGO_TARGET_ARMV7_AXIS_LINUX_GNUEABI_LINKER="arm-linux-gnueabi-gcc" \
  CC_armv7_axis_linux_gnueabihf="arm-linux-gnueabihf-gcc" \
  CXX_armv7_axis_linux_gnueabihf="arm-linux-gnueabihf-g++" \
  CARGO_TARGET_ARMV7_AXIS_LINUX_GNUEABIHF_LINKER="arm-linux-gnueabihf-gcc" \
  CC_mipsel_axis_linux_gnu="mipsisa32r2el-axis-linux-gnu-gcc" \
  CXX_mipsel_axis_linux_gnu="mipsisa32r2el-axis-linux-gnu-g++" \
  CARGO_TARGET_MIPSEL_AXIS_LINUX_GNU_LINKER="mipsisa32r2el-axis-linux-gnu-gcc" \
  PATH="/usr/local/sbin:/usr/local/bin:/usr/local/mipsisa32r2el/r23/bin:/usr/sbin:/usr/bin:/sbin:/bin" \
  PKG_CONFIG_ALLOW_CROSS=1 \
# This hack should become obsolete when this lands: https://github.com/rust-lang/rust/issues/44839
# Needed to build crossbeam-utils which is a common async dependency with 70 million downloads.
# https://crates.io/crates/crossbeam-utils/reverse_dependencies
# As of Rust 1.61, it's currently not possible to do #[cfg(target-feature = "atmoic64")] or similar.
# Therefore crossbeam-utils keeps a blacklist of targets that don't support 64bit atomic operations.
# Since we use custom targets, we're not in the blacklist, and we would fail to compile.
# This problem should go away when the rust RFC lands and when the ecosystem has adopted its use.
  CARGO_TARGET_ARMV5TE_AXIS_LINUX_GNUEABI_RUSTFLAGS="--cfg crossbeam_no_atomic_64" \
  CARGO_TARGET_MIPSEL_AXIS_LINUX_GNU_RUSTFLAGS="--cfg crossbeam_no_atomic_64"
