//! HTML generation for Spotify playlist cards
//! 
//! This module handles the generation of HTML documents containing
//! printable cards with QR codes for Spotify songs.

use crate::SongCard;
use crate::qr_generator;

/// Configuration for HTML generation
#[derive(Clone)]
pub struct HtmlGenerator {
    /// Card width in millimeters
    card_width_mm: f64,
    /// Card height in millimeters
    card_height_mm: f64,
    /// Margin between cards in millimeters
    margin_mm: f64,
}

impl HtmlGenerator {
    /// Create a new HTML generator with default settings
    /// 
    /// Uses standard business card dimensions (90mm x 55mm)
    pub fn new() -> Self {
        Self {
            card_width_mm: 90.0,  // Standard business card width in mm
            card_height_mm: 55.0, // Standard business card height in mm
            margin_mm: 5.0,
        }
    }

    
    /// Build HTML content from song cards
    /// 
    /// Creates a complete HTML document string with printable cards.
    /// 
    /// # Arguments
    /// 
    /// * `cards` - Vector of song cards to generate HTML for
    /// * `title` - Title for the HTML document
    /// 
    /// # Returns
    /// 
    /// A complete HTML document string
    pub fn build_html_content(&self, cards: Vec<SongCard>, title: &str) -> String {
        let cards_count = cards.len();
        let cards_html = self.generate_cards_html(&cards);
        
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hitster Cards - {title}</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <style>
        /* Print optimization */
        @page {{
            size: A4;
            margin: 8mm;
        }}
        
        /* Card dimensions and styling */
        .card {{
            width: {card_width_mm}mm;
            height: {card_height_mm}mm;
            border: 2px solid #e5e7eb;
            page-break-inside: avoid;
            box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
            transition: all 0.2s ease-in-out;
        }}
        
        .card:hover {{
            transform: translateY(-2px);
            box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
        }}
        
        .qr-code {{
            width: 22mm;
            height: 22mm;
            min-width: 22mm;
            min-height: 22mm;
        }}
        
        /* Responsive grid adjustments */
        .card-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            gap: 1rem;
            justify-items: center;
            align-items: start;
        }}
        
        /* Print-specific styles */
        @media print {{
            body {{
                margin: 0;
                padding: 0;
                background: white;
            }}
            
            .card-grid {{
                grid-template-columns: repeat(2, 1fr);
                gap: {margin_mm}mm;
                margin: 0;
            }}
            
            .card {{
                width: {card_width_mm}mm !important;
                height: {card_height_mm}mm !important;
                border: 1px solid #000;
                box-shadow: none;
                page-break-inside: avoid;
                break-inside: avoid;
            }}
            
            .card:hover {{
                transform: none;
            }}
            
            .qr-code {{
                width: 20mm !important;
                height: 20mm !important;
            }}
            
            /* Hide interactive elements in print */
            .no-print {{
                display: none !important;
            }}
            
            /* Ensure proper page breaks */
            .card-container {{
                page-break-inside: avoid;
            }}
        }}
        
        /* Large screen optimizations */
        @media (min-width: 1536px) {{
            .card-grid {{
                grid-template-columns: repeat(4, 1fr);
            }}
        }}
        
        @media (min-width: 1280px) and (max-width: 1535px) {{
            .card-grid {{
                grid-template-columns: repeat(3, 1fr);
            }}
        }}
        
        /* Mobile optimizations */
        @media (max-width: 767px) {{
            .card-grid {{
                grid-template-columns: 1fr;
                gap: 1.5rem;
            }}
            
            .card {{
                width: 100% !important;
                max-width: 350px;
                height: auto !important;
                min-height: 200px;
            }}
            
            .qr-code {{
                width: 60px !important;
                height: 60px !important;
                min-width: 60px;
                min-height: 60px;
            }}
        }}
        
        /* Loading animation */
        @keyframes pulse {{
            0%, 100% {{ opacity: 1; }}
            50% {{ opacity: 0.5; }}
        }}
        
        .loading {{
            animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
        }}
        
        /* Smooth scrolling for pagination */
        html {{
            scroll-behavior: smooth;
        }}
        
        /* Line clamp for text truncation */
        .line-clamp-2 {{
            display: -webkit-box;
            -webkit-line-clamp: 2;
            -webkit-box-orient: vertical;
            overflow: hidden;
        }}
    </style>
