use std::{
    io::Write,
    path::{Path, PathBuf},
};
use anyhow::Result;
use move_package::{
    Architecture,
    BuildConfig,
};
use move_package::source_package::layout::SourcePackageLayout;
use super::core::PackageAst;
use super::config::AstConfig;

/// BuildConfig
use super::lock::PackageLock;
trait TraitAstConfig {
    fn compile_ast<W: Write>(self, path: &Path, writer: &mut W) -> Result<PackageAst>;
}

impl TraitAstConfig for BuildConfig {
    fn compile_ast<W: Write>(self, path: &Path, writer: &mut W) -> Result<PackageAst> {
        let resolution_graph = self.resolution_graph_for_package(path, writer)?;
        let mutx = PackageLock::lock();
        let ret = PackageAst::build(resolution_graph);
        mutx.unlock();
        ret
    }
}

fn handle_reroot_path<T, F>(path: Option<PathBuf>, f: F) -> Result<T>
where F: FnOnce(PathBuf) -> Result<T> {
    let path = path.unwrap_or_else(|| PathBuf::from("."));
    // Always root ourselves to the package root, and then compile relative to that.
    let rooted_path = SourcePackageLayout::try_find_root(&path.canonicalize()?)?;
    let pop = std::env::current_dir().unwrap();
    std::env::set_current_dir(&rooted_path).unwrap();
    let ret = f(PathBuf::from("."));
    std::env::set_current_dir(pop).unwrap();
    return ret;
}

pub fn build_ast(path: Option<PathBuf>, config: AstConfig) -> Result<PackageAst> {
    let config = config.get_meta();
    handle_reroot_path(path, |rerooted_path| {
        let architecture = config.architecture.unwrap_or(Architecture::Move);
        match architecture {
            Architecture::Move | Architecture::AsyncMove => {
                return config.compile_ast(&rerooted_path, &mut std::io::stdout());
            }
            Architecture::Ethereum => {
                anyhow::bail!("The Ethereum architecture is not supported.");
            }
        }
    })
}

pub fn _main(path: Option<PathBuf>, config: AstConfig) -> Result<PackageAst> {
    build_ast(path, config).and_then(|ast| {
        let data =format!("{:#?}", ast);
        let p = ast.source_info.path.join("output").join("ast.json");
        std::fs::create_dir_all(&p.parent().unwrap())?;
        let mut writer = std::fs::File::create(p).expect("Open error: {p}");
        writer.write(data.as_bytes()).unwrap();

        Ok(ast)
    })
}
