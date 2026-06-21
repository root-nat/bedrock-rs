use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::io::{Read, Write};

/// An item stack encoded WITHOUT the trailing user-data buffer (NBT / can-place-on / can-destroy /
/// blocking tick) and without a stack network id. This is the form used inside the deprecated
/// `CraftResults` item-stack-request action — see PMMP `CraftResultsDeprecatedStackRequestAction` /
/// `PacketSerializer::getItemStackWithoutUserData` and gophertunnel's item read.
///
/// Wire format: a signed varint network `id`; if `id != 0`, then a little-endian `u16` `count`, an
/// unsigned varint `meta` (aux value), and a signed varint `block_runtime_id`. An `id` of 0 means
/// air / empty and carries no further fields.
#[derive(Clone, Debug)]
pub struct ItemStackWithoutUserData {
    pub id: i32,
    pub count: Option<u16>,
    pub meta: Option<u32>,
    pub block_runtime_id: Option<i32>,
}

impl ItemStackWithoutUserData {
    /// An empty descriptor (air / no item).
    pub fn empty() -> Self {
        Self {
            id: 0,
            count: None,
            meta: None,
            block_runtime_id: None,
        }
    }
}

impl ProtoCodec for ItemStackWithoutUserData {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        ProtoCodecVAR::serialize(&self.id, stream)?;

        if self.id != 0 {
            ProtoCodecLE::serialize(self.count.as_ref().ok_or(ProtoCodecError::ExpectedSome("count"))?, stream)?;
            ProtoCodecVAR::serialize(self.meta.as_ref().ok_or(ProtoCodecError::ExpectedSome("meta"))?, stream)?;
            ProtoCodecVAR::serialize(self.block_runtime_id.as_ref().ok_or(ProtoCodecError::ExpectedSome("block_runtime_id"))?, stream)?;
        }

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let id = <i32 as ProtoCodecVAR>::deserialize(stream)?;

        let (count, meta, block_runtime_id) = if id == 0 {
            (None, None, None)
        } else {
            let count = <u16 as ProtoCodecLE>::deserialize(stream)?;
            let meta = <u32 as ProtoCodecVAR>::deserialize(stream)?;
            let block_runtime_id = <i32 as ProtoCodecVAR>::deserialize(stream)?;

            (Some(count), Some(meta), Some(block_runtime_id))
        };

        Ok(Self {
            id,
            count,
            meta,
            block_runtime_id,
        })
    }

    fn size_hint(&self) -> usize {
        ProtoCodecVAR::size_hint(&self.id)
            + if self.id != 0 {
                self.count.as_ref().map_or(0, ProtoCodecLE::size_hint)
                    + self.meta.as_ref().map_or(0, ProtoCodecVAR::size_hint)
                    + self.block_runtime_id.as_ref().map_or(0, ProtoCodecVAR::size_hint)
            } else {
                0
            }
    }
}
