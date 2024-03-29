use move_binary_format::errors::VMError;
use move_binary_format::errors::VMResult;
use move_binary_format::CompiledModule;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::IdentStr;
use move_core_types::language_storage::ModuleId;
use move_core_types::language_storage::StructTag;
use move_core_types::resolver::LinkageResolver;
use move_core_types::resolver::ModuleResolver;
use move_core_types::resolver::ResourceResolver;
use move_core_types::runtime_value::serialize_values;
use move_core_types::runtime_value::MoveValue;
use move_core_types::vm_status::StatusCode;
use move_model::ty::Type;
use move_vm_config::runtime::VMConfig;
use move_vm_runtime::move_vm::MoveVM;
use move_vm_types::gas::UnmeteredGasMeter;
use std::collections::HashMap;
use crate::fuzzer::coverage::{Coverage, CoverageData};
use crate::fuzzer::error::Error;
use crate::mutator::types::Type as FuzzerType;
use crate::runner::runner::Runner;
use super::sui_runner_utils::generate_abi_from_bin;
use super::sui_runner_utils::generate_abi_from_source;
use super::sui_runner_utils::generate_inputs;
use super::sui_runner_utils::load_compiled_module;

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
    target_module: String,
    target_function: FuzzerType,
    max_coverage: usize,
}

impl SuiRunner {
    pub fn new(module_path: &str, target_module: &str, target_function: &str) -> Self {
        let move_vm = MoveVM::new_with_config(vec![], VMConfig::default()).unwrap();
        // Loading compiled module
        let module = load_compiled_module(module_path);
        let with_source = false;
        let params = if with_source {
            generate_abi_from_source(&module_path, target_module, target_function)
        } else {
            generate_abi_from_bin(&module, target_module, target_function)
        };
        SuiRunner {
            move_vm,
            module,
            target_module: String::from(target_module),
            target_function: FuzzerType::Function(
                String::from(target_function),
                Self::transform_params(params.0),
                None,
            ),
            max_coverage: params.1,
        }
    }

    fn transform_params(params: Vec<Type>) -> Vec<FuzzerType> {
        let mut res = vec![];
        for param in params {
            res.push(FuzzerType::from(param));
        }
        res
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
}

impl Runner for SuiRunner {
    fn set_target_function(&mut self, function: &FuzzerType) {
        self.target_function = function.clone();
    }

    fn get_target_parameters(&self) -> Vec<FuzzerType> {
        self.target_function.as_function().unwrap().1.clone()
    }

    fn get_target_module(&self) -> String {
        self.target_module.clone()
    }

    fn get_target_function(&self) -> FuzzerType {
        self.target_function.clone()
    }

    fn get_max_coverage(&self) -> usize {
        self.max_coverage
    }

    fn execute(
        &mut self,
        inputs: Vec<FuzzerType>,
    ) -> Result<Option<Coverage>, (Option<Coverage>, Error)> {
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
            IdentStr::new(&self.target_function.as_function().unwrap().0).unwrap(),
            ty_args,
            combine_signers_and_args(vec![], serialize_values(&generate_inputs(inputs.clone()))),
            &mut UnmeteredGasMeter,
            &mut coverage,
        );

        match result {
            Ok(_values) => Ok(Some(Self::create_coverage(inputs.clone(), coverage))),
            Err(err) => {
                let mut message = String::from("");
                if let Some(m) = err.message() {
                    message = m.to_string();
                }
                let error = match err.major_status() {
                    StatusCode::ABORTED => Error::Abort { message },
                    StatusCode::ARITHMETIC_ERROR => Error::ArithmeticError { message },
                    StatusCode::MEMORY_LIMIT_EXCEEDED => Error::MemoryLimitExceeded { message },
                    StatusCode::OUT_OF_GAS => Error::OutOfGas { message },
                    _ => Error::Unknown { message },
                };
                Err((Some(Self::create_coverage(inputs.clone(), coverage)), error))
            }
        }
    }
}
