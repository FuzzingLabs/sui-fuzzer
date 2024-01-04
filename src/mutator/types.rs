use std::fmt::Display;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Type {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),

    Bool(bool),

    Vector(Box<Type>, Vec<Type>),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::U8(_)
            | Type::U16(_)
            | Type::U32(_)
            | Type::U64(_)
            | Type::U128(_)
            | Type::Bool(_) => write!(f, "{:?}", self),
            Type::Vector(t, v) => match **t {
                Type::U8(_) => {
                    let buffer: Vec<u8> = v
                        .iter()
                        .map(|v| {
                            if let Type::U8(a) = v {
                                a.to_owned()
                            } else {
                                todo!()
                            }
                        })
                        .collect();
                    write!(f, "Vector(U8, [{}])", String::from_utf8_lossy(&buffer))
                }
                _ => todo!(),
            },
        }
    }
}

pub struct Parameters(pub Vec<Type>);

impl Display for Parameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ ").unwrap();
        for v in self.0.clone() {
            write!(f, "{}", v).unwrap();
            if v != *self.0.last().unwrap() {
                write!(f, ", ").unwrap();
            }
        }
        write!(f, " ]")
    }
}
