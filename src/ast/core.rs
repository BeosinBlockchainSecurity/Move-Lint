use std::{
    path::PathBuf,
    collections::{BTreeMap, BTreeSet},
};
use anyhow::Result;
use move_command_line_common::{
    env::get_bytecode_version_from_env,
};
use move_model::{model::GlobalEnv, options::ModelBuilderOptions, run_model_builder_with_options};
use move_abigen::{Abigen, AbigenOptions};
use move_package::{
    resolution::resolution_graph::{ResolvedPackage, ResolvedTable, ResolvedGraph, Renaming},
    compilation::{
        compiled_package::{CompiledPackageInfo, CompiledUnitWithSource},
    },
};
use move_symbol_pool::symbol::Symbol;
use move_compiler::{
    shared::{Flags, NamedAddressMap, NumericalAddress, PackagePaths},
    command_line::compiler::{
        self as CompilerModule,
    },
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

/// CompiledPackage
#[derive(Debug, Clone)]
pub struct PackageAst {
    pub package_root: PathBuf,
    pub install_root: PathBuf,
    pub files: FileSources,
    pub package_info: CompiledPackageInfo,
    pub full_ast: FullyAst,
    /// filename -> json bytes for ScriptABI. Can then be used to generate transaction builders in
    /// various languages.
    pub abis: Option<Vec<(String, Vec<u8>)>>,
}

impl PackageAst {
    pub fn build_all(
        resolved_package: ResolvedPackage,
        transitive_dependencies: Vec<(
            /* name */ Symbol,
            /* is immediate */ bool,
            /* source paths */ Vec<Symbol>,
            /* address mapping */ &ResolvedTable,
        )>,
        resolution_graph: &ResolvedGraph,
    ) -> Result<Self> {
        let transitive_dependencies = transitive_dependencies
            .into_iter()
            .map(|(name, _is_immediate, source_paths, address_mapping)| {
                (name, source_paths, address_mapping)
            })
            .collect::<Vec<_>>();
        let root_package_name = resolved_package.source_package.package.name;

        // gather source/dep files with their address mappings
        let (sources_package_paths, deps_package_paths) = make_source_and_deps_for_compiler(
            resolution_graph,
            &resolved_package,
            transitive_dependencies,
        )?;
        let mut paths = deps_package_paths.clone();
        paths.push(sources_package_paths.clone());

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
        let file_map = &fully_compiled_program.files;
        let all_compiled_units = &fully_compiled_program.compiled;

        let mut root_compiled_units = vec![];
        let mut deps_compiled_units = vec![];
        for annot_unit in all_compiled_units {
            let source_path = PathBuf::from(file_map[&annot_unit.loc().file_hash()].0.as_str());
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

        let mut compiled_abis = None;
        if resolution_graph.build_options.generate_abis {
            let model = run_model_builder_with_options(
                vec![sources_package_paths],
                deps_package_paths,
                ModelBuilderOptions::default(),
            )?;

            if resolution_graph.build_options.generate_abis {
                compiled_abis = Some(Self::build_abis(
                    get_bytecode_version_from_env(),
                    &model,
                    &root_compiled_units,
                ));
            }
        };

        let package_ast = Self {
            package_root: if resolution_graph.root_package_path.to_str().unwrap_or("").eq(".") {
                std::env::current_dir().unwrap_or(resolution_graph.root_package_path.clone())
            } else {
                resolution_graph.root_package_path.clone()
            },
            install_root: match &resolution_graph.build_options.install_dir {
                Some(under_path) => under_path.clone(),
                None => resolution_graph.root_package_path.clone(),
            },
            files: FileSources::from(fully_compiled_program.files),
            package_info: CompiledPackageInfo {
                package_name: resolved_package.source_package.package.name,
                address_alias_instantiation: resolved_package.resolution_table,
                source_digest: Some(resolved_package.source_digest),
                build_flags: resolution_graph.build_options.clone(),
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
            abis: compiled_abis,
        };

        Ok(package_ast)
    }

    fn build_abis(
        bytecode_version: Option<u32>,
        model: &GlobalEnv,
        compiled_units: &[CompiledUnitWithSource],
    ) -> Vec<(String, Vec<u8>)> {
        let bytecode_map: BTreeMap<_, _> = compiled_units
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
        let mut abigen = Abigen::new(model, &abi_options);
        abigen.gen();
        abigen.into_result()
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