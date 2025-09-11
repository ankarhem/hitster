#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pdf(Vec<u8>);

impl Pdf {
    /// Create a new PDF from bytes
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }
}

impl From<Vec<u8>> for Pdf {
    fn from(data: Vec<u8>) -> Self {
        Self(data)
    }
}

impl From<Pdf> for Vec<u8> {
    fn from(pdf: Pdf) -> Self {
        pdf.0
    }
}

impl AsRef<[u8]> for Pdf {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_into_vec() {
        let data = vec![1, 2, 3];
        let pdf: Pdf = data.clone().into();
        let result: Vec<u8> = pdf.into();
        assert_eq!(result, data);
    }
}
