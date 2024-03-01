use std::fmt::Display;
use enum_as_inner::EnumAsInner;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, EnumAsInner)]
pub enum Type {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),

    Bool(bool),

    Vector(Box<Type>, Vec<Type>),

    Struct(Vec<Type>),

    Reference(bool, Box<Type>),

    Function(String, Vec<Type>, Option<Box<Type>>),
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
                    if buffer.len() > 0 {
                        write!(f, "Vector(U8, [{}])", String::from_utf8_lossy(&buffer))
                    } else {
                        write!(f, "Vector(U8)")
                    }
                }
                _ => todo!(),
            },
            Type::Struct(types) => {
                if types.is_empty() {
                    write!(f, "Struct([])")
                } else {
                    write!(f, "Struct([ ").unwrap();
                    for (i, t) in types.iter().enumerate() {
                        eprintln!("{:?}", t);
                        write!(f, "{}", t).unwrap();
                        if i != types.len() - 1 {
                            write!(f, ", ").unwrap();
                        }
                    }
                    write!(f, " ])")
                }
            }
            Type::Reference(b, t) => write!(f, "Reference({}, {})", b, *t),
            _ => unimplemented!(),
        }
    }
}

pub struct Parameters(pub Vec<Type>);

impl Display for Parameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(f, "[]")
        } else {
            write!(f, "[ ").unwrap();
            for (i, v) in self.0.clone().iter().enumerate() {
                write!(f, "{}", v).unwrap();
                if i != self.0.len() - 1 {
                    write!(f, ", ").unwrap();
                }
            }
            write!(f, " ]")
        }
    }
}
