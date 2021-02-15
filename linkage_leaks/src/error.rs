use std::fmt;

#[derive(Debug)]
pub enum HttpError{
    ResponseError,
    ParseError,
}

impl std::error::Error for HttpError{}


impl fmt::Display for HttpError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        match self{
            HttpError::ResponseError => write!(f , "Error contacting ipleak.net"),
            HttpError::ParseError => write!(f,"Error getting the DNS server")
        }
    }
}