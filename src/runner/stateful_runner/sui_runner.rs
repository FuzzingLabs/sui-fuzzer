use std::sync::Arc;
use itertools::Itertools;
use move_core_types::runtime_value::MoveValue;
use rand::seq::SliceRandom;
use serde_json::{json, Value as JsonValue};
use sui_core::test_utils::send_and_confirm_transaction;
use sui_json_rpc::transaction_builder_api::AuthorityStateDataReader;
use sui_sdk::json::SuiJsonValue;
use sui_transaction_builder::TransactionBuilder;
use sui_types::{
    base_types::{ObjectID, SuiAddress},
    effects::{TransactionEffects, TransactionEffectsAPI},
    error::SuiError,
    execution_status::{ExecutionFailureStatus, ExecutionStatus},
    object::Owner,
    transaction::TransactionData,
    utils::to_sender_signed_transaction,
    MOVE_STDLIB_PACKAGE_ID, SUI_FRAMEWORK_PACKAGE_ID, SUI_SYSTEM_PACKAGE_ID,
};
use tokio::runtime::Runtime;
use transaction_fuzzer::{
    account_universe::{AccountCurrent, AccountData, PUBLISH_BUDGET},
    executor::Executor,
};
use crate::runner::runner::{Runner, StatefulRunner};
use crate::{
    fuzzer::{coverage::Coverage, error::Error},
    mutator::types::Type as FuzzerType,
    runner::stateless_runner::sui_runner_utils::generate_inputs,
};

pub struct SuiRunner {
    executor: Option<Executor>,
    runtime: Runtime,
    reader: Option<Arc<AuthorityStateDataReader>>,
    account: Option<AccountCurrent>,
    package_id: Option<ObjectID>,
    obj_ids: Vec<ObjectID>,
    target_module: String,
    target_function: Option<FuzzerType>,
    modules: Vec<Vec<u8>>,
}

impl SuiRunner {
    pub fn new(target_module: &str, modules: Vec<Vec<u8>>) -> Self {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let mut s = Self {
            executor: None,
            runtime,
            reader: None,
            account: None,
            package_id: None,
            obj_ids: vec![],
            target_module: target_module.to_string(),
            target_function: None,
            modules,
        };

        s.setup();
        s
    }

    pub fn publish(
        dep_ids: Vec<ObjectID>,
        account: &mut AccountCurrent,
        modules: Vec<Vec<u8>>,
        executor: &mut Executor,
    ) -> TransactionEffects {
        let gas_object = account.new_gas_object(executor);
        let data = TransactionData::new_module(
            account.initial_data.account.address,
            gas_object.compute_object_reference(),
            modules,
            dep_ids,
            PUBLISH_BUDGET,
            1000,
        );
        let txn = to_sender_signed_transaction(data, &account.initial_data.account.key);
        let effects = executor
            .rt
            .block_on(send_and_confirm_transaction(&executor.state, None, txn))
            .unwrap()
            .1
            .into_data();

        assert!(
            matches!(effects.status(), ExecutionStatus::Success { .. }),
            "{:?}",
            effects.status()
        );
        effects
    }

    fn send_transaction(
        &mut self,
        target_function: &str,
        args: Vec<SuiJsonValue>,
    ) -> Result<TransactionEffects, SuiError> {
        let account = self.account.as_mut().unwrap();
        let mut executor = self.executor.as_mut().unwrap();
        let reader = self.reader.as_mut().unwrap();
        let package_id = self.package_id.unwrap();

        let gas_object = account.new_gas_object(&mut executor);
        let transaction_builder = TransactionBuilder::new(reader.clone());
        let transaction = self
            .runtime
            .block_on(transaction_builder.move_call(
                account.initial_data.account.address,
                package_id,
                &self.target_module,
                &target_function,
                vec![],
                args,
                Some(gas_object.id()),
                50000000,
            ))
            .unwrap();

        match self.runtime.block_on(send_and_confirm_transaction(
            &executor.state,
            None,
            to_sender_signed_transaction(transaction, &account.initial_data.account.key),
        )) {
            Ok(res) => Ok(res.1.into_data()),
            Err(err) => Err(err),
        }
    }
}

