use serde::{Serialize, Deserialize};
use crate::core::item::Item;

/// A searchable item with multiple indexed fields
#[derive(Clone, Serialize, Deserialize)]
pub struct SearchableItem {
    /// The original item
    pub item: Item,
    
    /// Indexed fields for searching (original text)
    pub name: String,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub generic_name: Option<String>,
    pub description: Option<String>,
    pub executable: String,
    
    /// Tokenized versions for fast matching
    pub name_tokens: Vec<String>,
    pub keyword_tokens: Vec<String>,
    pub category_tokens: Vec<String>,
    pub generic_name_tokens: Vec<String>,
    pub description_tokens: Vec<String>,
    pub executable_tokens: Vec<String>,
    
    /// Extracted acronyms
    pub acronyms: Vec<String>,
}

impl SearchableItem {
    /// Create a new searchable item with tokenization
    pub fn new(
        item: Item,
        name: String,
        keywords: Vec<String>,
        categories: Vec<String>,
        generic_name: Option<String>,
        description: Option<String>,
        executable: String,
    ) -> Self {
        use crate::search::tokenizer::Tokenizer;
        
        let tokenizer = Tokenizer::new();
        
        // Tokenize name
        let name_tokens = tokenizer.tokenize_comprehensive(&name);
        
        // Tokenize keywords
        let keyword_tokens: Vec<String> = keywords
            .iter()
            .flat_map(|k| tokenizer.tokenize(k))
            .collect();
        
        // Tokenize categories
        let category_tokens: Vec<String> = categories
            .iter()
            .flat_map(|c| tokenizer.tokenize(c))
            .collect();
        
        // Tokenize generic name
        let generic_name_tokens = if let Some(ref gn) = generic_name {
            tokenizer.tokenize_comprehensive(gn)
        } else {
            Vec::new()
        };
        
        // Tokenize description
        let description_tokens = if let Some(ref desc) = description {
            tokenizer.tokenize(desc)
        } else {
            Vec::new()
        };
        
        // Tokenize executable
        let executable_tokens = tokenizer.tokenize(&executable);
        
        // Extract all acronyms
        let mut acronyms = Vec::new();
        acronyms.extend(tokenizer.extract_all_acronyms(&name));
        if let Some(ref gn) = generic_name {
            acronyms.extend(tokenizer.extract_all_acronyms(gn));
        }
        
        // Remove duplicate acronyms
        acronyms.sort();
        acronyms.dedup();
        
        Self {
            item,
            name,
            keywords,
            categories,
            generic_name,
            description,
            executable,
            name_tokens,
            keyword_tokens,
            category_tokens,
            generic_name_tokens,
            description_tokens,
            executable_tokens,
            acronyms,
        }
    }

    /// Get all searchable text fields with their weights
    pub fn get_weighted_fields(&self) -> Vec<SearchField> {
        let mut fields = Vec::new();

        // Name (weight: 10.0) - highest priority
        fields.push(SearchField {
            text: self.name.clone(),
            tokens: self.name_tokens.clone(),
            weight: 10.0,
            field_type: FieldType::Name,
        });

        // Keywords (weight: 8.0)
        for keyword in &self.keywords {
            fields.push(SearchField {
                text: keyword.clone(),
                tokens: vec![keyword.to_lowercase()],
                weight: 8.0,
                field_type: FieldType::Keyword,
            });
        }

        // Generic name (weight: 6.0)
        if let Some(generic) = &self.generic_name {
            fields.push(SearchField {
                text: generic.clone(),
                tokens: self.generic_name_tokens.clone(),
                weight: 6.0,
                field_type: FieldType::GenericName,
            });
        }

        // Categories (weight: 5.0)
        for category in &self.categories {
            fields.push(SearchField {
                text: category.clone(),
                tokens: vec![category.to_lowercase()],
                weight: 5.0,
                field_type: FieldType::Category,
            });
        }

        // Description (weight: 3.0)
        if let Some(desc) = &self.description {
            fields.push(SearchField {
                text: desc.clone(),
                tokens: self.description_tokens.clone(),
                weight: 3.0,
                field_type: FieldType::Description,
            });
        }

        // Executable (weight: 2.0) - lowest priority
        fields.push(SearchField {
            text: self.executable.clone(),
            tokens: self.executable_tokens.clone(),
            weight: 2.0,
            field_type: FieldType::Executable,
        });

        fields
    }
    
    /// Get all tokens from all fields
    pub fn get_all_tokens(&self) -> Vec<String> {
        let mut tokens = Vec::new();
        tokens.extend(self.name_tokens.clone());
        tokens.extend(self.keyword_tokens.clone());
        tokens.extend(self.category_tokens.clone());
        tokens.extend(self.generic_name_tokens.clone());
        tokens.extend(self.description_tokens.clone());
        tokens.extend(self.executable_tokens.clone());
        tokens.extend(self.acronyms.clone());
        
        // Remove duplicates
        tokens.sort();
        tokens.dedup();
        tokens
    }

    /// Get all text content for simple searching
    pub fn get_all_text(&self) -> String {
        let mut parts = vec![self.name.clone()];
        parts.extend(self.keywords.clone());
        parts.extend(self.categories.clone());
        
        if let Some(generic) = &self.generic_name {
            parts.push(generic.clone());
        }
        
        if let Some(desc) = &self.description {
            parts.push(desc.clone());
        }
        
        parts.push(self.executable.clone());
        
        parts.join(" ").to_lowercase()
    }
}

/// A searchable field with its weight
#[derive(Clone, Debug)]
pub struct SearchField {
    pub text: String,
    pub tokens: Vec<String>,
    pub weight: f64,
    pub field_type: FieldType,
}

/// Type of search field
#[derive(Clone, Debug, PartialEq)]
pub enum FieldType {
    Name,
    Keyword,
    GenericName,
    Category,
    Description,
    Executable,
}

impl FieldType {
    /// Get the display name of the field type
    pub fn display_name(&self) -> &str {
        match self {
            FieldType::Name => "Name",
            FieldType::Keyword => "Keyword",
            FieldType::GenericName => "Generic Name",
            FieldType::Category => "Category",
            FieldType::Description => "Description",
            FieldType::Executable => "Executable",
        }
    }
}
