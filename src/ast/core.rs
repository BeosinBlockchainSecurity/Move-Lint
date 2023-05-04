use std::{
    path::PathBuf,
    collections::{BTreeMap, BTreeSet},
};
use anyhow::Result;
use petgraph::algo::toposort;
use move_command_line_common::env::get_bytecode_version_from_env;
use move_model::{options::ModelBuilderOptions, run_model_builder_with_options};
use move_abigen::{Abigen, AbigenOptions};
use move_package::{
    BuildConfig,
    resolution::resolution_graph::{ResolvedPackage, ResolvedTable, ResolvedGraph, Renaming},
    compilation::compiled_package::CompiledUnitWithSource,
    source_package::parsed_manifest::{SourceManifest, PackageName},
};
use move_symbol_pool::symbol::Symbol;
use move_compiler::{
    shared::{Flags, NamedAddressMap, NumericalAddress, PackagePaths},
    command_line::compiler as CompilerModule,
    diagnostics,
    compiled_unit,
    parser,
    expansion,
    naming,
    typing,
    hlir,
    cfgir,
};

use super::FileSources;

#[derive(Debug, Clone)]
pub struct FullyAst {
    pub parser: parser::ast::Program,
    pub expansion: expansion::ast::Program,
    pub naming: naming::ast::Program,
    pub typing: typing::ast::Program,
    pub hlir: hlir::ast::Program,
    pub cfgir: cfgir::ast::Program,
    pub compiled: Vec<compiled_unit::AnnotatedCompiledUnit>,
}

#[derive(Debug, Clone)]
pub struct SourceInfo {
    pub manifest: SourceManifest,
    pub path: PathBuf,
    pub files: FileSources,
}

#[derive(Debug, Clone)]
pub struct PackageAst {
    pub source_info: SourceInfo,
    pub full_ast: FullyAst,
    /// filename -> json bytes for ScriptABI. Can then be used to generate transaction builders in
    /// various languages.
    abis: Option<Vec<(String, Vec<u8>)>>,
    pub build_options: BuildConfig,
    pub resolution_graph: ResolvedGraph,
}

impl PackageAst {
    pub fn build(
        resolution_graph: ResolvedGraph,
    ) -> Result<Self> {
        resolution_graph.check_cyclic_dependency()?;
        let root_package_path = &resolution_graph.root_package_path;
        let root_package = resolution_graph.get_root_package();

        let (sources_package_paths, deps_package_paths) = resolution_graph.get_root_package_paths()?;
        let mut paths = deps_package_paths;
        paths.push(sources_package_paths);

        let fully_compiled_program = match CompilerModule::construct_pre_compiled_lib(
            paths,
            None,
            Flags::empty(),
        )? {
            Ok(p) => p,
            Err((files, diags)) => {
                diagnostics::report_diagnostics(&files, diags);
            }
        };

        let package_ast = Self {
            source_info: SourceInfo {
                manifest: root_package.source_package.clone(),
                path: if root_package_path.to_str().unwrap_or("").eq(".") {
                    std::env::current_dir().unwrap_or(root_package_path.clone())
                } else {
                    root_package_path.clone()
                },
                files: FileSources::from(fully_compiled_program.files),
            },
            full_ast: FullyAst {
                parser: fully_compiled_program.parser,
                expansion: fully_compiled_program.expansion,
                naming: fully_compiled_program.naming,
                typing: fully_compiled_program.typing,
                hlir: fully_compiled_program.hlir,
                cfgir: fully_compiled_program.cfgir,
                compiled: fully_compiled_program.compiled,
            },
            abis: None,
            build_options: resolution_graph.build_options.clone(),
            resolution_graph,
        };

        Ok(package_ast)
    }

    pub fn abis(&mut self) -> Result<Vec<(String, Vec<u8>)>> {
        if let Some(ret) = &self.abis {
            return Ok(ret.clone());
        }
        let root_package_name = self.resolution_graph.root_package.package.name;
        let (sources_package_paths, deps_package_paths) = self.resolution_graph.get_root_package_paths()?;

        let file_map = &self.source_info.files;
        let all_compiled_units = &self.full_ast.compiled;

        let mut root_compiled_units = vec![];
        let mut deps_compiled_units = vec![];
        for annot_unit in all_compiled_units {
            let source_path = PathBuf::from(file_map[annot_unit.loc().file_hash()].filename());
            let package_name = match &annot_unit {
                compiled_unit::CompiledUnitEnum::Module(m) => m.named_module.package_name.unwrap(),
                compiled_unit::CompiledUnitEnum::Script(s) => s.named_script.package_name.unwrap(),
            };
            let unit = CompiledUnitWithSource {
                unit: annot_unit.clone().into_compiled_unit(),
                source_path,
            };
            if package_name == root_package_name {
                root_compiled_units.push(unit)
            } else {
                deps_compiled_units.push((package_name, unit))
            }
        }

        let model = run_model_builder_with_options(
            vec![sources_package_paths],
            deps_package_paths,
            ModelBuilderOptions::default(),
        )?;
        let bytecode_version = get_bytecode_version_from_env();
        let bytecode_map: BTreeMap<_, _> = root_compiled_units
            .iter()
            .map(|unit| match &unit.unit {
                compiled_unit::CompiledUnit::Script(script) => (
                    script.name.to_string(),
                    unit.unit.serialize(bytecode_version),
                ),
                compiled_unit::CompiledUnit::Module(module) => (
                    module.name.to_string(),
                    unit.unit.serialize(bytecode_version),
                ),
            })
            .collect();
        let abi_options = AbigenOptions {
            in_memory_bytes: Some(bytecode_map),
            output_directory: "".to_string(),
            ..AbigenOptions::default()
        };
        let mut abigen = Abigen::new(&model, &abi_options);
        abigen.gen();
        let abis = abigen.into_result();
        self.abis = Some(abis.clone());
        Ok(abis)
    }
}

