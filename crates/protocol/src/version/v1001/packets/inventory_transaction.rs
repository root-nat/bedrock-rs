use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};
use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{ProtoCodec, ProtoCodecVAR};
use std::io::{Read, Write};

// protocol_core models the three trailing fields as Option<_>, and its Option codec writes a 1-byte
// presence bool before each value. So the wire is: raw_id(varint i32) | bool(legacy?) [+ slots] |
// bool(type?) [+ type varu32] | bool(tx?) [+ actions]. When type == ItemUse(2) the per-type UseItem
// data follows (it is NOT part of the generic InventoryTransaction action list): action_type, trigger,
// block position, face, ... — Touch-mode clients place/interact this way (no PlayerAuthInput item_use).
// Only the fields up to `face` are read here; the rest (held item, vec3s, runtime id, prediction) is
// length-framed and drained by the codec.
#[packet(id = 30)]
#[derive(Clone, Debug)]
pub struct InventoryTransactionPacket<V: ProtoVersion> {
    pub raw_id: i32,
    pub legacy_set_item_slots: Option<Vec<LegacySetItemSlotsEntry>>,
    pub transaction_type: Option<u32>,
    pub transaction: Option<V::InventoryTransaction>,
    pub use_item: Option<UseItemData>,
}

#[derive(Clone, Debug)]
pub struct UseItemData {
    pub action_type: u32,
    pub block_x: i32,
    pub block_y: i32,
    pub block_z: i32,
    pub face: i32,
}

impl<V: ProtoVersion> ProtoCodec for InventoryTransactionPacket<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        <i32 as ProtoCodecVAR>::serialize(&self.raw_id, stream)?;
        match &self.legacy_set_item_slots {
            Some(slots) => {
                1u8.serialize(stream)?;
                slots.serialize(stream)?;
            }
            None => 0u8.serialize(stream)?,
        }
        match &self.transaction_type {
            Some(transaction_type) => {
                1u8.serialize(stream)?;
                <u32 as ProtoCodecVAR>::serialize(transaction_type, stream)?;
            }
            None => 0u8.serialize(stream)?,
        }
        match &self.transaction {
            Some(transaction) => {
                1u8.serialize(stream)?;
                transaction.serialize(stream)?;
            }
            None => 0u8.serialize(stream)?,
        }
        if let Some(use_item) = &self.use_item {
            <u32 as ProtoCodecVAR>::serialize(&use_item.action_type, stream)?;
            <i32 as ProtoCodecVAR>::serialize(&0, stream)?;
            <i32 as ProtoCodecVAR>::serialize(&use_item.block_x, stream)?;
            <i32 as ProtoCodecVAR>::serialize(&use_item.block_y, stream)?;
            <i32 as ProtoCodecVAR>::serialize(&use_item.block_z, stream)?;
            <i32 as ProtoCodecVAR>::serialize(&use_item.face, stream)?;
        }
        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let raw_id = <i32 as ProtoCodecVAR>::deserialize(stream)?;

        let legacy_set_item_slots = match u8::deserialize(stream)? != 0 {
            true => Some(Vec::<LegacySetItemSlotsEntry>::deserialize(stream)?),
            false => None,
        };
        let transaction_type = match u8::deserialize(stream)? != 0 {
            true => Some(<u32 as ProtoCodecVAR>::deserialize(stream)?),
            false => None,
        };
        let transaction = match u8::deserialize(stream)? != 0 {
            true => Some(V::InventoryTransaction::deserialize(stream)?),
            false => None,
        };

        let use_item = match transaction_type == Some(2) {
            true => {
                let action_type = <u32 as ProtoCodecVAR>::deserialize(stream)?;
                let _trigger_type = <i32 as ProtoCodecVAR>::deserialize(stream)?;
                let block_x = <i32 as ProtoCodecVAR>::deserialize(stream)?;
                let block_y = <i32 as ProtoCodecVAR>::deserialize(stream)?;
                let block_z = <i32 as ProtoCodecVAR>::deserialize(stream)?;
                let face = <i32 as ProtoCodecVAR>::deserialize(stream)?;
                Some(UseItemData { action_type, block_x, block_y, block_z, face })
            }
            false => None,
        };

        Ok(Self {
            raw_id,
            legacy_set_item_slots,
            transaction_type,
            transaction,
            use_item,
        })
    }

    fn size_hint(&self) -> usize {
        64
    }
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct LegacySetItemSlotsEntry {
    pub container_enum: i8,
    pub slot_vector: Vec<i8>,
}
