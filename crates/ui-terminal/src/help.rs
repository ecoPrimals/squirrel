use std::collections::HashMap;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// Help category for organizing help topics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HelpCategory {
    /// Overview of the dashboard
    Overview,
    /// Navigation shortcuts
    Navigation,
    /// Keyboard shortcuts
    Shortcuts,
    /// Alert management
    Alerts,
    /// Network monitoring
    Network,
    /// Protocol information
    Protocol,
    /// Command execution
    Commands,
    /// UI customization
    Settings,
    /// General help
    General,
    /// Metrics help
    Metrics,
    /// Debugging help
    Debugging,
}

impl HelpCategory {
    /// Get display name for category
    pub fn display_name(&self) -> &str {
        match self {
            HelpCategory::Overview => "Dashboard Overview",
            HelpCategory::Navigation => "Navigation",
            HelpCategory::Shortcuts => "Keyboard Shortcuts",
            HelpCategory::Alerts => "Alert Management",
            HelpCategory::Network => "Network Monitoring",
            HelpCategory::Protocol => "Protocol Information",
            HelpCategory::Commands => "Command Execution",
            HelpCategory::Settings => "Settings",
            HelpCategory::General => "General",
            HelpCategory::Metrics => "Metrics",
            HelpCategory::Debugging => "Debugging",
        }
    }
    
    /// Get all categories
    pub fn all() -> Vec<HelpCategory> {
        vec![
            HelpCategory::Overview,
            HelpCategory::Navigation,
            HelpCategory::Shortcuts,
            HelpCategory::Alerts,
            HelpCategory::Network,
            HelpCategory::Protocol,
            HelpCategory::Commands,
            HelpCategory::Settings,
            HelpCategory::General,
            HelpCategory::Metrics,
            HelpCategory::Debugging,
        ]
    }
}

/// Help section containing content for a specific help topic
#[derive(Debug, Clone)]
pub struct HelpSection {
    /// Title of the help section
    pub title: String,
    /// Content lines for this help section
    pub content: Vec<Line<'static>>,
    /// Category for this help section
    pub category: HelpCategory,
    /// Keywords for searching
    pub keywords: Vec<String>,
}

impl HelpSection {
    /// Create a new help section
    pub fn new(title: &str, category: HelpCategory) -> Self {
        Self {
            title: title.to_string(),
            content: Vec::new(),
            category,
            keywords: Vec::new(),
        }
    }
    
    /// Add content line to the section
    pub fn add_line(mut self, line: Line<'static>) -> Self {
        self.content.push(line);
        self
    }
    
    /// Add text to the help section
    pub fn add_text(mut self, text: &str) -> Self {
        self.content.push(Line::from(text.to_string()));
        self
    }
    
    /// Add a keyword for searching
    pub fn add_keyword(mut self, keyword: &str) -> Self {
        self.keywords.push(keyword.to_string());
        self
    }
}

/// Help system for providing contextual help
#[derive(Debug, Clone)]
pub struct HelpSystem {
    /// Is the help panel active?
    active: bool,
    /// Help sections
    sections: HashMap<String, HelpSection>,
    /// Help content by category
    content: HashMap<HelpCategory, Vec<Line<'static>>>,
    /// Currently selected category
    selected_category: HelpCategory,
}

impl HelpSystem {
    /// Create a new help system
    pub fn new() -> Self {
        let mut help_system = Self {
            active: true,
            sections: HashMap::new(),
            content: HashMap::new(),
            selected_category: HelpCategory::Overview,
        };
        
        help_system.initialize_content();
        
        help_system
    }
    
