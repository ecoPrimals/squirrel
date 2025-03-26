//! Layout components for the Squirrel Web UI.
//!
//! This module provides layout components for the web-based user interface.

use std::collections::HashMap;
use uuid::Uuid;
use super::{Component, ComponentType};

/// Header component
#[derive(Debug, Clone)]
pub struct Header {
    /// Header ID
    id: String,
    /// Header title
    title: String,
    /// Version
    version: String,
    /// Additional links
    links: Vec<(String, String)>,
}

impl Header {
    /// Create a new header component
    pub fn new(title: &str, version: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            version: version.to_string(),
            links: Vec::new(),
        }
    }
    
    /// Add a link to the header
    pub fn add_link(&mut self, text: &str, url: &str) {
        self.links.push((text.to_string(), url.to_string()));
    }
    
    /// Set the title
    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }
    
    /// Set the version
    pub fn set_version(&mut self, version: &str) {
        self.version = version.to_string();
    }
}

impl Component for Header {
    fn render(&self) -> String {
        let mut links_html = String::new();
        for (text, url) in &self.links {
            links_html.push_str(&format!(r#"<a href="{}" class="header-link">{}</a>"#, url, text));
        }
        
        format!(r#"
<header id="{}" class="app-header">
    <div class="header-title">
        <h1>{}</h1>
        <span class="version">v{}</span>
    </div>
    <div class="header-links">
        {}
    </div>
</header>
"#, self.id, self.title, self.version, links_html)
    }
    
    fn id(&self) -> String {
        self.id.clone()
    }
    
    fn name(&self) -> String {
        "Header".to_string()
    }
    
    fn component_type(&self) -> ComponentType {
        ComponentType::Container
    }
}

/// Footer component
#[derive(Debug, Clone)]
pub struct Footer {
    /// Footer ID
    id: String,
    /// Footer text
    text: String,
    /// Copyright year
    year: String,
    /// Additional links
    links: Vec<(String, String)>,
}

impl Footer {
    /// Create a new footer component
    pub fn new(text: &str, year: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            text: text.to_string(),
            year: year.to_string(),
            links: Vec::new(),
        }
    }
    
    /// Add a link to the footer
    pub fn add_link(&mut self, text: &str, url: &str) {
        self.links.push((text.to_string(), url.to_string()));
    }
    
    /// Set the text
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }
    
    /// Set the year
    pub fn set_year(&mut self, year: &str) {
        self.year = year.to_string();
    }
}

impl Component for Footer {
    fn render(&self) -> String {
        let mut links_html = String::new();
        for (text, url) in &self.links {
            links_html.push_str(&format!(r#"<a href="{}" class="footer-link">{}</a>"#, url, text));
        }
        
        format!(r#"
<footer id="{}" class="app-footer">
    <div class="footer-content">
        <div class="footer-text">
            {} &copy; {}
        </div>
        <div class="footer-links">
            {}
        </div>
    </div>
</footer>
"#, self.id, self.text, self.year, links_html)
    }
    
    fn id(&self) -> String {
        self.id.clone()
    }
    
    fn name(&self) -> String {
        "Footer".to_string()
    }
    
    fn component_type(&self) -> ComponentType {
        ComponentType::Container
    }
}

/// Navigation item
#[derive(Debug, Clone)]
pub struct NavigationItem {
    /// Item ID
    id: String,
    /// Item text
    text: String,
    /// Item URL
    url: String,
    /// Active state
    active: bool,
    /// Icon
    icon: Option<String>,
}

impl NavigationItem {
    /// Create a new navigation item
    pub fn new(text: &str, url: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            text: text.to_string(),
            url: url.to_string(),
            active: false,
            icon: None,
        }
    }
    
    /// Set the active state
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
    
    /// Set the icon
    pub fn set_icon(&mut self, icon: &str) {
        self.icon = Some(icon.to_string());
    }
}

/// Navigation component
#[derive(Debug, Clone)]
pub struct Navigation {
    /// Navigation ID
    id: String,
    /// Navigation items
    items: Vec<NavigationItem>,
}

impl Navigation {
    /// Create a new navigation component
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            items: Vec::new(),
        }
    }
    
    /// Add an item to the navigation
    pub fn add_item(&mut self, text: &str, url: &str) -> &mut Self {
        self.items.push(NavigationItem::new(text, url));
        self
    }
    
