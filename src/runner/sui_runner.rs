use move_core_types::vm_status::StatusCode;
use move_vm_runtime::move_vm::MoveVM;
use move_vm_types::gas::UnmeteredGasMeter;
use move_core_types::value::serialize_values;
use move_core_types::identifier::IdentStr;
use move_binary_format::errors::VMResult;
use move_binary_format::CompiledModule;
use move_core_types::value::MoveValue;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::StructTag;
use move_core_types::language_storage::ModuleId;
use move_core_types::resolver::ResourceResolver;
use move_core_types::resolver::ModuleResolver;
use move_binary_format::errors::VMError;
use move_core_types::resolver::LinkageResolver;
use std::collections::HashMap;

use std::io::prelude::*;
use std::fs::File;

use crate::fuzzer::coverage::{Coverage, CoverageData};
use crate::fuzzer::error::Error;
use crate::runner::runner::Runner;

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
}

impl SuiRunner {

    pub fn new(module_path: &str) -> Self {
        let move_vm = MoveVM::new(vec![]).unwrap();
        // Loading compiled module
        let mut f = File::open(module_path).unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).unwrap();
        let module = CompiledModule::deserialize_with_defaults(&buffer).unwrap();
        SuiRunner {
            move_vm,
            module,
        }
    }

    fn create_coverage(input: Vec<u8>, cov: Vec<u16>) -> Coverage {
        let mut coverage_data = vec![];
        for c in cov {
            coverage_data.push(CoverageData { pc: c as u64 });
        }
        Coverage { input, data: coverage_data }
    }

    fn generate_input(input: Vec<u8>) -> MoveValue {
        let mut res = vec![];
        for i in input {
            res.push(MoveValue::U8(i));
        }
        MoveValue::Vector(res)
    }

}

impl Runner for SuiRunner {
    type InputType = Vec<u8>;

    fn execute(&mut self, input: Self::InputType) -> Result<Option<Coverage>, (Coverage, Error)> {
        let mut remote_view = RemoteStore::new();
        remote_view.add_module(self.module.clone());
        let mut session = self.move_vm.new_session(&remote_view);

        let mut coverage = vec![];

        let ty_args = vec![]
            .into_iter()
            .map(|tag| session.load_type(&tag))
            .collect::<VMResult<_>>().unwrap();

        let result = session.execute_function_bypass_visibility(
            &self.module.self_id(),
            IdentStr::new("fuzzinglabs").unwrap(),
            ty_args,
            combine_signers_and_args(vec![], serialize_values(vec![&Self::generate_input(input.clone())])),
            &mut UnmeteredGasMeter,
            &mut coverage
            );

        match result {
            Ok(_values) => Ok(Some(Self::create_coverage(input.clone(), coverage))),
            Err(err) => {
                let message = err.message().unwrap().to_string();
                let error = match err.major_status() {
                    StatusCode::ABORTED => {
                        Error::Abort{message: message}
                    },
                    _ => {
                        Error::Unknown{message}
                    }
                };
                Err((Self::create_coverage(input.clone(), coverage), error))
            }
        }
    }
}
