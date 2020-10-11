use clap::Clap;
use serde::de::{Deserialize, Deserializer};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Clap)]
pub enum Target {
    Aarch64,
    Armv5,
    Armv6,
    Armv7,
    Armv7Hf,
    Mips,
}

impl Target {
    pub fn all() -> &'static [Target] {
        &[
            Target::Aarch64,
            Target::Armv5,
            Target::Armv6,
            Target::Armv7,
            Target::Armv7Hf,
            Target::Mips,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Target::Aarch64 => "aarch64",
            Target::Armv5 => "armv5",
            Target::Armv6 => "armv6",
            Target::Armv7 => "armv7",
            Target::Armv7Hf => "armv7hf",
            Target::Mips => "mips",
        }
    }

    pub fn rust_target_triple(&self) -> &'static str {
        match self {
            Target::Aarch64 => "aarch64-unknown-linux-gnu",
            Target::Armv5 => "armv5te-unknown-linux-gnueabi",
            Target::Armv6 => "arm-unknown-linux-gnueabi",
            Target::Armv7 => "armv7-unknown-linux-gnueabi",
            Target::Armv7Hf => "armv7-unknown-linux-gnueabihf",
            Target::Mips => "mipsisa32r2el-axis-linux-gnu",
        }
    }

    pub fn docker_objcopy_command(&self) -> &'static str {
        match self {
            Target::Aarch64 => "aarch64-linux-gnu-objcopy",
            Target::Armv5 => "arm-linux-gnueabi-objcopy",
            Target::Armv6 => "arm-linux-gnueabi-objcopy",
            Target::Armv7 => "arm-linux-gnueabihf-objcopy",
            Target::Armv7Hf => "arm-linux-gnueabihf-objcopy",
            Target::Mips => "mipsisa32r2el-axis-linux-gnu-objcopy",
        }
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
            .into_iter()
            .find(|arch| arch.name() == s || arch.rust_target_triple() == s)
            .map(|a| *a)
            .ok_or(NoSuchTargetError(s.into()))
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

impl std::fmt::Display for NoSuchTargetError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "no such target: {}\nexpected one of:\n", &self.0)?;
        for arch in Target::all() {
            write!(f, "  * {}\n", arch.name())?;
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Clap)]
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

    pub fn name(&self) -> &'static str {
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

    pub fn target(&self) -> Option<Target> {
        match self {
            SOC::Artpec1 | SOC::Artpec2 | SOC::Artpec3 => None,
            SOC::Artpec4 | SOC::Artpec5 => Some(Target::Mips),
            SOC::Artpec6 | SOC::Artpec7 => Some(Target::Armv7Hf),
            SOC::A5S => Some(Target::Armv6),
            SOC::Hi3516cV300 => Some(Target::Armv5),
            SOC::Hi3719cV100 => Some(Target::Armv7Hf),
            SOC::MX8QP => Some(Target::Aarch64),
            SOC::S2 => Some(Target::Armv7),
            SOC::S2E | SOC::S2L => Some(Target::Armv7Hf),
            SOC::S3L => Some(Target::Armv7Hf),
            SOC::S5 | SOC::S5L => Some(Target::Aarch64),
        }
    }
}
