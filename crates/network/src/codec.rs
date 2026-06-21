use crate::compression::Compression;
use crate::encryption::Encryption;
use crate::error::NetworkCodecError;
use bedrock_protocol_core::{PacketHeader, Packets, ProtoCodecVAR};
use std::io::{Cursor, Read, Write};

pub fn encode_packets<T: Packets>(
    packets: &[T],
    compression: Option<&Compression>,
    encryption: Option<&mut Encryption>,
) -> Result<Vec<u8>, NetworkCodecError> {
    tracing::trace!("Encoding packets");

    let mut packets_stream = batch_packets::<T>(packets)?;
    packets_stream = compress_packets(packets_stream, compression)?;
    packets_stream = encrypt_packets(packets_stream, encryption)?;

    Ok(packets_stream)
}

pub fn decode_packets<T: Packets>(
    mut packets_stream: Vec<u8>,
    compression: Option<&Compression>,
    encryption: Option<&mut Encryption>,
) -> Result<Vec<T>, NetworkCodecError> {
    tracing::trace!("Decoding packets");

    packets_stream = decrypt_packets(packets_stream, encryption)?;
    packets_stream = decompress_packets(packets_stream, compression)?;
    let packets = separate_packets::<T>(packets_stream)?;

    Ok(packets)
}

fn batch_packets<T: Packets>(packets: &[T]) -> Result<Vec<u8>, NetworkCodecError> {
    let mut packets_stream = Vec::new();

    for packet in packets {
        let header = PacketHeader {
            packet_id: packet.id(),
            sender_sub_client_id: 0,
            target_sub_client_id: 0,
        };

        let mut buf = Vec::with_capacity(packet.size_hint(&header));

        // A single packet that fails to serialize must NOT drop the whole batch (the old try_for_each
        // aborted the entire send, silently losing every other packet queued that tick). Skip the bad
        // one, log its id, and ship the rest.
        match packet.serialize(&header, &mut buf) {
            Ok(()) => {
                if packet.id() == 56 || packet.id() == 46 {
                    tracing::info!("[pkt {}] len={} {:02x?}", packet.id(), buf.len(), buf);
                }
                <u32 as ProtoCodecVAR>::serialize(&(buf.len() as u32), &mut packets_stream)?;
                packets_stream.write_all(&buf)?;
            }
            Err(err) => tracing::warn!("skipping packet id {} that failed to serialize: {:?}", packet.id(), err),
        }
    }

    Ok(packets_stream)
}

fn separate_packets<T: Packets>(packets_stream: Vec<u8>) -> Result<Vec<T>, NetworkCodecError> {
    let mut packets_stream = Cursor::new(packets_stream.as_slice());
    let mut packets = vec![];

    loop {
        if packets_stream.position() == packets_stream.get_ref().len() as u64 {
            break;
        }

        let buf_len = <u32 as ProtoCodecVAR>::deserialize(&mut packets_stream)?;
        let mut buf = packets_stream.by_ref().take(buf_len as u64);

        // A single malformed or unmodelled packet must not drop the whole batch — nor, when a nested
        // legacy decoder overflows a varint, panic the connection task. Decode defensively and skip
        // anything that errors or panics; the drain below realigns the cursor on the next length
        // prefix regardless of how far this packet's decoder read.
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| T::deserialize(&mut buf))) {
            Ok(Ok((packet, _))) => packets.push(packet),
            Ok(Err(err)) => tracing::debug!("skipping undecodable packet in batch: {:?}", err),
            Err(_) => tracing::debug!("skipping packet that panicked while decoding"),
        }

        std::io::copy(&mut buf, &mut std::io::sink())?;
    }

    Ok(packets)
}

pub fn compress_packets(
    mut packet_stream: Vec<u8>,
    compression: Option<&Compression>,
) -> Result<Vec<u8>, NetworkCodecError> {
    if let Some(compression) = compression {
        packet_stream = compression.compress(packet_stream)?;
    }

    Ok(packet_stream)
}

pub fn decompress_packets(
    mut packet_stream: Vec<u8>,
    compression: Option<&Compression>,
) -> Result<Vec<u8>, NetworkCodecError> {
    if let Some(compression) = compression {
        packet_stream = compression.decompress(packet_stream)?;
    }

    Ok(packet_stream)
}

pub fn encrypt_packets(
    mut packet_stream: Vec<u8>,
    encryption: Option<&mut Encryption>,
) -> Result<Vec<u8>, NetworkCodecError> {
    if let Some(encryption) = encryption {
        packet_stream = encryption.encrypt(packet_stream)?;
    }

    Ok(packet_stream)
}

pub fn decrypt_packets(
    mut packet_stream: Vec<u8>,
    encryption: Option<&mut Encryption>,
) -> Result<Vec<u8>, NetworkCodecError> {
    if let Some(encryption) = encryption {
        packet_stream = encryption.decrypt(packet_stream)?;
    }

    Ok(packet_stream)
}