impl Runner for SuiRunner {
    fn execute(
        &mut self,
        inputs: Vec<FuzzerType>,
    ) -> Result<Option<Coverage>, (Option<Coverage>, Error)> {
        let mut args = vec![];
        let mut inputs = inputs.clone();
        // Removes TxContext reference
        inputs.pop();

        for i in &generate_inputs(inputs) {
            args.push(match i {
                MoveValue::Address(_) => SuiJsonValue::from_object_id(
                    self.obj_ids
                        .choose(&mut rand::thread_rng())
                        .unwrap()
                        .clone(),
                ),
                _ => SuiJsonValue::new(move_value_to_json(i)).unwrap(),
            });
        }

        let response;
        response = self.send_transaction(
            &self
                .target_function
                .clone()
                .unwrap()
                .as_function()
                .unwrap()
                .0,
            args,
        );

        match response {
            Ok(resp) => match resp.status() {
                ExecutionStatus::Success => Ok(None),
                ExecutionStatus::Failure { error, command: _ } => match error {
                    ExecutionFailureStatus::MovePrimitiveRuntimeError(loc) => {
                        if let Some(location) = &loc.0 {
                            let error = Error::Runtime {
                                message: format!(
                                    "in function: {}",
                                    location
                                        .function_name
                                        .clone()
                                        .unwrap_or("Unkown function".to_string())
                                ),
                            };
                            Err((None, error))
                        } else {
                            Err((
                                None,
                                Error::Unknown {
                                    message: error.to_string(),
                                },
                            ))
                        }
                    }
                    ExecutionFailureStatus::MoveAbort(location, abort_code) => {
                        let error = Error::Abort {
                            message: format!(
                                "in function: {} with error code: {}",
                                location
                                    .function_name
                                    .clone()
                                    .unwrap_or("Unkown function".to_string()),
                                abort_code
                            ),
                        };
                        Err((None, error))
                    }
                    _ => Err((
                        None,
                        Error::Unknown {
                            message: error.to_string(),
                        },
                    )),
                },
            },
            Err(err) => Err((
                None,
                Error::Unknown {
                    message: err.to_string(),
                },
            )),
        }
    }

    fn get_target_parameters(&self) -> Vec<FuzzerType> {
        self.target_function
            .clone()
            .unwrap()
            .as_function()
            .unwrap()
            .1
            .clone()
    }

    fn get_target_module(&self) -> String {
        self.target_module.clone()
    }

    fn get_target_function(&self) -> FuzzerType {
        self.target_function.clone().unwrap()
    }

    fn get_max_coverage(&self) -> usize {
        84
    }

    fn set_target_function(&mut self, function: &FuzzerType) {
        self.target_function = Some(function.clone());
    }
}

impl StatefulRunner for SuiRunner {
    fn setup(&mut self) {
        self.obj_ids.clear();

        let mut executor = Executor::new();
        let mut account = AccountCurrent::new(AccountData::new_random());
        let effects = Self::publish(
            vec![
                MOVE_STDLIB_PACKAGE_ID,
                SUI_FRAMEWORK_PACKAGE_ID,
                SUI_SYSTEM_PACKAGE_ID,
            ],
            &mut account,
            self.modules.clone(),
            &mut executor,
        );
        let ((package_id, _, _), _) = effects
            .created()
            .into_iter()
            .find(|(_, owner)| matches!(owner, Owner::Immutable))
            .unwrap();
        let reader = Arc::new(AuthorityStateDataReader::new(executor.state.clone()));
        self.executor = Some(executor);
        self.reader = Some(reader);
        self.account = Some(account);
        self.package_id = Some(package_id);

        let obj_ids = match self.send_transaction("fuzz_init", vec![]) {
            Ok(effects) => effects
                .created()
                .into_iter()
                .map(|((obj_id, _, _), _)| obj_id),
            Err(_) => panic!("Could not init fuzzing !"),
        };
        self.obj_ids.append(&mut obj_ids.collect_vec())
    }
}

fn move_value_to_json(move_value: &MoveValue) -> JsonValue {
    match move_value {
        MoveValue::Vector(values) => JsonValue::Array(
            values
                .iter()
                .map(move_value_to_json)
                .collect::<Vec<JsonValue>>(),
        ),
        MoveValue::Bool(v) => json!(v),
        MoveValue::Signer(v) | MoveValue::Address(v) => json!(SuiAddress::from(*v).to_string()),
        MoveValue::U8(v) => json!(v),
        MoveValue::U64(v) => json!(v.to_string()),
        MoveValue::U128(v) => json!(v.to_string()),
        MoveValue::U16(v) => json!(v),
        MoveValue::U32(v) => json!(v),
        MoveValue::U256(v) => json!(v.to_string()),
        MoveValue::Struct(_) => todo!(),
    }
}
