#!/bin/sh -e

# The rustc_target/spec directory has moved
# Find it in the current checkout
target_specs=
for dir in src/librustc_target/spec compiler/rustc_target/spec
do
  [ -d "$dir" ] && target_specs="$dir"
done

if [ -z "$target_specs" ]
then
  echo "rustc_target could not be found" >&2
  exit 1
fi

# mipsisa32r2el_axis_linux_gnu is mipsel_unknown_linux_gnu, but with target features mips32r2 and soft-float
cp $target_specs/mipsel_unknown_linux_gnu.rs $target_specs/mipsisa32r2el_axis_linux_gnu.rs
sed -i -e 's/features: ".*"/features: "+mips32r2,+soft-float"/' $target_specs/mipsisa32r2el_axis_linux_gnu.rs

# Make sure the target actually contains "soft-float"
grep -c soft-float $target_specs/mipsisa32r2el_axis_linux_gnu.rs >/dev/null || (
  echo "couldn't define features at $target_specs/mipsisa32r2el_axis_linux_gnu.rs" >&2
  echo
  cat $target_specs/mipsisa32r2el_axis_linux_gnu.rs
  exit 1
)

# Add to the list of supported targets
sed -i -e 's/supported_targets! {/supported_targets! { ("mipsisa32r2el-axis-linux-gnu", mipsisa32r2el_axis_linux_gnu),/' \
  $target_specs/mod.rs
