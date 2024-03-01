use std::fs::File;
use std::io::Read;
use std::path::Path;
use itertools::Itertools;
use move_binary_format::access::ModuleAccess;
use move_binary_format::file_format::{FunctionDefinitionIndex, StructDefinitionIndex};
use move_binary_format::CompiledModule;
use move_core_types::account_address::AccountAddress;
use move_core_types::runtime_value::MoveValue;
use move_model::addr_to_big_uint;
use move_model::ast::ModuleName;
use move_model::model::FunId;
use move_model::model::FunctionData;
use move_model::model::GlobalEnv;
use move_model::model::Loc;
use move_model::model::ModuleData;
use move_model::model::ModuleId as ModelModuleId;
use move_model::model::StructId;
use move_model::symbol::SymbolPool;
use move_model::ty::{PrimitiveType, Type};
use move_bytecode_utils::Modules;
use move_package::compilation::model_builder::ModelBuilder;
use move_package::BuildConfig;
use move_package::ModelConfig;
use crate::mutator::types::Type as FuzzerType;

/// From https://github.com/kunalabs-io/sui-client-gen
pub fn add_modules_to_model<'a>(
    env: &mut GlobalEnv,
    modules: impl IntoIterator<Item = &'a CompiledModule>,
) {
    for (i, m) in modules.into_iter().enumerate() {
        let id = m.self_id();
        let addr = addr_to_big_uint(id.address());
        let module_name = ModuleName::new(addr, env.symbol_pool().make(id.name().as_str()));
        let module_id = ModelModuleId::new(i);
        let mut module_data = ModuleData::stub(module_name.clone(), module_id, m.clone());

        // add functions
        for (i, def) in m.function_defs().iter().enumerate() {
            let def_idx = FunctionDefinitionIndex(i as u16);
            let name = m.identifier_at(m.function_handle_at(def.function).name);
            let symbol = env.symbol_pool().make(name.as_str());
            let fun_id = FunId::new(symbol);
            let data = FunctionData::stub(symbol, def_idx, def.function);
            module_data.function_data.insert(fun_id, data);
            module_data.function_idx_to_id.insert(def_idx, fun_id);
        }

        // add structs
        for (i, def) in m.struct_defs().iter().enumerate() {
            let def_idx = StructDefinitionIndex(i as u16);
            let name = m.identifier_at(m.struct_handle_at(def.struct_handle).name);
            let symbol = env.symbol_pool().make(name.as_str());
            let struct_id = StructId::new(symbol);
            let data =
                env.create_move_struct_data(m, def_idx, symbol, Loc::default(), Vec::default());
            module_data.struct_data.insert(struct_id, data);
            module_data.struct_idx_to_id.insert(def_idx, struct_id);
        }

        env.module_data.push(module_data);
    }
}

impl From<FuzzerType> for Type {
    fn from(value: FuzzerType) -> Self {
        match value {
            FuzzerType::U8(_) => Type::Primitive(PrimitiveType::U8),
            FuzzerType::U16(_) => Type::Primitive(PrimitiveType::U16),
            FuzzerType::U32(_) => Type::Primitive(PrimitiveType::U32),
            FuzzerType::U64(_) => Type::Primitive(PrimitiveType::U64),
            FuzzerType::U128(_) => Type::Primitive(PrimitiveType::U128),
            FuzzerType::Bool(_) => Type::Primitive(PrimitiveType::Bool),
            FuzzerType::Vector(t, _) => Type::Vector(Box::new(Type::from(*t))),
            FuzzerType::Struct(types) => Type::Struct(
                ModelModuleId::new(42),
                StructId::new(SymbolPool::new().make("")),
                types.into_iter().map(|t| Type::from(t)).collect_vec(),
            ),
            FuzzerType::Reference(b, t) => Type::Reference(b, Box::new(Type::from(*t))),
            _ => unimplemented!(),
        }
    }
}

impl From<Type> for FuzzerType {
    fn from(value: Type) -> Self {
        match value {
            Type::Primitive(p) => match p {
                move_model::ty::PrimitiveType::Bool => todo!(),
                move_model::ty::PrimitiveType::U8 => FuzzerType::U8(0),
                move_model::ty::PrimitiveType::U16 => FuzzerType::U16(0),
                move_model::ty::PrimitiveType::U32 => FuzzerType::U32(0),
                move_model::ty::PrimitiveType::U64 => FuzzerType::U64(0),
                move_model::ty::PrimitiveType::U128 => FuzzerType::U128(0),
                move_model::ty::PrimitiveType::U256 => todo!(),
                move_model::ty::PrimitiveType::Address => todo!(),
                move_model::ty::PrimitiveType::Signer => todo!(),
                move_model::ty::PrimitiveType::Num => todo!(),
                move_model::ty::PrimitiveType::Range => todo!(),
                move_model::ty::PrimitiveType::EventStore => todo!(),
            },
            Type::Tuple(_) => todo!(),
            Type::Vector(vec) => FuzzerType::Vector(Box::new(FuzzerType::from(*vec)), vec![]),
            Type::Struct(_, _, types) => {
                FuzzerType::Struct(types.into_iter().map(|t| FuzzerType::from(t)).collect_vec())
            }
            Type::TypeParameter(_) => todo!(),
            Type::Reference(b, t) => FuzzerType::Reference(b, Box::new(FuzzerType::from(*t))),
            Type::Fun(_, _) => todo!(),
            Type::TypeDomain(_) => todo!(),
            Type::ResourceDomain(_, _, _) => todo!(),
            Type::Error => todo!(),
            Type::Var(_) => todo!(),
        }
    }
}

