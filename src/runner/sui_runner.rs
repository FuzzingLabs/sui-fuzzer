use move_binary_format::errors::VMError;
use move_binary_format::errors::VMResult;
use move_binary_format::CompiledModule;
use move_bytecode_utils::Modules;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::IdentStr;
use move_core_types::language_storage::ModuleId;
use move_core_types::language_storage::StructTag;
use move_core_types::resolver::LinkageResolver;
use move_core_types::resolver::ModuleResolver;
use move_core_types::resolver::ResourceResolver;
use move_core_types::value::serialize_values;
use move_core_types::value::MoveValue;
use move_core_types::vm_status::StatusCode;
use move_model::model::GlobalEnv;
use move_model::ty::Type;
use move_package::compilation::model_builder::ModelBuilder;
use move_package::BuildConfig;
use move_package::ModelConfig;
use move_vm_runtime::move_vm::MoveVM;
use move_vm_types::gas::UnmeteredGasMeter;
use std::collections::HashMap;
use std::path::Path;

use std::fs::File;
use std::io::prelude::*;

use crate::fuzzer::coverage::{Coverage, CoverageData};
use crate::fuzzer::error::Error;
use crate::mutator::types::Type as FuzzerType;
use crate::runner::runner::Runner;

use super::sui_runner_utils::add_modules_to_model;

#[derive(Clone)]
pub struct RemoteStore {
    modules: HashMap<ModuleId, Vec<u8>>,
}

impl RemoteStore {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn add_module(&mut self, compiled_module: CompiledModule) {
        let id = compiled_module.self_id();
        let mut bytes = vec![];
        compiled_module.serialize(&mut bytes).unwrap();
        self.modules.insert(id, bytes);
    }
}

impl LinkageResolver for RemoteStore {
    type Error = VMError;
}

impl ModuleResolver for RemoteStore {
    type Error = VMError;
    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(self.modules.get(module_id).cloned())
    }
}

impl ResourceResolver for RemoteStore {
    type Error = VMError;

    fn get_resource(
        &self,
        _address: &AccountAddress,
        _tag: &StructTag,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(None)
    }
}

fn combine_signers_and_args(
    signers: Vec<AccountAddress>,
    non_signer_args: Vec<Vec<u8>>,
) -> Vec<Vec<u8>> {
    signers
        .into_iter()
        .map(|s| MoveValue::Signer(s).simple_serialize().unwrap())
        .chain(non_signer_args)
        .collect()
}

pub struct SuiRunner {
    move_vm: MoveVM,
    module: CompiledModule,
    target_function: String,
    pub target_parameters: Vec<FuzzerType>,
}

impl SuiRunner {
    pub fn new(module_path: &str, target_module: &str, target_function: &str) -> Self {
        let move_vm = MoveVM::new(vec![]).unwrap();
        // Loading compiled module
        let mut f = File::open(module_path).unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).unwrap();
        let module = CompiledModule::deserialize_with_defaults(&buffer).unwrap();
        let with_source = true;
        let params = if with_source {
            Self::generate_abi_from_source(&module_path, target_module, target_function)
        } else {
            Self::generate_abi_from_bin(&module, target_module, target_function)
        };
        SuiRunner {
            move_vm,
            module,
            target_function: String::from(target_function),
            target_parameters: Self::transform_params(params),
        }
    }

    fn transform_params(params: Vec<Type>) -> Vec<FuzzerType> {
        let mut res = vec![];
        for param in params {
            res.push(FuzzerType::from(param));
        }
        res
    }

    fn generate_abi_from_source(path: &str, module_name: &str, function_name: &str) -> Vec<Type> {
        let params;

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
                params = f.get_parameters().iter().map(|p| p.1.clone()).collect();
            } else {
                panic!("Could not find target function !");
            }
        } else {
            panic!("Could not find target module !");
        }
        params
    }

    fn generate_abi_from_bin(
        module: &CompiledModule,
        module_name: &str,
        function_name: &str,
    ) -> Vec<Type> {
        let params;

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
                params = f.get_parameters().iter().map(|p| p.1.clone()).collect();
            } else {
                panic!("Could not find target function !");
            }
        } else {
            panic!("Could not find target module !");
        }
        params
    }

    fn create_coverage(inputs: Vec<FuzzerType>, cov: Vec<u16>) -> Coverage {
        let mut coverage_data = vec![];
        for c in cov {
            coverage_data.push(CoverageData { pc: c as u64 });
        }
        Coverage {
            inputs,
            data: coverage_data,
        }
    }

    fn generate_inputs(inputs: Vec<FuzzerType>) -> Vec<MoveValue> {
        let mut res = vec![];
        for i in inputs {
            match i {
                FuzzerType::U8(value) => res.push(MoveValue::U8(value)),
                FuzzerType::U16(value) => res.push(MoveValue::U16(value)),
                FuzzerType::U32(value) => res.push(MoveValue::U32(value)),
                FuzzerType::U64(value) => res.push(MoveValue::U64(value)),
                FuzzerType::U128(value) => res.push(MoveValue::U128(value)),
                FuzzerType::Bool(value) => res.push(MoveValue::Bool(value)),
                FuzzerType::Vector(_, vec) => {
                    res.push(MoveValue::Vector(Self::generate_inputs(vec)))
                }
            }
        }
        res
    }
}

impl Runner for SuiRunner {
    fn get_target_parameters(&self) -> Vec<FuzzerType> {
        self.target_parameters.clone()
    }

    fn execute(&mut self, inputs: Vec<FuzzerType>) -> Result<Option<Coverage>, (Coverage, Error)> {
        let mut remote_view = RemoteStore::new();
        remote_view.add_module(self.module.clone());
        let mut session = self.move_vm.new_session(&remote_view);

        let mut coverage = vec![];

        let ty_args = vec![]
            .into_iter()
            .map(|tag| session.load_type(&tag))
            .collect::<VMResult<_>>()
            .unwrap();

        let result = session.execute_function_bypass_visibility(
            &self.module.self_id(),
            IdentStr::new(&self.target_function).unwrap(),
            ty_args,
            combine_signers_and_args(
                vec![],
                serialize_values(&Self::generate_inputs(inputs.clone())),
            ),
            &mut UnmeteredGasMeter,
            &mut coverage,
        );

        match result {
            Ok(_values) => Ok(Some(Self::create_coverage(inputs.clone(), coverage))),
            Err(err) => {
                let message = err.message().unwrap().to_string();
                let error = match err.major_status() {
                    StatusCode::ABORTED => Error::Abort { message: message },
                    _ => Error::Unknown { message },
                };
                Err((Self::create_coverage(inputs.clone(), coverage), error))
            }
        }
    }
}