    /// Add an item with an icon to the navigation
    pub fn add_item_with_icon(&mut self, text: &str, url: &str, icon: &str) -> &mut Self {
        let mut item = NavigationItem::new(text, url);
        item.set_icon(icon);
        self.items.push(item);
        self
    }
    
    /// Set the active item by URL
    pub fn set_active_item(&mut self, url: &str) {
        for item in &mut self.items {
            item.set_active(item.url == url);
        }
    }
}

impl Component for Navigation {
    fn render(&self) -> String {
        let mut items_html = String::new();
        for item in &self.items {
            let active_class = if item.active { " active" } else { "" };
            let icon_html = match &item.icon {
                Some(icon) => format!(r#"<span class="icon">{}</span>"#, icon),
                None => String::new(),
            };
            
            items_html.push_str(&format!(r#"
<li class="nav-item{}">
    <a href="{}" class="nav-link">
        {}
        <span class="text">{}</span>
    </a>
</li>
            "#, active_class, item.url, icon_html, item.text));
        }
        
        format!(r#"
<nav id="{}" class="app-nav">
    <ul class="nav-list">
        {}
    </ul>
</nav>
"#, self.id, items_html)
    }
    
    fn id(&self) -> String {
        self.id.clone()
    }
    
    fn name(&self) -> String {
        "Navigation".to_string()
    }
    
    fn component_type(&self) -> ComponentType {
        ComponentType::Navigation
    }
}

/// Sidebar component
#[derive(Debug, Clone)]
pub struct Sidebar {
    /// Sidebar ID
    id: String,
    /// Sidebar title
    title: String,
    /// Sidebar content
    content: String,
    /// Collapsed state
    collapsed: bool,
}

impl Sidebar {
    /// Create a new sidebar component
    pub fn new(title: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            content: String::new(),
            collapsed: false,
        }
    }
    
    /// Set the sidebar content
    pub fn set_content(&mut self, content: &str) {
        self.content = content.to_string();
    }
    
    /// Set the collapsed state
    pub fn set_collapsed(&mut self, collapsed: bool) {
        self.collapsed = collapsed;
    }
}

impl Component for Sidebar {
    fn render(&self) -> String {
        let collapsed_class = if self.collapsed { " collapsed" } else { "" };
        
        format!(r#"
<aside id="{}" class="app-sidebar{}">
    <div class="sidebar-header">
        <h2>{}</h2>
        <button class="sidebar-toggle">
            <span class="toggle-icon"></span>
        </button>
    </div>
    <div class="sidebar-content">
        {}
    </div>
</aside>
"#, self.id, collapsed_class, self.title, self.content)
    }
    
    fn id(&self) -> String {
        self.id.clone()
    }
    
    fn name(&self) -> String {
        "Sidebar".to_string()
    }
    
    fn component_type(&self) -> ComponentType {
        ComponentType::Container
    }
}

/// Layout component
#[derive(Debug, Clone)]
pub struct Layout {
    /// Layout ID
    id: String,
    /// Header
    header: Header,
    /// Navigation
    navigation: Navigation,
    /// Footer
    footer: Footer,
    /// Sidebar
    sidebar: Option<Sidebar>,
    /// Content
    content: String,
}

impl Layout {
    /// Create a new layout component
    pub fn new(header: Header, navigation: Navigation, footer: Footer) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            header,
            navigation,
            footer,
            sidebar: None,
            content: String::new(),
        }
    }
    
    /// Set the sidebar
    pub fn set_sidebar(&mut self, sidebar: Sidebar) {
        self.sidebar = Some(sidebar);
    }
    
    /// Set the content
    pub fn set_content(&mut self, content: &str) {
        self.content = content.to_string();
    }
}

impl Component for Layout {
    fn render(&self) -> String {
        let sidebar_html = match &self.sidebar {
            Some(sidebar) => sidebar.render(),
            None => String::new(),
        };
        
        let has_sidebar_class = if self.sidebar.is_some() { " has-sidebar" } else { "" };
        
        format!(r#"
<div id="{}" class="app-layout{}">
    {}
    <div class="layout-main">
        {}
        <main class="layout-content">
            {}
        </main>
    </div>
    {}
</div>
"#, self.id, has_sidebar_class, self.header.render(), sidebar_html, self.content, self.footer.render())
    }
    
    fn id(&self) -> String {
        self.id.clone()
    }
    
    fn name(&self) -> String {
        "Layout".to_string()
    }
    
    fn component_type(&self) -> ComponentType {
        ComponentType::Container
    }
} 