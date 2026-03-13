use haya_ident::Ident;

#[derive(Clone, Serialize, Deserialize)]
pub struct AttributeModifier<'a> {
    pub id: Ident<'a>,
    pub amount: f64,
    pub operation: AttributeOperation,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum AttributeOperation {
    AddValue,
    AddMultipliedBase,
    AddMultipliedTotal,
}
