use binrw::Error as BrwError;
use std::error::Error;
#[cfg(feature = "serde")]
use csv::Error as CsvError;
use xlsxwriter::XlsxError;
use std::fmt::Display;
use std::fmt::Error as FmtError;

#[derive(Debug)]
pub enum BCSVError {
    BrwError(BrwError),
    #[cfg(feature = "serde")]
    CSVError(CsvError),
    XLSXError(XlsxError),
    FmtError(FmtError),
    Other(Box<dyn Error>)
}

impl From<BrwError> for BCSVError {
    fn from(value: BrwError) -> Self {
        Self::BrwError(value)
    }
}

impl From<std::io::Error> for BCSVError {
    fn from(value: std::io::Error) -> Self {
        Self::BrwError(value.into())
    }
}
#[cfg(feature = "serde")]
impl From<CsvError> for BCSVError {
    fn from(value: CsvError) -> Self {
        Self::CSVError(value)
    }
}

impl From<&'static dyn Error> for BCSVError {
    fn from(value: &'static dyn Error) -> Self {
        Self::Other(Box::new(value))
    }
}

impl From<XlsxError> for BCSVError {
    fn from(value: XlsxError) -> Self {
        Self::XLSXError(value)
    }
}

impl<'a> From<&'a str> for BCSVError {
    fn from(value: &'a str) -> Self {
        Self::Other(value.into())
    }
}

impl From<FmtError> for BCSVError {
    fn from(value: FmtError) -> Self {
        Self::FmtError(value)
    }
}

impl Display for BCSVError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BrwError(brw) => Display::fmt(brw, f),
            #[cfg(feature = "serde")]
            Self::CSVError(csv) => Display::fmt(csv, f),
            Self::Other(oth) => Display::fmt(oth, f),
            Self::XLSXError(xlsx) => Display::fmt(xlsx, f),
            Self::FmtError(fmt) => Display::fmt(fmt, f)
        }
    }
}

impl Error for BCSVError {}