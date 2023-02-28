use std::{
    io::Write,
    path::{Path, PathBuf},
};
use anyhow::Result;
use move_package::{
    Architecture,
    BuildConfig,
    source_package::parsed_manifest::PackageName,
    resolution::resolution_graph::ResolvedGraph,
};
use move_package::source_package::layout::SourcePackageLayout;
use super::core::PackageAst;
use super::config::AstConfig;

/// BuildPlan
use petgraph::algo::toposort;
struct BuildPlan {
    root: PackageName,
    _sorted_deps: Vec<PackageName>,
    resolution_graph: ResolvedGraph,
}

impl BuildPlan {
    fn create(resolution_graph: ResolvedGraph) -> Result<Self> {
        let mut sorted_deps = match toposort(&resolution_graph.graph, None) {
            Ok(nodes) => nodes,
            Err(err) => {
                // Is a DAG after resolution otherwise an error should be raised from that.
                anyhow::bail!("IPE: Cyclic dependency found after resolution {:?}", err)
            }
        };

        sorted_deps.reverse();

        Ok(Self {
            root: resolution_graph.root_package.package.name,
            _sorted_deps: sorted_deps,
            resolution_graph,
        })
    }

    fn compile_ast(&self) -> Result<PackageAst> {
        let root_package = &self.resolution_graph.package_table[&self.root];
        let immediate_dependencies_names =
            root_package.immediate_dependencies(&self.resolution_graph);
        let transitive_dependencies = root_package
            .transitive_dependencies(&self.resolution_graph)
            .into_iter()
            .map(|package_name| {
                let dep_package = self
                    .resolution_graph
                    .package_table
                    .get(&package_name)
                    .unwrap();
                let dep_source_paths = dep_package
                    .get_sources(&self.resolution_graph.build_options)
                    .unwrap();
                (
                    package_name,
                    immediate_dependencies_names.contains(&package_name),
                    dep_source_paths,
                    &dep_package.resolution_table,
                )
            })
            .collect();

        let compiled = PackageAst::build_all(
            root_package.clone(),
            transitive_dependencies,
            &self.resolution_graph,
        )?;

        Ok(compiled)
    }
}

/// BuildConfig
use super::lock::PackageLock;
trait TraitAstConfig {
    fn compile_ast<W: Write>(self, path: &Path, writer: &mut W) -> Result<PackageAst>;
}

impl TraitAstConfig for BuildConfig {
    fn compile_ast<W: Write>(self, path: &Path, writer: &mut W) -> Result<PackageAst> {
        let resolved_graph = self.resolution_graph_for_package(path, writer)?;
        let mutx = PackageLock::lock();
        let ret = BuildPlan::create(resolved_graph)?.compile_ast();
        mutx.unlock();
        ret
    }
}

fn reroot_path(path: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    let path = path.unwrap_or_else(|| PathBuf::from("."));
    // Always root ourselves to the package root, and then compile relative to that.
    let rooted_path = SourcePackageLayout::try_find_root(&path.canonicalize()?)?;
    std::env::set_current_dir(&rooted_path).unwrap();

    Ok(PathBuf::from("."))
}

pub fn build_ast(path: Option<PathBuf>, config: AstConfig) -> Result<PackageAst> {
    let config = config.get_meta();
    let rerooted_path = reroot_path(path)?;
    let architecture = config.architecture.unwrap_or(Architecture::Move);
    match architecture {
        Architecture::Move | Architecture::AsyncMove => {
            return config.compile_ast(&rerooted_path, &mut std::io::stdout());
        }
        Architecture::Ethereum => {
            anyhow::bail!("The Ethereum architecture is not supported.");
        }
    }
}

pub fn _main(path: Option<PathBuf>, config: AstConfig) -> Result<PackageAst> {
    build_ast(path, config).and_then(|ast| {
        let data =format!("{:#?}", ast);
            let p = ast.package_root.join("output").join("ast.json");
            std::fs::create_dir_all(&p.parent().unwrap())?;
            let mut writer = std::fs::File::create(p).expect("Open error: {p}");
            writer.write(data.as_bytes()).unwrap();

            Ok(ast)
    })
}

// pub struct Build;
// impl Build {
//     pub fn execute(self, path: Option<PathBuf>, config: BuildConfig) -> anyhow::Result<()> {
//         let rerooted_path = reroot_path(path)?;
//         if config.fetch_deps_only {
//             let mut config = config;
//             if config.test_mode {
//                 config.dev_mode = true;
//             }
//             config.download_deps_for_package(&rerooted_path, &mut std::io::stdout())?;
//             return Ok(());
//         }
//         let architecture = config.architecture.unwrap_or(Architecture::Move);
// 
//         match architecture {
//             Architecture::Move | Architecture::AsyncMove => {
//                 config.compile_package(&rerooted_path, &mut std::io::stdout())?;
//             }
// 
//             Architecture::Ethereum => {
//                 #[cfg(feature = "evm-backend")]
//                 config.compile_package_evm(&rerooted_path, &mut std::io::stderr())?;
// 
//                 #[cfg(not(feature = "evm-backend"))]
//                 anyhow::bail!("The Ethereum architecture is not supported because move-cli was not compiled with feature flag `evm-backend`.");
//             }
//         }
//         Ok(())
//     }
// }
