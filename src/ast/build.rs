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
use super::reroot_path;
use super::compiled_ast::CompiledAst;
use super::config::AstBuildConfig;

/// BuildPlan
use petgraph::algo::toposort;
struct AstBuildPlan {
    root: PackageName,
    _sorted_deps: Vec<PackageName>,
    resolution_graph: ResolvedGraph,
}

impl AstBuildPlan {
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

    fn compile_ast(&self) -> Result<CompiledAst> {
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

        let compiled = CompiledAst::build_all(
            root_package.clone(),
            transitive_dependencies,
            &self.resolution_graph,
        )?;

        Ok(compiled)
    }
}

/// BuildConfig
use super::lock::PackageLock;
trait TraitAstBuildConfig {
    fn compile_ast<W: Write>(self, path: &Path, writer: &mut W) -> Result<CompiledAst>;
}

impl TraitAstBuildConfig for BuildConfig {
    fn compile_ast<W: Write>(self, path: &Path, writer: &mut W) -> Result<CompiledAst> {
        let resolved_graph = self.resolution_graph_for_package(path, writer)?;
        let mutx = PackageLock::lock();
        let ret = AstBuildPlan::create(resolved_graph)?.compile_ast();
        mutx.unlock();
        ret
    }
}

/// Main
pub fn ast(path: Option<PathBuf>, config: AstBuildConfig) -> Result<CompiledAst> {
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

pub fn main(path: Option<PathBuf>, config: AstBuildConfig) -> Result<CompiledAst> {
    match ast(path, config) {
        Ok(ast) => {

            let data =format!("{:#?}", ast);
            // let data: String = format!("{:#?}", ast.full_ast.parser);

            // println!("{data}");

            let p = std::path::Path::new("/Users/edz/Documents/GitLab/move-ast/output").join("ast.json");
            let mut writer = std::fs::File::create(p).expect("Open error: {p}");
            writer.write(data.as_bytes()).unwrap();

            Ok(ast)
        },
        Err(err) => {
            // let mut w = std::io::stderr();
            // w.write(buf)
            // eprintln!("{:#?}", err);
            // std::process::exit(1);
            Err(err)
        }
    }
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
