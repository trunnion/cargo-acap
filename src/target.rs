use clap::Parser;
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::error::Error;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Parser)]
pub enum Target {
    Aarch64,
    Armv5tej,
    Armv6,
    Armv7,
    Armv7Hf,
    Mips,
}

impl Target {
    pub fn all() -> &'static [Target] {
        &[
            Target::Aarch64,
            Target::Armv5tej,
            Target::Armv6,
            Target::Armv7,
            Target::Armv7Hf,
            Target::Mips,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Target::Aarch64 => "aarch64",
            Target::Armv5tej => "armv5tej",
            Target::Armv6 => "armv6",
            Target::Armv7 => "armv7",
            Target::Armv7Hf => "armv7hf",
            Target::Mips => "mips",
        }
    }

    pub fn rust_target_triple(&self) -> &'static str {
        match self {
            Target::Aarch64 => "aarch64-axis-linux-gnu",
            Target::Armv5tej => "armv5te-axis-linux-gnueabi",
            Target::Armv6 => "arm-axis-linux-gnueabi",
            Target::Armv7 => "armv7-axis-linux-gnueabi",
            Target::Armv7Hf => "armv7-axis-linux-gnueabihf",
            Target::Mips => "mipsel-axis-linux-gnu",
        }
    }

    pub fn docker_objcopy_command(&self) -> &'static str {
        match self {
            Target::Aarch64 => "aarch64-linux-gnu-objcopy",
            Target::Armv5tej => "arm-linux-gnueabi-objcopy",
            Target::Armv6 => "arm-linux-gnueabi-objcopy",
            Target::Armv7 => "arm-linux-gnueabihf-objcopy",
            Target::Armv7Hf => "arm-linux-gnueabihf-objcopy",
            Target::Mips => "mipsisa32r2el-axis-linux-gnu-objcopy",
        }
    }
}

/// A system architecture used by an AXIS product.
///
/// This enumeration contains all known architectures. It is `#[non_exhaustive]` since it is
/// expected that AXIS will use additional architectures in the future.
///
/// `Architecture` encodes both a processor instruction set and ABI. For example, the AXIS ARTPEC-7
/// SoC could in principle run `Armv7Hf`, `Armv7`, `Armv6`, or `Armv5tej` software, but only one of
/// these will work in practice because the Linux kernel and `libc` were built for `Armv7Hf`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Architecture {
    /// Aarch 64
    Aarch64,
    /// Arm v5 with Thumb, Enhanced DSP, and Jazelle, in little endian byte order, using the GNU
    /// Arm embedded ABI.
    Armv5tej,
    /// Arm v6 in little endian byte order, using the GNU Arm embedded ABI.
    Armv6,
    /// Arm v7 in little endian byte order, using the GNU Arm embedded ABI.
    Armv7,
    /// Arm v7 with hardware floating point, in little endian byte order, using the GNU Arm embedded
    /// ABI.
    Armv7Hf,
    /// CRIS v0–v10, i.e. chips up to and including ETRAX 100LX and ARTPEC-2.
    ///
    /// These version numbers are defined in the [ETRAX FS Designer's Reference].
    ///
    /// [ETRAX FS Designer's Reference]: https://www.axis.com/files/manuals/etrax_fs_des_ref-070821.pdf
    CrisV0,
    /// CRIS v32, as used in ETRAX FS and ARTPEC-3.
    CrisV32,
    /// MIPS 32-bit revision 2, in little endian byte order.
    Mips,
}

impl From<Target> for Architecture {
    fn from(t: Target) -> Architecture {
        match t {
            Target::Aarch64 => Architecture::Aarch64,
            Target::Armv5tej => Architecture::Armv5tej,
            Target::Armv6 => Architecture::Armv6,
            Target::Armv7 => Architecture::Armv7,
            Target::Armv7Hf => Architecture::Armv7Hf,
            Target::Mips => Architecture::Mips,
        }
    }
}

impl TryFrom<Architecture> for Target {
    type Error = ();

    fn try_from(value: Architecture) -> Result<Self, Self::Error> {
        Ok(match value {
            Architecture::Aarch64 => Target::Aarch64,
            Architecture::Armv5tej => Target::Armv5tej,
            Architecture::Armv6 => Target::Armv6,
            Architecture::Armv7 => Target::Armv7,
            Architecture::Armv7Hf => Target::Armv7Hf,
            Architecture::Mips => Target::Mips,
            _ => return Err(()),
        })
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl std::str::FromStr for Target {
    type Err = NoSuchTargetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Target::all()
            .iter()
            .find(|arch| arch.name() == s || arch.rust_target_triple() == s)
            .copied()
            .ok_or_else(|| NoSuchTargetError(s.into()))
    }
}

impl<'de> Deserialize<'de> for Target {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone)]
pub struct NoSuchTargetError(String);

