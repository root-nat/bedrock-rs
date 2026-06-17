use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct FullContainerName<V: ProtoVersion> {
    container: V::ContainerEnumName,
    #[endianness(le)]
    dynamic_id: Option<i32>,
}

impl<V: ProtoVersion> FullContainerName<V> {
    pub fn new(container: V::ContainerEnumName, dynamic_id: Option<i32>) -> Self {
        Self { container, dynamic_id }
    }
}
