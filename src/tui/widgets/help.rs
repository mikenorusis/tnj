use ratatui::widgets::{Block, Borders, Paragraph, Clear};
use ratatui::style::Style;
use ratatui::Frame;
use ratatui::layout::{Rect, Alignment, Constraint, Layout, Flex};
use crate::Config;
use crate::tui::widgets::color::parse_color;

pub fn render_help(f: &mut Frame, area: Rect, config: &Config) {
    let active_theme = config.get_active_theme();
    let fg_color = parse_color(&active_theme.fg);
    let bg_color = parse_color(&active_theme.bg);
    
    // Calculate popup area (60% width, 70% height, centered)
    // Using Layout with Flex::Center for proper centering, following ratatui popup example
    let popup_area = popup_area(area, 60, 70);
    
    // Clear the background first - this prevents content from showing through
    f.render_widget(Clear, popup_area);
    
    // Build help text organized by sections
    let help_text = build_help_text(config);
    
    let paragraph = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Help - Key Bindings")
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(fg_color).bg(bg_color)))
        .style(Style::default().fg(fg_color).bg(bg_color))
        .wrap(ratatui::widgets::Wrap { trim: true });
    
    f.render_widget(paragraph, popup_area);
}

/// Helper function to create a centered rect using up certain percentage of the available rect
/// Based on ratatui popup example: https://ratatui.rs/examples/apps/popup/
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

fn build_help_text(config: &Config) -> String {
    let mut text = String::new();
    
    // Navigation section
    text.push_str("Navigation:\n");
    text.push_str(&format!("  {} / {}: Switch tabs\n", 
        crate::utils::format_key_binding_for_display(&config.key_bindings.tab_left),
        crate::utils::format_key_binding_for_display(&config.key_bindings.tab_right)));
    text.push_str(&format!("  {} / {} / {}: Jump to tab\n", 
        crate::utils::format_key_binding_for_display(&config.key_bindings.tab_1),
        crate::utils::format_key_binding_for_display(&config.key_bindings.tab_2),
        crate::utils::format_key_binding_for_display(&config.key_bindings.tab_3)));
    text.push_str(&format!("  {} / {}: Navigate list up/down\n", 
        crate::utils::format_key_binding_for_display(&config.key_bindings.list_up),
        crate::utils::format_key_binding_for_display(&config.key_bindings.list_down)));
    text.push_str(&format!("  {}: Select item\n", 
        crate::utils::format_key_binding_for_display(&config.key_bindings.select)));
    text.push_str("\n");
    
    // Actions section
    text.push_str("Actions:\n");
    text.push_str(&format!("  {}: New item\n", 
        crate::utils::format_key_binding_for_display(&config.key_bindings.new)));
    text.push_str(&format!("  {}: Edit selected item\n", 
        crate::utils::format_key_binding_for_display(&config.key_bindings.edit)));
    text.push_str(&format!("  {}: Delete selected item\n", 
        crate::utils::format_key_binding_for_display(&config.key_bindings.delete)));
    text.push_str(&format!("  {}: Toggle task status (Tasks tab only)\n", 
        crate::utils::format_key_binding_for_display(&config.key_bindings.toggle_task_status)));
    #[cfg(target_os = "macos")]
    {
        text.push_str("  Opt+↑ / Opt+↓: Reorder task (Tasks tab only)\n");
    }
    #[cfg(not(target_os = "macos"))]
    {
        text.push_str("  Ctrl+↑ / Ctrl+↓: Reorder task (Tasks tab only)\n");
    }
    text.push_str(&format!("  {}: Start search\n", 
        crate::utils::format_key_binding_for_display(&config.key_bindings.search)));
    text.push_str("\n");
    
    // Editor Mode section
    text.push_str("Editor Mode:\n");
    text.push_str(&format!("  {}: Save and exit\n", 
        crate::utils::format_key_binding_for_display(&config.key_bindings.save)));
    text.push_str(&format!("  {}: Undo\n", 
        crate::utils::format_key_binding_for_display(&config.key_bindings.undo)));
    text.push_str(&format!("  {} / {}: Word navigation\n", 
        crate::utils::format_key_binding_for_display(&config.key_bindings.word_left),
        crate::utils::format_key_binding_for_display(&config.key_bindings.word_right)));
    text.push_str("  Arrow keys: Move cursor\n");
    text.push_str("  Shift+Arrow: Extend selection\n");
    text.push_str("  Home/End: Line start/end\n");
    text.push_str("  Backspace: Delete character\n");
    text.push_str("  Enter: Insert newline\n");
    text.push_str("  Esc: Cancel edit\n");
    text.push_str("\n");
    
    // General section
    text.push_str("General:\n");
    text.push_str(&format!("  {}: Quit\n", 
        crate::utils::format_key_binding_for_display(&config.key_bindings.quit)));
    text.push_str(&format!("  {}: Show/hide help\n", 
        crate::utils::format_key_binding_for_display(&config.key_bindings.help)));
    text.push_str(&format!("  {}: Open settings\n", 
        crate::utils::format_key_binding_for_display(&config.key_bindings.settings)));
    text.push_str(&format!("  {}: Toggle sidebar\n", 
        crate::utils::format_key_binding_for_display(&config.key_bindings.toggle_sidebar)));
    
    text
}

