use std::io;

use crate::frame::{FrameReader, FrameWriter};
use crate::io::Serializable;

use super::data_packet::DataPacket;
use super::message_type::MessageType;

#[derive(Debug, PartialEq, Clone)]
pub struct UnicastData {
    pub client_id: String,
    pub topic: String,
    pub data_packets: Vec<DataPacket>,
}

impl UnicastData {
    pub fn message_type(&self) -> MessageType {
        MessageType::UnicastData
    }

    pub fn read(reader: &mut FrameReader) -> io::Result<UnicastData> {
        Ok(UnicastData {
            client_id: String::read(reader)?,
            topic: String::read(reader)?,
            data_packets: Vec::<DataPacket>::read(reader)?,
        })
    }

    pub fn write(&self, writer: &mut FrameWriter) -> io::Result<()> {
        (&self.client_id).write(writer)?;
        (&self.topic).write(writer)?;
        (&self.data_packets).write(writer)?;
        Ok(())
    }
}
