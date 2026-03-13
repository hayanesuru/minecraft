use haya_collection::List;
use mser::Utf8;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct GameProfileRef<'a> {
    pub id: Uuid,
    pub name: Utf8<'a, 16>,
    pub properties: List<'a, PropertyRef<'a>, 16>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct PropertyRef<'a> {
    pub name: Utf8<'a, 64>,
    pub value: Utf8<'a, 32767>,
    pub signature: Option<Utf8<'a, 1024>>,
}