</head>
<body class="bg-gray-50 min-h-screen">
    <!-- Header with controls -->
    <div class="no-print bg-white shadow-sm border-b sticky top-0 z-10">
        <div class="max-w-7xl mx-auto px-4 py-4">
            <div class="flex flex-col sm:flex-row justify-between items-center gap-4">
                <div class="text-center sm:text-left">
                    <h1 class="text-2xl sm:text-3xl font-bold text-gray-900">Hitster Cards</h1>
                    <p class="text-sm sm:text-base text-gray-600 mt-1">{title} • {cards_count} songs</p>
                </div>
                
                <!-- Controls -->
                <div class="flex flex-wrap gap-2 justify-center">
                    <button onclick="window.print()" 
                            class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors text-sm font-medium">
                        🖨️ Print Cards
                    </button>
                    <button onclick="toggleLayout()" 
                            class="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors text-sm font-medium">
                        📋 Toggle Layout
                    </button>
                    <button onclick="scrollToTop()" 
                            class="px-4 py-2 bg-gray-500 text-white rounded-lg hover:bg-gray-600 transition-colors text-sm font-medium">
                        ⬆️ Top
                    </button>
                </div>
            </div>
        </div>
    </div>

    <!-- Main content -->
    <main class="max-w-7xl mx-auto px-4 py-8">
        <!-- Loading indicator (initially hidden) -->
        <div id="loading" class="hidden text-center py-8">
            <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
            <p class="mt-2 text-gray-600">Loading more cards...</p>
        </div>

        <!-- Cards grid -->
        <div id="cardsGrid" class="card-grid">
            {cards_html}
        </div>

        <!-- Load more button for large playlists -->
        <div id="loadMoreContainer" class="hidden text-center mt-12 no-print">
            <button onclick="loadMoreCards()" 
                    class="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium">
                Load More Cards
            </button>
        </div>
    </main>

    <!-- Footer -->
    <footer class="no-print bg-gray-100 border-t mt-16">
        <div class="max-w-7xl mx-auto px-4 py-6">
            <div class="text-center text-sm text-gray-600">
                <p>💡 Tip: Use Ctrl+P (or Cmd+P) to print, or save as PDF</p>
                <p class="mt-1">Each card contains a QR code that links to the song on Spotify</p>
            </div>
        </div>
    </footer>

    <script>
        // Layout management
        let isCompactLayout = false;
        const cardsGrid = document.getElementById('cardsGrid');
        
        function toggleLayout() {{
            isCompactLayout = !isCompactLayout;
            if (isCompactLayout) {{
                cardsGrid.classList.add('compact-layout');
                cardsGrid.style.gridTemplateColumns = 'repeat(auto-fit, minmax(200px, 1fr))';
            }} else {{
                cardsGrid.classList.remove('compact-layout');
                cardsGrid.style.gridTemplateColumns = '';
            }}
        }}
        
        function scrollToTop() {{
            window.scrollTo({{ top: 0, behavior: 'smooth' }});
        }}
        
        // Pagination for large playlists
        const cardsPerPage = 24;
        let currentPage = 1;
        const totalCards = {cards_count};
        
        function showLoadMoreButton() {{
            const loadMoreContainer = document.getElementById('loadMoreContainer');
            if (totalCards > cardsPerPage) {{
                loadMoreContainer.classList.remove('hidden');
                updateLoadMoreButton();
            }}
        }}
        
        function updateLoadMoreButton() {{
            const loadMoreBtn = document.querySelector('#loadMoreContainer button');
            const remainingCards = totalCards - (currentPage * cardsPerPage);
            if (remainingCards <= 0) {{
                loadMoreBtn.textContent = 'All cards loaded';
                loadMoreBtn.disabled = true;
                loadMoreBtn.classList.add('opacity-50', 'cursor-not-allowed');
            }} else {{
                loadMoreBtn.textContent = `Load More Cards (${{remainingCards}} remaining)`;
            }}
        }}
        
        function loadMoreCards() {{
            const loading = document.getElementById('loading');
            loading.classList.remove('hidden');
            
            // Simulate loading (in real implementation, this would fetch more data)
            setTimeout(() => {{
                currentPage++;
                loading.classList.add('hidden');
                updateLoadMoreButton();
            }}, 800);
        }}
        
        // Initialize
        document.addEventListener('DOMContentLoaded', function() {{
            showLoadMoreButton();
            
            // Add keyboard shortcuts
            document.addEventListener('keydown', function(e) {{
                if (e.ctrlKey || e.metaKey) {{
                    switch(e.key) {{
                        case 'p':
                            e.preventDefault();
                            window.print();
                            break;
                        case 'g':
                            e.preventDefault();
                            toggleLayout();
                            break;
                    }}
                }}
            }});
            
            // Add print preview optimization
            window.addEventListener('beforeprint', function() {{
                document.body.classList.add('print-preview');
            }});
            
            window.addEventListener('afterprint', function() {{
                document.body.classList.remove('print-preview');
            }});
        }});
        
        // Intersection Observer for lazy loading (if needed in future)
        const observerOptions = {{
            root: null,
            rootMargin: '50px',
            threshold: 0.1
        }};
        
        const observer = new IntersectionObserver((entries) => {{
            entries.forEach(entry => {{
                if (entry.isIntersecting) {{
                    entry.target.classList.add('visible');
                }}
            }});
        }}, observerOptions);
        
        // Observe all cards for animation
        document.addEventListener('DOMContentLoaded', function() {{
            const cards = document.querySelectorAll('.card');
            cards.forEach(card => {{
                observer.observe(card);
            }});
        }});
    </script>