impl Error for NoSuchTargetError {}

impl std::fmt::Display for NoSuchTargetError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "no such target: {}\nexpected one of:\n", &self.0)?;
        for arch in Target::all() {
            write!(f, "  * {}", arch.name())?;
        }
        Ok(())
    }
}

/// A system-on-chip used by an AXIS product.
///
/// This enumeration contains all known SOCs. It is `#[non_exhaustive]` since it is expected that
/// AXIS will use additional SOCs in the future.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum SOC {
    Artpec1,
    Artpec2,
    Artpec3,
    Artpec4,
    Artpec5,
    Artpec6,
    Artpec7,
    A5S,
    Hi3516cV300,
    Hi3719cV100,
    MX8QP,
    S2,
    S2E,
    S2L,
    S3L,
    S5,
    S5L,
}

impl SOC {
    /// List all SoCs.
    pub fn all() -> &'static [SOC] {
        &[
            SOC::Artpec1,
            SOC::Artpec2,
            SOC::Artpec3,
            SOC::Artpec4,
            SOC::Artpec5,
            SOC::Artpec6,
            SOC::Artpec7,
            SOC::A5S,
            SOC::Hi3516cV300,
            SOC::Hi3719cV100,
            SOC::MX8QP,
            SOC::S2,
            SOC::S2E,
            SOC::S2L,
            SOC::S3L,
            SOC::S5,
            SOC::S5L,
        ]
    }

    /// The display name of this SoC.
    pub fn display_name(&self) -> &'static str {
        match self {
            SOC::Artpec1 => "Axis ARTPEC-1",
            SOC::Artpec2 => "Axis ARTPEC-2",
            SOC::Artpec3 => "Axis ARTPEC-3",
            SOC::Artpec4 => "Axis ARTPEC-4",
            SOC::Artpec5 => "Axis ARTPEC-5",
            SOC::Artpec6 => "Axis ARTPEC-6",
            SOC::Artpec7 => "Axis ARTPEC-7",
            SOC::A5S => "Ambarella A5S",
            SOC::Hi3516cV300 => "Hi3516C V300",
            SOC::Hi3719cV100 => "Hi3719C V100",
            SOC::MX8QP => "NXP i.MX 8 QP",
            SOC::S2 => "Ambarella S2",
            SOC::S2E => "Ambarella S2E",
            SOC::S2L => "Ambarella S2L",
            SOC::S3L => "Ambarella S3L",
            SOC::S5 => "Ambarella S5",
            SOC::S5L => "Ambarella S5L",
        }
    }

    /// The year when this SoC was released.
    pub fn year(&self) -> u32 {
        match self {
            SOC::Artpec1 => 1999,
            SOC::Artpec2 => 2003,
            SOC::Artpec3 => 2007,
            SOC::Artpec4 => 2011,
            SOC::Artpec5 => 2013,
            SOC::Artpec6 => 2017,
            SOC::Artpec7 => 2019,
            SOC::A5S => 2010,
            SOC::Hi3516cV300 => 2016, //?
            SOC::Hi3719cV100 => 2016, //?
            SOC::MX8QP => 2013,
            SOC::S2 | SOC::S2E | SOC::S2L => 2012,
            SOC::S3L => 2014,
            SOC::S5 | SOC::S5L => 2016,
        }
    }

    /// The architecture most commonly used by this SoC.
    ///
    /// In principle, an SoC can support multiple architectures, varying by firmware image. In
    /// practice, Axis has compiled every firmware released for every product using a given SoC with
    /// the same architecture. Still, if you specifically need to know which architecture a given
    /// device is using, you should ask instead of assuming.
    pub fn architecture(&self) -> Result<Target, &'static str> {
        Ok(match self {
            SOC::Artpec1 | SOC::Artpec2 | SOC::Artpec3 => {
                return Err("ARTPEC 1, 2 and 3 use CrisV32, which is not supported")
            }
            SOC::Artpec4 | SOC::Artpec5 => Target::Mips,
            SOC::Artpec6 | SOC::Artpec7 => Target::Armv7Hf,
            SOC::A5S => Target::Armv6,
            SOC::Hi3516cV300 => Target::Armv5tej,
            SOC::Hi3719cV100 => Target::Armv7Hf,
            SOC::MX8QP => Target::Aarch64,
            SOC::S2 => Target::Armv7,
            SOC::S2E | SOC::S2L => Target::Armv7Hf,
            SOC::S3L => Target::Armv7Hf,
            SOC::S5 | SOC::S5L => Target::Aarch64,
        })
    }
}

impl std::fmt::Display for SOC {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.display_name())
    }
}
