#!/bin/sh -e

# The rustc_target/spec directory has moved
# Find it in the current checkout
target_specs=
for dir in src/librustc_target/spec compiler/rustc_target/spec compiler/rustc_target/src/spec
do
  [ -d "$dir" ] && target_specs="$dir"
done

if [ -z "$target_specs" ]
then
  echo "rustc_target could not be found" >&2
  exit 1
fi

# mipsel_axis_linux_gnu is mipsel_unknown_linux_gnu, but with target features mips32r2 and soft-float
cp $target_specs/mipsel_unknown_linux_gnu.rs $target_specs/mipsel_axis_linux_gnu.rs
sed -i -e 's/features: ".*"/features: "+mips32r2,+soft-float"/' $target_specs/mipsel_axis_linux_gnu.rs

# Make sure the target actually contains "soft-float"
grep -c soft-float $target_specs/mipsel_axis_linux_gnu.rs >/dev/null || (
  echo "couldn't define features at $target_specs/mipsel_axis_linux_gnu.rs" >&2
  echo
  cat $target_specs/mipsel_axis_linux_gnu.rs
  exit 1
)

# Add to the list of supported targets
sed -i -e 's/supported_targets! {/supported_targets! { ("mipsel-axis-linux-gnu", mipsel_axis_linux_gnu),/' \
  $target_specs/mod.rs

# Shuffle around the other targets
cp $target_specs/aarch64_unknown_linux_gnu.rs     $target_specs/aarch64_axis_linux_gnu.rs
cp $target_specs/armv5te_unknown_linux_gnueabi.rs $target_specs/armv5te_axis_linux_gnueabi.rs
cp $target_specs/arm_unknown_linux_gnueabi.rs     $target_specs/arm_axis_linux_gnueabi.rs
cp $target_specs/armv7_unknown_linux_gnueabi.rs   $target_specs/armv7_axis_linux_gnueabi.rs
cp $target_specs/armv7_unknown_linux_gnueabihf.rs $target_specs/armv7_axis_linux_gnueabihf.rs

sed -i -e 's/supported_targets! {/supported_targets! { ("aarch64-axis-linux-gnu", aarch64_axis_linux_gnu),/' \
  $target_specs/mod.rs
sed -i -e 's/supported_targets! {/supported_targets! { ("armv5te-axis-linux-gnueabi", armv5te_axis_linux_gnueabi),/' \
  $target_specs/mod.rs
sed -i -e 's/supported_targets! {/supported_targets! { ("arm-axis-linux-gnueabi", arm_axis_linux_gnueabi),/' \
  $target_specs/mod.rs
sed -i -e 's/supported_targets! {/supported_targets! { ("armv7-axis-linux-gnueabi", armv7_axis_linux_gnueabi),/' \
  $target_specs/mod.rs
sed -i -e 's/supported_targets! {/supported_targets! { ("armv7-axis-linux-gnueabihf", armv7_axis_linux_gnueabihf),/' \
  $target_specs/mod.rs
