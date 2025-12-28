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
        // Ensure minimum terminal size (accounting for outer border)
        let min_width_with_border = Self::MIN_WIDTH + 2; // +2 for left/right borders
        let min_height_with_border = Self::MIN_HEIGHT + 2; // +2 for top/bottom borders
        let width = size.width.max(min_width_with_border);
        let height = size.height.max(min_height_with_border);
        let size = Rect::new(size.x, size.y, width, height);

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
        let vertical = RatLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Tabs
                Constraint::Min(1),    // Content (sidebar + main)
                Constraint::Length(3), // Filters (needs borders + content)
                Constraint::Length(1), // Status
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

