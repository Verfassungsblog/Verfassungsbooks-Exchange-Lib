use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::{create_dir, create_dir_all};
use std::path::PathBuf;
use std::time::Duration;
use async_recursion::async_recursion;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time;
use tokio_rustls::TlsStream;
use crate::export_formats::ExportFormat;
use crate::projects::PreparedProject;

pub mod certs;
pub mod projects;
pub mod export_formats;

#[derive(bincode::Decode, bincode::Encode)]
pub enum Message{
    RenderingRequest(RenderingRequest),
    TemplateDataRequest(TemplateDataRequest),
    TemplateDataResult(TemplateDataResult),
    RenderingRequestStatus(RenderingStatus),
    CommunicationError(CommunicationError),
    UnexpectedError(String)
}

#[derive(bincode::Decode, bincode::Encode)]
pub struct TemplateDataResult{
    #[bincode(with_serde)]
    pub template_id: uuid::Uuid,
    #[bincode(with_serde)]
    pub template_version_id: uuid::Uuid,
    pub contents: TemplateContents,
    pub export_formats: HashMap<String, ExportFormat>
}

impl TemplateContents{
    pub async fn from_path(path: PathBuf) -> tokio::io::Result<TemplateContents>{
        let contents = recursive_read_dir_async(path).await?;

        Ok(TemplateContents{
            contents,
        })
    }

    /// Writes the template data to the specified path.
    /// If path does not exist, creates it.
    pub async fn to_file(self, dest: PathBuf) -> tokio::io::Result<()>{
        if !&dest.try_exists()? {
            create_dir_all(&dest).unwrap();
        }
        recursive_write_dir_async(dest, self.contents).await?;

        Ok(())
    }
}

#[async_recursion]
pub async fn recursive_read_dir_async(path: PathBuf) -> tokio::io::Result<Vec<FileOrFolder>> {
    let mut contents: Vec<FileOrFolder> = Vec::new();
    let mut entries = tokio::fs::read_dir(path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        let file_name = path.file_name().and_then(OsStr::to_str).map(String::from);
        let file_name = match file_name {
            Some(fname) => fname,
            None => {
                eprintln!("Warning: skipped file because of unreadable file name.");
                continue;
            }
        };

        let metadata = entry.metadata().await?;

        if metadata.is_dir() {
            contents.push(FileOrFolder::Folder(NamedFolder {
                name: file_name,
                contents: recursive_read_dir_async(path).await?
            }));
        } else {
            contents.push(FileOrFolder::File(NamedFile {
                name: file_name,
                content: tokio::fs::read(path).await?
            }));
        }
    }

    Ok(contents)
}

#[async_recursion]
pub async fn recursive_write_dir_async(base_path: PathBuf, contents: Vec<FileOrFolder>) -> tokio::io::Result<()>{
    for entry in contents{
        match entry {
            FileOrFolder::File(file) => {
                let res_path = base_path.join(PathBuf::from(file.name));
                tokio::fs::write(res_path, file.content).await?;
            }
            FileOrFolder::Folder(folder) => {
                let res_path = base_path.join(PathBuf::from(folder.name));
                create_dir(&res_path)?;
                recursive_write_dir_async(res_path, folder.contents).await?;
            }
        }
    }

    Ok(())
}

#[derive(bincode::Decode, bincode::Encode, Debug, PartialEq)]
pub struct TemplateContents{
    pub contents: Vec<FileOrFolder>
}

#[derive(bincode::Decode, bincode::Encode, Debug, PartialEq)]
pub enum FileOrFolder{
    File(NamedFile),
    Folder(NamedFolder)
}

#[derive(bincode::Decode, bincode::Encode, Debug, PartialEq)]
pub struct NamedFolder {
    pub name: String,
    pub contents: Vec<FileOrFolder>
}

#[derive(bincode::Decode, bincode::Encode, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct NamedFile {
    pub name: String,
    pub content: Vec<u8>
}

#[derive(bincode::Decode, bincode::Encode, Debug)]
pub enum CommunicationError{
    /// Received an unexpected message
    UnexpectedMessageType,
    /// template_id and/or template_version_id doesn't match requested one
    WrongTemplateDataSend,
}

#[derive(Default, Serialize, Deserialize, bincode::Decode, bincode::Encode, Clone, Debug)]
pub enum RenderingStatus{
    #[default]
    QueuedOnLocal,
    PreparingOnLocal,
    PreparedOnLocal,
    SendToRenderingServer,
    RequestingTemplate,
    TransmittingTemplate,
    QueuedOnRendering,
    Running,
    Finished(RenderingResult),
    /// Rendering result got saved on local, path to the result file (zip / single file), path to the result folder
    SavedOnLocal(PathBuf, PathBuf),
    Failed(RenderingError),
}

