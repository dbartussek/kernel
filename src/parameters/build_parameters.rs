use crate::parameters::config::Config;
use std::{
    fmt::{Display, Formatter},
    path::{Path, PathBuf},
};

pub struct Target {
    pub name: String,
    pub is_custom: bool,
}

impl Display for Target {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            &self.name,
            if self.is_custom { ".json" } else { "" }
        )
    }
}

impl Target {
    pub fn builtin(name: String) -> Self {
        Target {
            name,
            is_custom: false,
        }
    }

    pub fn custom(name: String) -> Self {
        Target {
            name,
            is_custom: true,
        }
    }
}

pub struct BuildParameters {
    pub target: Target,
    pub manifest_directory: Option<PathBuf>,
    pub config: Config,
}

impl BuildParameters {
    pub fn uefi_default() -> Self {
        BuildParameters {
            target: Target::builtin("x86_64-unknown-uefi".to_string()),
            manifest_directory: Some("crates/uefi/uefi_loader".into()),
            config: Default::default(),
        }
    }

    pub fn kernel_default() -> Self {
        BuildParameters {
            target: Target::custom("x86_64-bare".to_string()),
            manifest_directory: Some("crates/kernel/core".into()),
            config: Default::default(),
        }
    }

    pub fn manifest_path(&self) -> Option<PathBuf> {
        self.manifest_directory
            .as_ref()
            .map(|manifest| manifest.join("Cargo.toml"))
    }

    pub fn build_directory(&self) -> PathBuf {
        Path::new("target")
            .join(&self.target.name)
            .join(self.config.to_str())
    }
}
