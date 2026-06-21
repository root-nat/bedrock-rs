use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;
use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub struct PackedItemUseLegacyInventoryTransaction<V: ProtoVersion> {
    pub id: i32,
    pub container_slots: Option<Vec<ContainerSlotEntry>>,
    pub actions: Vec<V::InventoryAction>,
    pub action_type: V::ItemUseInventoryTransactionType,
    pub trigger_type: TriggerType,
    pub position: V::NetworkBlockPosition,
    pub face: i32,
    pub slot: i32,
    pub item: V::NetworkItemStackDescriptor,
    pub from_position: (f32, f32, f32),
    pub click_position: (f32, f32, f32),
    pub target_block_id: u32,
    pub predicted_result: u8,
    pub cooldown_state: i8,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ContainerSlotEntry {
    pub container_id: i8,
    pub slots: Vec<i8>,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum TriggerType {
    Unknown = 0,
    PlayerInput = 1,
    SimulationTick = 2,
}

impl<V: ProtoVersion> ProtoCodec for PackedItemUseLegacyInventoryTransaction<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        ProtoCodecVAR::serialize(&self.id, stream)?;
        if self.id < -1 && (self.id & 1) == 0 {
            let vec = self.container_slots.as_ref().ok_or(ProtoCodecError::ExpectedSome("container_slots"))?;
            ProtoCodecVAR::serialize(&(vec.len() as u32), stream)?;
            for entry in vec {
                entry.serialize(stream)?;
            }
        }

        ProtoCodecVAR::serialize(&(self.actions.len() as u32), stream)?;
        for action in &self.actions {
            action.serialize(stream)?;
        }

        self.action_type.serialize(stream)?;
        self.trigger_type.serialize(stream)?;
        self.position.serialize(stream)?;
        ProtoCodecVAR::serialize(&self.face, stream)?;
        ProtoCodecVAR::serialize(&self.slot, stream)?;
        self.item.serialize(stream)?;
        ProtoCodecLE::serialize(&self.from_position, stream)?;
        ProtoCodecLE::serialize(&self.click_position, stream)?;
        ProtoCodecVAR::serialize(&self.target_block_id, stream)?;
        self.predicted_result.serialize(stream)?;
        self.cooldown_state.serialize(stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let id = <i32 as ProtoCodecVAR>::deserialize(stream)?;
        let container_slots = match id < -1 && (id & 1) == 0 {
            true => {
                let len = <u32 as ProtoCodecVAR>::deserialize(stream)?;
                let mut vec = Vec::with_capacity(len.try_into()?);
                for _ in 0..len {
                    vec.push(ContainerSlotEntry::deserialize(stream)?);
                }
                Some(vec)
            }
            false => None,
        };

        let actions = {
            let len = <u32 as ProtoCodecVAR>::deserialize(stream)?;
            let mut vec = Vec::with_capacity(len.try_into()?);
            for _ in 0..len {
                vec.push(V::InventoryAction::deserialize(stream)?);
            }
            vec
        };

        let action_type = V::ItemUseInventoryTransactionType::deserialize(stream)?;
        let trigger_type = TriggerType::deserialize(stream)?;
        let position = V::NetworkBlockPosition::deserialize(stream)?;
        let face = <i32 as ProtoCodecVAR>::deserialize(stream)?;
        let slot = <i32 as ProtoCodecVAR>::deserialize(stream)?;
        let item = V::NetworkItemStackDescriptor::deserialize(stream)?;
        let from_position = <(f32, f32, f32) as ProtoCodecLE>::deserialize(stream)?;
        let click_position = <(f32, f32, f32) as ProtoCodecLE>::deserialize(stream)?;
        let target_block_id = <u32 as ProtoCodecVAR>::deserialize(stream)?;
        let predicted_result = u8::deserialize(stream)?;
        let cooldown_state = i8::deserialize(stream)?;

        Ok(Self {
            id,
            container_slots,
            actions,
            action_type,
            trigger_type,
            position,
            face,
            slot,
            item,
            from_position,
            click_position,
            target_block_id,
            predicted_result,
            cooldown_state,
        })
    }

    fn size_hint(&self) -> usize {
        ProtoCodecVAR::size_hint(&self.id)
            + match self.id < -1 && (self.id & 1) == 0 {
                true => self.container_slots.as_ref().map_or(0, |vec| <u32 as ProtoCodecVAR>::size_hint(&(vec.len() as u32)) + vec.iter().map(|e| e.size_hint()).sum::<usize>()),
                false => 0,
            }
            + <u32 as ProtoCodecVAR>::size_hint(&(self.actions.len() as u32))
            + self.actions.iter().map(|a| a.size_hint()).sum::<usize>()
            + self.action_type.size_hint()
            + self.trigger_type.size_hint()
            + self.position.size_hint()
            + ProtoCodecVAR::size_hint(&self.face)
            + ProtoCodecVAR::size_hint(&self.slot)
            + self.item.size_hint()
            + ProtoCodecLE::size_hint(&self.from_position)
            + ProtoCodecLE::size_hint(&self.click_position)
            + ProtoCodecVAR::size_hint(&self.target_block_id)
            + self.predicted_result.size_hint()
            + self.cooldown_state.size_hint()
    }
}
