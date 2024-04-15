#[derive(PartialEq)]
pub enum TypeKind {
    Mapping,
    Array,
    NaiveStruct,
    Primitive,
}
impl TypeKind {
    pub fn is_iterish(&self) -> bool {
        match self {
            TypeKind::Mapping | TypeKind::Array => true,
            TypeKind::NaiveStruct | TypeKind::Primitive => false,
        }
    }
}
