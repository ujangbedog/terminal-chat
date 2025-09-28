//! Menu item definition and related functionality

/// Represents a single menu item with its properties
#[derive(Debug, Clone)]
pub struct MenuItem {
    /// Unique identifier for the menu item
    pub id: usize,
    /// Display title of the menu item
    pub title: String,
    /// Description text shown when item is selected
    pub description: String,
    /// Whether this menu item is currently available/enabled
    pub available: bool,
}

impl MenuItem {
    /// Creates a new menu item
    pub fn new(id: usize, title: impl Into<String>, description: impl Into<String>, available: bool) -> Self {
        Self {
            id,
            title: title.into(),
            description: description.into(),
            available,
        }
    }

    /// Creates a new available menu item
    pub fn available(id: usize, title: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new(id, title, description, true)
    }

    /// Creates a new disabled menu item (coming soon)
    pub fn coming_soon(id: usize, title: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new(id, title, description, false)
    }
}