impl Display for RenderingError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            RenderingError::ProjectNotFound => String::from("Couldn't find project to render."),
            RenderingError::ProjectMetadataMissing => String::from("Couldn't find project metadata."),
            RenderingError::ConnectionToRenderingServerFailed => String::from("Couldn't connect to a rendering server."),
            RenderingError::TemplateNotFound => String::from("Couldn't find the projects template."),
            RenderingError::CommunicationError => String::from("Communication Error with rendering server occurred."),
            RenderingError::CouldntLoadHandlebarTemplates(log) => format!("Couldn't register Template: {}", log),
            RenderingError::HandlebarsRenderingFailed(log) => format!("Couldn't render Template: {}", log),
            RenderingError::MissingExpectedFileToKeep(filename, log) => format!("Couldn't find the expected file {} after rendering: {}", filename, log),
            RenderingError::VivliostyleRenderingFailed(log) => format!("Couldn't render PDF with vivliostyle: {}", log),
            RenderingError::PandocConversionFailed(log) => format!("Couldn't convert with pandoc: {}", log),
            RenderingError::NoResultFiles => String::from("No file was transmitted. Check your templates export steps."),
            RenderingError::Other(other) => format!("Error occured: {}", other)
        };
        write!(f, "{}", str)
    }
}

#[derive(Clone, bincode::Encode, bincode::Decode, Serialize, Deserialize, Debug)]
pub struct RenderingResult{
    pub files: Vec<NamedFile>
}

#[derive(Serialize, Deserialize, bincode::Decode, bincode::Encode, Clone, Debug)]
pub enum RenderingError{
    ProjectNotFound,
    ProjectMetadataMissing,
    ConnectionToRenderingServerFailed,
    TemplateNotFound,
    CommunicationError,
    CouldntLoadHandlebarTemplates(String),
    /// Handlebars didn't run successfully, String contains the rendering log
    HandlebarsRenderingFailed(String),
    /// A file that should be present is missing after export step. (String, String) -> (Filepath, RenderingLog)
    MissingExpectedFileToKeep(String, String),
    /// Vivliostyle didn't run sucessfully, String contains the rendering log
    VivliostyleRenderingFailed(String),
    /// Pandoc didn't run successsfully, String contains the rendering log
    PandocConversionFailed(String),
    NoResultFiles,
    Other(String)
}

#[derive(bincode::Decode, bincode::Encode)]
pub struct RenderingRequest{
    /// Random uuid to identify the rendering request
    #[bincode(with_serde)]
    pub request_id: uuid::Uuid,
    /// All contents & metadata of the project as [PreparedProject]
    pub prepared_project: PreparedProject,
    /// Contains files uploaded to the project, especially images from image blocks
    pub project_uploaded_files: Vec<FileOrFolder>,
    /// id of the template the project uses
    #[bincode(with_serde)]
    pub template_id: uuid::Uuid,
    /// id of the version of the template
    #[bincode(with_serde)]
    pub template_version_id: uuid::Uuid,
    /// Export format names to render
    pub export_formats: Vec<String>
}

#[derive(bincode::Decode, bincode::Encode)]
pub struct TemplateDataRequest{
    #[bincode(with_serde)]
    pub template_id: uuid::Uuid,
    #[bincode(with_serde)]
    pub template_version_id: uuid::Uuid,
}

/// Tries to read a message from a TcpStream
/// First reads the length of the message as u64, then reads the next bytes (based on the length)
/// Tries to decode the read bytes via bincode into a Message
/// It waits up to 10 minutes until the connection is cancelled
pub async fn read_message(socket: &mut TlsStream<TcpStream>) -> Result<Message, ()>{
    let timeout = Duration::from_secs(600);
    // Read length of message

    let read_future = socket.read_u64();
    let len = match time::timeout(timeout, read_future).await{
        Ok(Ok(len)) => len as usize,
        Ok(Err(e)) => {
            eprintln!("Failed to read msg length, {}", e);
            return Err(())
        },
        Err(_) => {
            eprintln!("Read operation timed out.");
            return Err(())
        }
    };

    // Read message into buffer
    let mut buf = vec![0; len];

    let read_future = socket.read_exact(&mut buf);
    match time::timeout(timeout, read_future).await{
        Ok(Err(e)) => {
            eprintln!("Couldn't read into buffer: {}", e);
            return Err(());
        },
        Err(_) => {
            eprintln!("Read operation timed out.");
            return Err(())
        }
        _ => {}
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

/// Tries to send a specified message via the TcpStream
/// First sends the length of the (bincode) encoded message as u64, then sends the encoded message struct
pub async fn send_message(socket: &mut TlsStream<TcpStream>, message: Message) -> Result<(), ()>{
    let encoded_msg = match bincode::encode_to_vec(message, bincode::config::standard()){
        Ok(msg) => msg,
        Err(e) => {
            eprintln!("Couldn't encode Message with bincode: {}", e);
            return Err(())
        }
    };
    let len = encoded_msg.len() as u64;

    // Send length via socket:
    if let Err(e) = socket.write_u64(len).await{
        eprintln!("Couldn't send message length: {}", e);
        return Err(())
    };

    if let Err(e) = socket.write_all(&encoded_msg[..]).await{
        eprintln!("Couldn't send message: {}", e);
        return Err(())
    }

    Ok(())
}