pub trait TraitResolvedGraph {
    /// Check all packages for cyclic dependencies
    fn check_cyclic_dependency(&self) -> Result<Vec<PackageName>>;
    /// root_package
    fn get_root_package(&self) -> &ResolvedPackage;
    /// root_package and all the source file paths of all its dependent packages
    fn get_root_package_paths(&self) -> Result<(
        /* sources */ PackagePaths,
        /* deps */ Vec<PackagePaths>,
    )>;
    /// All dependency packages for root_package
    fn get_root_package_transitive_dependencies(&self) -> BTreeSet<PackageName>;
    /// Direct dependency package of root_package
    fn get_root_package_immediate_dependencies(&self) -> BTreeSet<PackageName>;
}

impl TraitResolvedGraph for ResolvedGraph {
    fn check_cyclic_dependency(&self) -> Result<Vec<PackageName>> {
        let mut sorted_deps = match toposort(&self.graph, None) {
            Ok(nodes) => nodes,
            Err(err) => {
                // Is a DAG after resolution otherwise an error should be raised from that.
                anyhow::bail!("IPE: Cyclic dependency found after resolution {:?}", err)
            }
        };
        sorted_deps.reverse();
        return Ok(sorted_deps);
    }

    fn get_root_package(&self) -> &ResolvedPackage {
        &self.package_table[&self.root_package.package.name]
    }

    fn get_root_package_paths(&self) -> Result<(
        /* sources */ PackagePaths,
        /* deps */ Vec<PackagePaths>,
    )> {
        let root_package = self.get_root_package();

        let transitive_dependencies = self.get_root_package_transitive_dependencies()
            .into_iter()
            .map(|package_name| {
                let dep_package = self.package_table.get(&package_name).unwrap();
                let dep_source_paths = dep_package.get_sources(&self.build_options).unwrap();
                // (name, source paths, address mapping)
                (package_name, dep_source_paths, &dep_package.resolution_table)
            })
            .collect();

        make_source_and_deps_for_compiler(&self, root_package, transitive_dependencies)
    }

    fn get_root_package_transitive_dependencies(&self) -> BTreeSet<PackageName> {
        self.get_root_package().transitive_dependencies(&self)
    }

    fn get_root_package_immediate_dependencies(&self) -> BTreeSet<PackageName> {
        self.get_root_package().immediate_dependencies(&self)
    }
}

fn named_address_mapping_for_compiler(
    resolution_table: &ResolvedTable,
) -> BTreeMap<Symbol, NumericalAddress> {
    resolution_table
        .iter()
        .map(|(ident, addr)| {
            let parsed_addr =
                NumericalAddress::new(addr.into_bytes(), move_compiler::shared::NumberFormat::Hex);
            (*ident, parsed_addr)
        })
        .collect::<BTreeMap<_, _>>()
}

fn apply_named_address_renaming(
    current_package_name: Symbol,
    address_resolution: BTreeMap<Symbol, NumericalAddress>,
    renaming: &Renaming,
) -> NamedAddressMap {
    let package_renamings = renaming
        .iter()
        .filter_map(|(rename_to, (package_name, from_name))| {
            if package_name == &current_package_name {
                Some((from_name, *rename_to))
            } else {
                None
            }
        })
        .collect::<BTreeMap<_, _>>();

    address_resolution
        .into_iter()
        .map(|(name, value)| {
            let new_name = package_renamings.get(&name).copied();
            (new_name.unwrap_or(name), value)
        })
        .collect()
}

fn make_source_and_deps_for_compiler(
    resolution_graph: &ResolvedGraph,
    root: &ResolvedPackage,
    deps: Vec<(
        /* name */ Symbol,
        /* source paths */ Vec<Symbol>,
        /* address mapping */ &ResolvedTable,
    )>,
) -> Result<(
    /* sources */ PackagePaths,
    /* deps */ Vec<PackagePaths>,
)> {
    let deps_package_paths = deps
        .into_iter()
        .map(|(name, source_paths, resolved_table)| {
            let paths = source_paths
                .into_iter()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let named_address_map = named_address_mapping_for_compiler(resolved_table);
            Ok(PackagePaths {
                name: Some(name),
                paths,
                named_address_map,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let root_named_addrs = apply_named_address_renaming(
        root.source_package.package.name,
        named_address_mapping_for_compiler(&root.resolution_table),
        &root.renaming,
    );
    let sources = root.get_sources(&resolution_graph.build_options)?;
    let source_package_paths = PackagePaths {
        name: Some(root.source_package.package.name),
        paths: sources,
        named_address_map: root_named_addrs,
    };
    Ok((source_package_paths, deps_package_paths))
}