use std::io::prelude::*;
use std::io;

use uuid::Uuid;

use crate::io::Serializable;

use super::data_packet::DataPacket;
use super::message_type::MessageType;

#[derive(PartialEq, Debug)]
pub struct UnicastData {
    pub client_id: Uuid,
    pub feed: String,
    pub topic: String,
    pub content_type: String,
    pub data_packets: Vec<DataPacket>
}

impl UnicastData {
    pub fn message_type(&self) -> MessageType {
        MessageType::UnicastData
    }

    pub fn read<R: Read>(mut reader: R) -> io::Result<UnicastData> {
        Ok(UnicastData {
            client_id: Uuid::read(&mut reader)?,
            feed: String::read(&mut reader)?,
            topic: String::read(&mut reader)?,
            content_type: String::read(&mut reader)?,
            data_packets: Vec::<DataPacket>::read(&mut reader)?,
        })
    }

    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        (&self.client_id).write(&mut writer)?;
        (&self.feed).write(&mut writer)?;
        (&self.topic).write(&mut writer)?;
        (&self.content_type).write(&mut writer)?;
        (&self.data_packets).write(&mut writer)?;
        Ok(())
    }
}
