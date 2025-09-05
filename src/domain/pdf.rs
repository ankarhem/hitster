use std::str::FromStr;

/// PDF side specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfSide {
    /// Front side of the cards
    Front,
    /// Back side of the cards
    Back,
}

impl PdfSide {
    /// Get the PDF side as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            PdfSide::Front => "front",
            PdfSide::Back => "back",
        }
    }
    
    /// Get the PDF side as a lowercase string for URLs
    pub fn as_url_param(&self) -> &'static str {
        self.as_str()
    }
    
    /// Get the opposite side
    pub fn opposite(&self) -> Self {
        match self {
            PdfSide::Front => PdfSide::Back,
            PdfSide::Back => PdfSide::Front,
        }
    }
    
    /// Get all sides
    pub fn all() -> [Self; 2] {
        [PdfSide::Front, PdfSide::Back]
    }
}

impl std::fmt::Display for PdfSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for PdfSide {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "front" | "frontside" => Ok(PdfSide::Front),
            "back" | "backside" => Ok(PdfSide::Back),
            _ => Err(anyhow::anyhow!("Invalid PDF side: {}. Must be 'front' or 'back'", s)),
        }
    }
}

/// PDF data wrapper
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pdf(Vec<u8>);

impl Pdf {
    /// Create a new PDF from bytes
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }
    
    /// Get the PDF data as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    
    /// Get the PDF data as a Vec<u8>
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
    
    /// Get the PDF data as a Vec<u8> (alias for into_bytes)
    pub fn into_vec(self) -> Vec<u8> {
        self.into_bytes()
    }
    
    /// Get the length of the PDF data
    pub fn len(&self) -> usize {
        self.0.len()
    }
    
    /// Check if the PDF is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    
    /// Create a PDF from a file path
    pub async fn from_file(path: &std::path::Path) -> Result<Self, anyhow::Error> {
        let data = tokio::fs::read(path).await?;
        Ok(Self(data))
    }
    
    /// Save the PDF to a file
    pub async fn to_file(&self, path: &std::path::Path) -> Result<(), anyhow::Error> {
        tokio::fs::write(path, &self.0).await?;
        Ok(())
    }
    
    /// Get the PDF size in a human-readable format
    pub fn size_string(&self) -> String {
        let bytes = self.len();
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        }
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
    fn test_pdf_side_from_str() {
        assert_eq!(PdfSide::from_str("front").unwrap(), PdfSide::Front);
        assert_eq!(PdfSide::from_str("back").unwrap(), PdfSide::Back);
        assert_eq!(PdfSide::from_str("FRONT").unwrap(), PdfSide::Front);
        assert_eq!(PdfSide::from_str("BACK").unwrap(), PdfSide::Back);
        assert_eq!(PdfSide::from_str("frontside").unwrap(), PdfSide::Front);
        assert_eq!(PdfSide::from_str("backside").unwrap(), PdfSide::Back);
    }

    #[test]
    fn test_pdf_side_invalid() {
        assert!(PdfSide::from_str("invalid").is_err());
        assert!(PdfSide::from_str("").is_err());
    }

    #[test]
    fn test_pdf_side_display() {
        assert_eq!(PdfSide::Front.to_string(), "front");
        assert_eq!(PdfSide::Back.to_string(), "back");
    }

    #[test]
    fn test_pdf_side_opposite() {
        assert_eq!(PdfSide::Front.opposite(), PdfSide::Back);
        assert_eq!(PdfSide::Back.opposite(), PdfSide::Front);
    }

    #[test]
    fn test_pdf_side_all() {
        let sides = PdfSide::all();
        assert_eq!(sides.len(), 2);
        assert!(sides.contains(&PdfSide::Front));
        assert!(sides.contains(&PdfSide::Back));
    }

    #[test]
    fn test_pdf_creation() {
        let data = vec![1, 2, 3, 4, 5];
        let pdf = Pdf::new(data.clone());
        assert_eq!(pdf.as_bytes(), &data);
        assert_eq!(pdf.len(), 5);
        assert!(!pdf.is_empty());
    }

    #[test]
    fn test_pdf_empty() {
        let pdf = Pdf::new(vec![]);
        assert!(pdf.is_empty());
        assert_eq!(pdf.len(), 0);
    }

    #[test]
    fn test_pdf_from_vec() {
        let data = vec![1, 2, 3];
        let pdf: Pdf = data.clone().into();
        assert_eq!(pdf.as_bytes(), &data);
    }

    #[test]
    fn test_pdf_into_vec() {
        let data = vec![1, 2, 3];
        let pdf = Pdf::new(data.clone());
        let result: Vec<u8> = pdf.into();
        assert_eq!(result, data);
    }

    #[test]
    fn test_pdf_size_string() {
        let pdf_small = Pdf::new(vec![0; 500]);
        assert_eq!(pdf_small.size_string(), "500 B");
        
        let pdf_kb = Pdf::new(vec![0; 1024]);
        assert_eq!(pdf_kb.size_string(), "1.0 KB");
        
        let pdf_mb = Pdf::new(vec![0; 1024 * 1024]);
        assert_eq!(pdf_mb.size_string(), "1.0 MB");
    }
}