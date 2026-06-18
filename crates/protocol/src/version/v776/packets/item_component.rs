use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 162)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemComponentPacket<V: ProtoVersion> {
    pub items: Vec<ItemsEntry<V>>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemsEntry<V: ProtoVersion> {
    pub component_item_name: String,
    #[endianness(le)]
    pub runtime_id: i16,
    pub is_component_based: bool,
    pub version: V::ItemVersion,
    #[nbt]
    pub component_data: nbtx::Value,
}

impl<V: ProtoVersion> ItemsEntry<V> {
    /// A vanilla item registry entry with an empty component tag.
    pub fn vanilla(component_item_name: String, runtime_id: i16, is_component_based: bool, version: V::ItemVersion) -> Self {
        Self {
            component_item_name,
            runtime_id,
            is_component_based,
            version,
            component_data: nbtx::Value::Compound(Default::default()),
        }
    }
}
