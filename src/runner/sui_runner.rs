use move_vm_runtime::move_vm::MoveVM;
use sui_move_build::BuildConfig;
use std::path::PathBuf;
use sui_move_build::CompiledPackage;
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

use crate::fuzzer::coverage::Coverage;
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
    module: CompiledModule
}

impl SuiRunner {

    pub fn new(package_path: &str) -> Self {
        let move_vm = MoveVM::new(vec![]).unwrap();
        let compiled_package = Self::compile_package(package_path);
        let module = compiled_package.get_modules().last().unwrap().clone();
        SuiRunner {
            move_vm,
            module
        }
    }

    fn compile_package(package_path: &str) -> CompiledPackage {
        let config = BuildConfig::default();
        config.build(PathBuf::from(package_path)).unwrap()
    }

    fn create_coverage(cov: Vec<u16>) -> Vec<Coverage> {
        let mut coverage = vec![];
        for c in cov {
            coverage.push(Coverage { pc: c as u64 });
        }
        coverage
    }

}

impl Runner for SuiRunner {

    fn execute(&mut self) -> Result<Option<Vec<Coverage>>, String> {
        let mut remote_view = RemoteStore::new();
        remote_view.add_module(self.module.clone());
        let mut session = self.move_vm.new_session(&remote_view);

        let ty_args = vec![]
            .into_iter()
            .map(|tag| session.load_type(&tag))
            .collect::<VMResult<_>>().unwrap();

        let args = vec![MoveValue::Vector(vec![
                                          MoveValue::U8(0x66u8),
                                          MoveValue::U8(0x75u8),
                                          MoveValue::U8(0x7au8),
                                          MoveValue::U8(0x7au8),
                                          MoveValue::U8(0x69u8),
                                          MoveValue::U8(0x6eu8),
                                          MoveValue::U8(0x67u8),
                                          MoveValue::U8(0x6cu8),
                                          MoveValue::U8(0x61u8),
                                          MoveValue::U8(0x62u8),
                                          MoveValue::U8(0x73u8)
        ])];
        
        let mut coverage = vec![];

        let result = session.execute_function_bypass_visibility(
            &self.module.self_id(),
            IdentStr::new("fuzzinglabs").unwrap(),
            ty_args,
            combine_signers_and_args(vec![], serialize_values(&args)),
            &mut UnmeteredGasMeter,
            &mut coverage
            );
        match result {
            Ok(_values) => Ok(Some(Self::create_coverage(coverage))),
            Err(err) => Err(err.message().unwrap().to_string())
        }
    }
}