</body>
</html>"#,
            title = title,
            cards_count = cards_count,
            card_width_mm = self.card_width_mm,
            card_height_mm = self.card_height_mm,
            margin_mm = self.margin_mm,
            cards_html = cards_html
        )
    }

    /// Generate HTML for all cards
    /// 
    /// # Arguments
    /// 
    /// * `cards` - Slice of song cards to generate HTML for
    /// 
    /// # Returns
    /// 
    /// HTML string for all cards
    fn generate_cards_html(&self, cards: &[SongCard]) -> String {
        cards
            .iter()
            .map(|card| self.generate_single_card_html(card))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Generate HTML for a single card
    /// 
    /// # Arguments
    /// 
    /// * `card` - The song card to generate HTML for
    /// 
    /// # Returns
    /// 
    /// HTML string for a single card
    fn generate_single_card_html(&self, card: &SongCard) -> String {
        let qr_data_url = match qr_generator::generate_qr_data_url(&card.spotify_url) {
            Ok(url) => url,
            Err(_) => "".to_string(), // Fallback to no QR code if generation fails
        };

        format!(
            r#"<div class="card-container">
                <div class="card bg-white rounded-lg shadow-md overflow-hidden flex flex-col">
                    <!-- Card header with song info -->
                    <div class="p-3 pb-2 flex-1">
                        <h3 class="text-sm font-bold text-gray-900 leading-tight mb-1 line-clamp-2" title="{title_title}">
                            {title}
                        </h3>
                        <p class="text-xs text-gray-600 leading-tight mb-1 line-clamp-2" title="{artist_title}">
                            {artist}
                        </p>
                        <div class="flex items-center justify-between">
                            <span class="text-xs text-gray-500 bg-gray-100 px-2 py-1 rounded-full">
                                {year}
                            </span>
                            <span class="text-xs text-green-600 font-medium">
                                ♪ Spotify
                            </span>
                        </div>
                    </div>
                    
                    <!-- QR Code section -->
                    <div class="border-t border-gray-100 bg-gray-50 p-3 flex items-center justify-center">
                        {qr_html}
                    </div>
                    
                    <!-- Footer -->
                    <div class="px-3 pb-2">
                        <p class="text-xs text-gray-400 text-center">
                            Scan to play
                        </p>
                    </div>
                </div>
            </div>"#,
            title = html_escape::encode_text(&card.title),
            artist = html_escape::encode_text(&card.artist),
            year = html_escape::encode_text(&card.year),
            title_title = html_escape::encode_double_quoted_attribute(&card.title),
            artist_title = html_escape::encode_double_quoted_attribute(&card.artist),
            qr_html = if qr_data_url.is_empty() {
                r#"<div class="text-gray-400 text-center">
                    <div class="w-12 h-12 border-2 border-dashed border-gray-300 rounded-lg flex items-center justify-center mx-auto mb-1">
                        <span class="text-2xl">🎵</span>
                    </div>
                    <span class="text-xs">QR unavailable</span>
                </div>"#.to_string()
            } else {
                format!(
                    r#"<div class="relative group cursor-pointer" onclick="window.open('{}', '_blank')">
                        <img src="{}" alt="QR Code" class="qr-code transition-all group-hover:scale-105">
                        <div class="absolute inset-0 bg-black bg-opacity-0 group-hover:bg-opacity-10 transition-all rounded-lg flex items-center justify-center">
                            <span class="text-white text-xs opacity-0 group-hover:opacity-100 transition-opacity">
                                Open in Spotify
                            </span>
                        </div>
                    </div>"#,
                    html_escape::encode_double_quoted_attribute(&card.spotify_url),
                    qr_data_url
                )
            }
        )
    }
}

impl Default for HtmlGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    
    #[test]
    fn test_build_html_content() {
        let generator = HtmlGenerator::new();
        let cards = vec![
            SongCard {
                title: "Test Song".to_string(),
                artist: "Test Artist".to_string(),
                year: "2023".to_string(),
                spotify_url: "https://example.com".to_string(),
            },
        ];
        
        let html = generator.build_html_content(cards, "Test Playlist");
        
        // Check that HTML contains expected elements
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Test Song"));
        assert!(html.contains("Test Artist"));
        assert!(html.contains("tailwindcss"));
    }
}