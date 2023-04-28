use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Default)]
// #[clap(author, version, about)]
pub struct LintConfig {

}