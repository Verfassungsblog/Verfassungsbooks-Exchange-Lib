use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use crate::export_formats::ExportFormat;
use crate::projects::PreparedProject;

pub mod certs;
pub mod projects;
pub mod export_formats;

#[derive(bincode::Decode, bincode::Encode)]
pub struct Message{
    /// [hostname]:[port], used to respond later, e.g. if rendering request finished
    pub return_host: String,
    pub message_type: MessageType,
}

#[derive(bincode::Decode, bincode::Encode)]
pub enum MessageType{
    RenderingRequest(RenderingRequest),
    TemplateDataRequest,
    TemplateDataResult,
    RenderingResult
}

#[derive(bincode::Decode, bincode::Encode)]
pub struct RenderingRequest{
    pub prepared_project: PreparedProject,
    #[bincode(with_serde)]
    pub template_id: uuid::Uuid,
    #[bincode(with_serde)]
    pub template_version_id: uuid::Uuid,
    pub export_formats: Vec<ExportFormat>
}

/// Tries to read a message from a TcpStream
/// First reads the length of the message as u64, then reads the next bytes (based on the length)
/// Tries to decode the read bytes into a Message
async fn read_message(mut socket: TcpStream) -> Result<Message, ()>{
    // Read length of message
    let len = match socket.read_u64().await {
        Ok(len) => len as usize,
        Err(_) => {
            // Connection closed
            return Err(());
        }
    };
    // Read message into buffer
    let mut buf = vec![0; len];

    if let Err(e) = socket.read_exact(&mut buf).await{
        eprintln!("Couldn't read into buffer: {}", e);
        return Err(());
    }

    let msg : Message = match bincode::decode_from_slice(&buf, bincode::config::standard()){
        Ok((msg, _)) => msg,
        Err(e) => {
            eprintln!("Couldn't decode Message with bincode: {}", e);
            return Err(())
        }
    };

    Ok(msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
