#[derive(Debug, Clone)]
pub enum Type {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),

    Bool(bool),

    Vector(Box<Type>, Vec<Type>)
}