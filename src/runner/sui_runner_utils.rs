use move_binary_format::access::ModuleAccess;
use move_binary_format::file_format::{FunctionDefinitionIndex, StructDefinitionIndex};
use move_binary_format::CompiledModule;
use move_model::addr_to_big_uint;
use move_model::ast::ModuleName;
use move_model::ast::Spec;
use move_model::model::FunId;
use move_model::model::FunctionData;
use move_model::model::GlobalEnv;
use move_model::model::Loc;
use move_model::model::ModuleData;
use move_model::model::ModuleId as ModelModuleId;
use move_model::model::StructId;
use move_model::ty::{PrimitiveType, Type};

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
            let data = env.create_move_struct_data(
                m,
                def_idx,
                symbol,
                Loc::default(),
                Vec::default(),
                Spec::default(),
            );
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
            FuzzerType::Vector(t, vec) => Type::Vector(Box::new(Type::from(*t))),
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
            Type::Struct(_, _, _) => todo!(),
            Type::TypeParameter(_) => todo!(),
            Type::Reference(_, _) => todo!(),
            Type::Fun(_, _) => todo!(),
            Type::TypeDomain(_) => todo!(),
            Type::ResourceDomain(_, _, _) => todo!(),
            Type::Error => todo!(),
            Type::Var(_) => todo!(),
        }
    }
}