pub fn generate_abi_from_source(
    path: &str,
    module_name: &str,
    function_name: &str,
) -> (Vec<Type>, usize) {
    let params;
    let max_coverage;

    let build_config = BuildConfig {
        skip_fetch_latest_git_deps: true,
        ..Default::default()
    };

    let resolv_graph = build_config
        .resolution_graph_for_package(Path::new(&path), &mut std::io::stderr())
        .unwrap();

    let source_env = ModelBuilder::create(
        resolv_graph,
        ModelConfig {
            all_files_as_targets: false,
            target_filter: None,
        },
    )
    .build_model()
    .unwrap();

    let module_env = source_env
        .get_modules()
        .find(|m| m.matches_name(module_name));

    if let Some(env) = module_env {
        let func = env
            .get_functions()
            .find(|f| f.get_name_str() == function_name);
        if let Some(f) = func {
            max_coverage = f.get_bytecode().len();
            params = f.get_parameters().iter().map(|p| p.1.clone()).collect();
        } else {
            panic!("Could not find target function !");
        }
    } else {
        panic!("Could not find target module {} !", module_name);
    }
    (params, max_coverage)
}

pub fn generate_abi_from_bin(
    module: &CompiledModule,
    module_name: &str,
    function_name: &str,
) -> (Vec<Type>, usize) {
    let params;
    let max_coverage;

    let modules = [module.clone()];
    let module_map = Modules::new(modules.iter());
    let dep_graph = module_map.compute_dependency_graph();
    let topo_order = dep_graph.compute_topological_order().unwrap();

    let mut env = GlobalEnv::new();
    add_modules_to_model(&mut env, topo_order);

    let module_env = env.get_modules().find(|m| m.matches_name(module_name));

    if let Some(env) = module_env {
        let func = env
            .get_functions()
            .find(|f| f.get_name_str() == function_name);
        if let Some(f) = func {
            max_coverage = f.get_bytecode().len();
            params = f.get_parameter_types();
        } else {
            panic!("Could not find target function !");
        }
    } else {
        panic!("Could not find target module !");
    }
    (params, max_coverage)
}

pub fn generate_abi_from_source_starts_with(
    path: &str,
    module_name: &str,
    function_name: &str,
) -> Vec<(String, Vec<Type>)> {

    let build_config = BuildConfig {
        skip_fetch_latest_git_deps: true,
        ..Default::default()
    };

    let resolv_graph = build_config
        .resolution_graph_for_package(Path::new(&path), &mut std::io::stderr())
        .unwrap();

    let source_env = ModelBuilder::create(
        resolv_graph,
        ModelConfig {
            all_files_as_targets: false,
            target_filter: None,
        },
    )
    .build_model()
    .unwrap();

    let module_env = source_env
        .get_modules()
        .find(|m| m.matches_name(module_name));

    let mut functions = vec![];

    if let Some(env) = module_env {
        let funcs = env
            .get_functions()
            .filter(|f| f.get_name_str().starts_with(function_name));
        for f in funcs {
            let params = f.get_parameters().iter().map(|p| p.1.clone()).collect();
            functions.push((f.get_name_str(), params));
        }
    } else {
        panic!("Could not find target module {} !", module_name);
    }
    functions
}

pub fn load_compiled_module(path: &str) -> CompiledModule {
    let mut f = File::open(path).unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap();
    CompiledModule::deserialize_with_defaults(&buffer).unwrap()
}

pub fn get_fuzz_functions_from_bin(path: &str, module_name: &str, prefix: &str) -> Vec<String> {
    let mut functions = vec![];

    let module = load_compiled_module(path);

    let modules = [module.clone()];
    let module_map = Modules::new(modules.iter());
    let dep_graph = module_map.compute_dependency_graph();
    let topo_order = dep_graph.compute_topological_order().unwrap();

    let mut env = GlobalEnv::new();
    add_modules_to_model(&mut env, topo_order);

    let module_env = env.get_modules().find(|m| m.matches_name(module_name));

    if let Some(env) = module_env {
        for f in env.get_functions() {
            if f.get_name_str().starts_with(prefix) {
                functions.push(f.get_full_name_str());
            }
        }
    } else {
        panic!("Could not find target module !");
    }
    functions
}

pub fn generate_inputs(inputs: Vec<FuzzerType>) -> Vec<MoveValue> {
    let mut res = vec![];
    for i in inputs {
        match i {
            FuzzerType::U8(value) => res.push(MoveValue::U8(value)),
            FuzzerType::U16(value) => res.push(MoveValue::U16(value)),
            FuzzerType::U32(value) => res.push(MoveValue::U32(value)),
            FuzzerType::U64(value) => res.push(MoveValue::U64(value)),
            FuzzerType::U128(value) => res.push(MoveValue::U128(value)),
            FuzzerType::Bool(value) => res.push(MoveValue::Bool(value)),
            FuzzerType::Vector(_, vec) => res.push(MoveValue::Vector(generate_inputs(vec))),
            FuzzerType::Struct(values) => res.push(MoveValue::Struct(
                move_core_types::runtime_value::MoveStruct(generate_inputs(values)),
            )),
            // Use address as reference for now
            FuzzerType::Reference(_, _) => res.push(MoveValue::Address(AccountAddress::random())),
            _ => unimplemented!(),
        }
    }
    res
}
