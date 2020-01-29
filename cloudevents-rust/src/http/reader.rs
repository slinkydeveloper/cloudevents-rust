use super::HttpEvent;
use std::fmt;
use std::error::Error;

pub enum ReaderError {
    InvalidMetadata {name: String, reason: String},
    InvalidEncoding {content_type: String, reason: String},
    Other(Box<dyn Error>)
}

impl fmt::Display for ReaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            ReaderError::InvalidMetadata {name, reason} => write!(f, "Invalid metadata '{}': {}", name, reason),
            ReaderError::InvalidEncoding {content_type, reason} => write!(f, "Invalid encoding with Content-Type header '{}': {}", content_type, reason),
            ReaderError::Other(e) => e.fmt(f)
        }
    }
}

impl fmt::Debug for ReaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}

impl Error for ReaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        return match self {
            ReaderError::Other(b) => Some(b.as_ref()),
            _ => None
        }
    }
}

pub trait Reader<R> {
    fn read_cloud_event(req: R) -> Result<HttpEvent, ReaderError>;
}
