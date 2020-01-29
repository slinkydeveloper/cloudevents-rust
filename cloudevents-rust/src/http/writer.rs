use super::HttpEvent;
use std::fmt;
use std::error::Error;

pub enum WriterError {
    Other(Box<dyn Error>)
}

impl fmt::Display for WriterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            WriterError::Other(e) => e.fmt(f)
        }
    }
}

impl fmt::Debug for WriterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}

impl Error for WriterError {
    fn cause(&self) -> Option<&dyn Error> {
        return match self {
            WriterError::Other(b) => Some(b.as_ref()),
        }
    }
}

pub trait Writer<R> {
    fn write_cloud_event(res: HttpEvent) -> Result<R, WriterError>;
}