    /// Initialize help content
    fn initialize_content(&mut self) {
        // Overview
        self.content.insert(HelpCategory::Overview, vec![
            Line::from(vec![
                Span::styled("Dashboard Overview", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
            Line::from("This dashboard provides real-time monitoring of system metrics and protocol status."),
            Line::from(""),
            Line::from(vec![
                Span::styled("Key Features:", Style::default().add_modifier(Modifier::BOLD))
            ]),
            Line::from("• Real-time system metrics (CPU, memory, disk)"),
            Line::from("• Network interface status and throughput"),
            Line::from("• Protocol connection status and statistics"),
            Line::from("• Alert management for system issues"),
        ]);
        
        // Navigation
        self.content.insert(HelpCategory::Navigation, vec![
            Line::from(vec![
                Span::styled("Navigation", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
            Line::from("The dashboard is divided into several tabs:"),
            Line::from(""),
            Line::from(vec![
                Span::styled("1. Overview", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - System health and resource usage overview")
            ]),
            Line::from(vec![
                Span::styled("2. Network", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - Network interface details and statistics")
            ]),
            Line::from(vec![
                Span::styled("3. Protocol", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - Protocol connection status and metrics")
            ]),
            Line::from(vec![
                Span::styled("4. Alerts", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - System alerts and notifications")
            ]),
        ]);
        
        // Shortcuts
        self.content.insert(HelpCategory::Shortcuts, vec![
            Line::from(vec![
                Span::styled("Keyboard Shortcuts", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Navigation:", Style::default().add_modifier(Modifier::BOLD))
            ]),
            Line::from(vec![
                Span::styled("1-4", Style::default().fg(Color::Cyan)),
                Span::raw(" - Switch between tabs")
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("General:", Style::default().add_modifier(Modifier::BOLD))
            ]),
            Line::from(vec![
                Span::styled("h", Style::default().fg(Color::Cyan)),
                Span::raw(" - Toggle this help screen")
            ]),
            Line::from(vec![
                Span::styled("q", Style::default().fg(Color::Cyan)),
                Span::raw(" - Quit the application")
            ]),
        ]);
        
        // Alerts
        self.content.insert(HelpCategory::Alerts, vec![
            Line::from(vec![
                Span::styled("Alert Management", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
            Line::from("The alerts tab shows system warnings and critical issues."),
            Line::from(""),
            Line::from("Alerts are color-coded by severity:"),
            Line::from(vec![
                Span::styled("• Critical", Style::default().fg(Color::Red)),
                Span::raw(" - Requires immediate attention")
            ]),
            Line::from(vec![
                Span::styled("• Warning", Style::default().fg(Color::Yellow)),
                Span::raw(" - Potential issue to monitor")
            ]),
            Line::from(vec![
                Span::styled("• Information", Style::default().fg(Color::Blue)),
                Span::raw(" - System notifications")
            ]),
        ]);
        
        // Network
        self.content.insert(HelpCategory::Network, vec![
            Line::from(vec![
                Span::styled("Network Monitoring", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
            Line::from("The network tab displays information about network interfaces:"),
            Line::from(""),
            Line::from("• Interface status (up/down)"),
            Line::from("• IP addresses and MAC addresses"),
            Line::from("• Transfer rates (RX/TX)"),
            Line::from("• Historical bandwidth usage graphs"),
        ]);
        
        // Protocol
        self.content.insert(HelpCategory::Protocol, vec![
            Line::from(vec![
                Span::styled("Protocol Information", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
            Line::from("The protocol tab shows the status of system communication:"),
            Line::from(""),
            Line::from("• Connection status and uptime"),
            Line::from("• Protocol version and compatibility"),
            Line::from("• Performance metrics and latency"),
            Line::from("• Error tracking and retry statistics"),
        ]);
    }
    
    /// Get help topic list as spans for rendering
    pub fn get_topic_list(&self) -> Vec<Line<'static>> {
        HelpCategory::all().iter().map(|category| {
            let selected = *category == self.selected_category;
            
            Line::from(vec![
                Span::styled(
                    format!("• {}", category.display_name()),
                    if selected {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    }
                )
            ])
        }).collect()
    }
    
    /// Get help content for the selected category
    pub fn get_content(&self) -> Vec<Line<'static>> {
        self.content.get(&self.selected_category)
            .cloned()
            .unwrap_or_else(|| vec![
                Line::from(vec![
                    Span::styled(
                        "No help content available for this category.",
                        Style::default().fg(Color::Red)
                    )
                ])
            ])
    }
    
    /// Set the selected help category
    pub fn set_category(&mut self, category: HelpCategory) {
        self.selected_category = category;
    }
    
    /// Get the selected help category
    pub fn selected_category(&self) -> HelpCategory {
        self.selected_category
    }
    
    /// Move to the next help category
    pub fn next_category(&mut self) {
        let categories = HelpCategory::all();
        let current_index = categories.iter()
            .position(|c| *c == self.selected_category)
            .unwrap_or(0);
        
        let next_index = (current_index + 1) % categories.len();
        self.selected_category = categories[next_index];
    }
    
    /// Move to the previous help category
    pub fn prev_category(&mut self) {
        let categories = HelpCategory::all();
        let current_index = categories.iter()
            .position(|c| *c == self.selected_category)
            .unwrap_or(0);
        
        let prev_index = if current_index == 0 {
            categories.len() - 1
        } else {
            current_index - 1
        };
        self.selected_category = categories[prev_index];
    }
    
    /// Get topic list spans (compatibility method)
    pub fn get_topic_list_spans(&self) -> Vec<Line<'static>> {
        self.get_topic_list()
    }
    
    /// Get content for the current help category as spans
    pub fn get_help_content_spans(&mut self, category: HelpCategory) -> Vec<Line<'static>> {
        self.set_category(category);
        
        // Get help content
        self.get_content()
    }
}

impl Default for HelpSystem {
    fn default() -> Self {
        Self::new()
    }
} 