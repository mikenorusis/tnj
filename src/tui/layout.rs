use ratatui::layout::{Rect, Layout as RatLayout, Direction, Constraint};

pub struct Layout {
    pub inner_area: Rect,  // Area inside the outer border
    pub tabs_area: Rect,
    pub sidebar_area: Rect,
    pub main_area: Rect,
    pub filters_area: Rect,
    pub status_area: Rect,
}

impl Layout {
    /// Minimum terminal dimensions required for the application
    /// Width: 38 columns (36 inner + 2 borders) allows sidebar (25) + main (11) when expanded,
    /// or just main (36) when sidebar is collapsed
    /// Height: 10 lines (2 outer borders + 1 tabs + 1 content + 3 filters + 1 status + 2 buffer)
    pub const MIN_WIDTH: u16 = 38;
    pub const MIN_HEIGHT: u16 = 10;

    pub fn calculate(
        size: Rect,
        sidebar_width_percent: u16,
        sidebar_collapsed: bool,
    ) -> Self {
        // Use the actual terminal size - minimum size check happens before entering TUI
        // Calculate inner area (accounting for outer border: 1 char on each side)
        let inner_area = Rect::new(
            size.x + 1,
            size.y + 1,
            size.width.saturating_sub(2),
            size.height.saturating_sub(2),
        );

        // Calculate sidebar width with constraints (min ~25 chars, max ~40%)
        // But ensure we don't exceed available width
        // The final constraint ensures main area gets at least 10 characters
        let sidebar_width = if sidebar_collapsed {
            0
        } else {
            let requested_width = (inner_area.width * sidebar_width_percent) / 100;
            let min_width = 25;
            let max_width = (inner_area.width * 40) / 100;
            
            // Ensure sidebar doesn't exceed available space
            // Main area needs at least 10 characters, so sidebar is limited accordingly
            requested_width.max(min_width).min(max_width).min(inner_area.width.saturating_sub(10))
        };

        // Split vertically: tabs (1 line), content area, filters (3 lines for borders + content), status (1 line)
        // Following ratatui example: tabs render in 1 line, content has borders that connect visually
        // For small terminals, make filters flexible to prevent clipping
        // Fixed elements: tabs (1) + status (1) = 2 lines
        // Filters ideally need 3 lines, but can shrink to 1 for very small terminals
        // Content gets whatever is left (minimum 1 line)
        let tabs_height = 1;
        let status_height = 1;
        let ideal_filters_height = 3;
        let min_filters_height = 1;
        
        // Calculate available height after fixed elements
        let available_after_fixed = inner_area.height.saturating_sub(tabs_height + status_height);
        
        // Determine filters height and content height based on available space
        // Critical: content_height + filters_height must equal available_after_fixed exactly
        // Content must get at least 1 line
        let (filters_height, content_height) = if available_after_fixed >= ideal_filters_height + 1 {
            // Enough space: use ideal height for filters, rest for content
            (ideal_filters_height, available_after_fixed - ideal_filters_height)
        } else if available_after_fixed >= min_filters_height + 1 {
            // Limited space: use minimum for filters, rest for content
            (min_filters_height, available_after_fixed - min_filters_height)
        } else {
            // Very limited space: prioritize content (needs at least 1 line)
            // Filters get whatever is left (could be 0 if available_after_fixed = 1)
            // If available_after_fixed = 1: filters = 0, content = 1
            // If available_after_fixed = 2: filters = 1, content = 1
            let content = 1;
            let filters = available_after_fixed.saturating_sub(content);
            (filters, content)
        };
        
        let vertical = RatLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(tabs_height), // Tabs
                Constraint::Length(content_height), // Content (explicit height to prevent overflow)
                Constraint::Length(filters_height), // Filters (flexible based on available space)
                Constraint::Length(status_height), // Status
            ])
            .split(inner_area);

        // Split content area horizontally: sidebar, main
        let horizontal = RatLayout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(sidebar_width),
                Constraint::Min(1),
            ])
            .split(vertical[1]);

        Self {
            inner_area,
            tabs_area: vertical[0],
            sidebar_area: horizontal[0],
            main_area: horizontal[1],
            filters_area: vertical[2],
            status_area: vertical[3],
        }
    }
}

