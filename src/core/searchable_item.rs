use serde::{Serialize, Deserialize};
use crate::core::item::Item;

/// A searchable item with multiple indexed fields
#[derive(Clone, Serialize, Deserialize)]
pub struct SearchableItem {
    /// The original item
    pub item: Item,
    
    /// Indexed fields for searching
    pub name: String,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub generic_name: Option<String>,
    pub description: Option<String>,
    pub executable: String,
}

impl SearchableItem {
    /// Create a new searchable item
    pub fn new(
        item: Item,
        name: String,
        keywords: Vec<String>,
        categories: Vec<String>,
        generic_name: Option<String>,
        description: Option<String>,
        executable: String,
    ) -> Self {
        Self {
            item,
            name,
            keywords,
            categories,
            generic_name,
            description,
            executable,
        }
    }

    /// Get all searchable text fields with their weights
    pub fn get_weighted_fields(&self) -> Vec<SearchField> {
        let mut fields = Vec::new();

        // Name (weight: 10.0) - highest priority
        fields.push(SearchField {
            text: self.name.clone(),
            weight: 10.0,
            field_type: FieldType::Name,
        });

        // Keywords (weight: 8.0)
        for keyword in &self.keywords {
            fields.push(SearchField {
                text: keyword.clone(),
                weight: 8.0,
                field_type: FieldType::Keyword,
            });
        }

        // Generic name (weight: 6.0)
        if let Some(generic) = &self.generic_name {
            fields.push(SearchField {
                text: generic.clone(),
                weight: 6.0,
                field_type: FieldType::GenericName,
            });
        }

        // Categories (weight: 5.0)
        for category in &self.categories {
            fields.push(SearchField {
                text: category.clone(),
                weight: 5.0,
                field_type: FieldType::Category,
            });
        }

        // Description (weight: 3.0)
        if let Some(desc) = &self.description {
            fields.push(SearchField {
                text: desc.clone(),
                weight: 3.0,
                field_type: FieldType::Description,
            });
        }

        // Executable (weight: 2.0) - lowest priority
        fields.push(SearchField {
            text: self.executable.clone(),
            weight: 2.0,
            field_type: FieldType::Executable,
        });

        fields
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
