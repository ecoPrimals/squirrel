/// Style roles used in the UI theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleRole {
    /// Style for normal text
    Normal,
    /// Style for headers
    Header,
    /// Style for titles
    Title,
    /// Style for labels
    Label,
    /// Style for values
    Value,
    /// Style for borders
    Border,
    /// Style for selected items
    Selected,
    /// Style for highlighted items
    Highlighted,
    /// Style for disabled items
    Disabled,
    /// Style for focused items
    Focused,
    /// Style for active items
    Active,
    /// Style for inactive items
    Inactive,
    /// Style for text
    Text,
    /// Style for links
    Link,
} 