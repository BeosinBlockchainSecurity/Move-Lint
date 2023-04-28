use std::{
    path::PathBuf,
    collections::BTreeMap,
};
use anyhow::{bail, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};

use move_core_types::account_address::AccountAddress;
use move_package::{Architecture, BuildConfig};

#[derive(Debug, Parser, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Default)]
// #[clap(author, version, about)]
pub struct AstConfig {
    /// Compile in 'dev' mode. The 'dev-addresses' and 'dev-dependencies' fields will be used if
    /// this flag is set. This flag is useful for development of packages that expose named
    /// addresses that are not set to a specific value.
    #[clap(name = "dev-mode", short = 'd', long = "dev", global = true, hide = true)]
    pub dev_mode: bool,

    /// Compile in 'test' mode. The 'dev-addresses' and 'dev-dependencies' fields will be used
    /// along with any code in the 'tests' directory.
    #[clap(name = "test-mode", long = "test", global = true, hide = true)]
    pub test_mode: bool,

    /// Generate documentation for packages
    #[clap(name = "generate-docs", long = "doc", global = true, hide = true)]
    pub generate_docs: bool,

    /// Generate ABIs for packages
    #[clap(name = "generate-abis", long = "abi", global = true, hide = true)]
    pub generate_abis: bool,

    /// Installation directory for compiled artifacts. Defaults to current directory.
    #[clap(long = "install-dir", parse(from_os_str), global = true, hide = true)]
    pub install_dir: Option<PathBuf>,

    /// Force recompilation of all packages
    #[clap(name = "force-recompilation", long = "force", global = true, hide = true)]
    pub force_recompilation: bool,

    /// Optional location to save the lock file to, if package resolution succeeds.
    #[clap(skip)]
    pub lock_file: Option<PathBuf>,

    /// Additional named address mapping. Useful for tools in rust
    #[clap(skip)]
    pub additional_named_addresses: BTreeMap<String, AccountAddress>,

    #[clap(long = "arch", global = true, hide = true, parse(try_from_str = Architecture::try_parse_from_str))]
    pub architecture: Option<Architecture>,

    /// Only fetch dependency repos to MOVE_HOME
    #[clap(long = "fetch-deps-only", global = true, hide = true)]
    pub fetch_deps_only: bool,

    /// Skip fetching latest git dependencies
    #[clap(long = "skip-fetch-latest-git-deps", global = true, hide = true)]
    pub skip_fetch_latest_git_deps: bool,
}

impl AstConfig {
    pub fn get_meta(&self) -> BuildConfig {
        // 使用序列化
        // if let Ok(v1) = serde_json::to_value(&self) {
        //     if let Ok(v2) = serde_json::from_value::<BuildConfig>(v1) {
        //         return v2;
        //     }
        // }
        BuildConfig {
            dev_mode: self.dev_mode,
            test_mode: self.test_mode,
            generate_docs: self.generate_docs,
            generate_abis: self.generate_abis,
            install_dir: self.install_dir.clone(),
            force_recompilation: self.force_recompilation,
            lock_file: self.lock_file.clone(),
            additional_named_addresses: self.additional_named_addresses.clone(),
            architecture: self.architecture,
            fetch_deps_only: self.fetch_deps_only,
            skip_fetch_latest_git_deps: self.skip_fetch_latest_git_deps,
        }
    }
}

/// Architecture
use std::array::IntoIter;
trait TraitArchitecture {
    fn all() -> IntoIter<Architecture, 2>;
    fn try_parse_from_str(s: &str) -> Result<Architecture>;
}

impl TraitArchitecture for Architecture {
    fn all() -> IntoIter<Architecture, 2> {
        IntoIterator::into_iter([
            Self::Move,
            Self::AsyncMove,
            #[cfg(feature = "evm-backend")]
            Self::Ethereum,
        ])
    }

    fn try_parse_from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "move" => Self::Move,

            "async-move" => Self::AsyncMove,

            "ethereum" => Self::Ethereum,

            _ => {
                let supported_architectures = Self::all()
                    .map(|arch| format!("\"{}\"", arch))
                    .collect::<Vec<_>>();
                let be = if supported_architectures.len() == 1 {
                    "is"
                } else {
                    "are"
                };
                bail!(
                    "Unrecognized architecture {} -- only {} {} supported",
                    s,
                    supported_architectures.join(", "),
                    be
                )
            }
        })
    }
}
