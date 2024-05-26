use tokio::io::{self,AsyncReadExt,AsyncWriteExt};

use crate::io::Serializable;

use super::data_packet::DataPacket;
use super::message_type::MessageType;

#[derive(Debug, PartialEq, Clone)]
pub struct ForwardedMulticastData {
    pub host: String,
    pub user: String,
    pub topic: String,
    pub content_type: String,
    pub data_packets: Vec<DataPacket>
}

impl ForwardedMulticastData {
    pub fn message_type(&self) -> MessageType {
        MessageType::ForwardedMulticastData
    }

    pub async fn read<R: AsyncReadExt + Unpin>(mut reader: &mut R) -> io::Result<ForwardedMulticastData> {
        Ok(ForwardedMulticastData {
            host: String::read(&mut reader).await?,
            user: String::read(&mut reader).await?,
            topic: String::read(&mut reader).await?,
            content_type: String::read(&mut reader).await?,
            data_packets: Vec::<DataPacket>::read(&mut reader).await?,
        })
    }

    pub async fn write<W: AsyncWriteExt + Unpin>(&self, mut writer: &mut W) -> io::Result<()> {
        (&self.host).write(&mut writer).await?;
        (&self.user).write(&mut writer).await?;
        (&self.topic).write(&mut writer).await?;
        (&self.content_type).write(&mut writer).await?;
        (&self.data_packets).write(&mut writer).await?;
        Ok(())
    }
}
