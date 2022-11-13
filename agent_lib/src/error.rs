use thiserror::Error;

pub type BoxAgentLibError = Box<dyn std::error::Error + 'static + Send + Sync>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Checksum mismatch expected {expected} but calculated {actual} from {data:?}")]
    ChecksumError {
        expected: u32,
        actual: u32,
        data: Vec<u8>,
    },
}
