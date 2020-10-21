use clap::Clap;
use serde::de::{Deserialize, Deserializer};
use std::convert::TryFrom;
use std::str::FromStr;

pub use vapix::v3::application::{Architecture, SOC};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Clap)]
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
