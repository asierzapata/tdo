use colored::*;
use jiff::civil::Date;

use crate::models::{store::Store, task::Task};

/// Get the terminal width, defaulting to 80 if unavailable
fn get_terminal_width() -> usize {
    term_size::dimensions().map(|(w, _)| w).unwrap_or(80)
}

/// Get the appropriate status glyph for a task
pub fn get_status_glyph(task: &Task, is_overdue: bool) -> ColoredString {
    if task.completed_at.is_some() {
        "✓".dimmed()
    } else if is_overdue {
        "●".red()
    } else {
        "○".normal()
    }
}

/// Build the context string for a task (Area/Project hierarchy)
/// Returns None if task has no area or project associations
pub fn get_task_context(task: &Task, store: &Store) -> Option<String> {
    // Rule A & C: If task has a project, show it (with area if project has one)
    if let Some(project_id) = task.project_id {
        if let Some(project) = store.get_project(project_id) {
            if let Some(area_id) = project.area_id {
                if let Some(area) = store.get_area(area_id) {
                    // Rule A: {Area Name} / {Project Name}
                    return Some(format!("{} / {}", area.name, project.name));
                }
            }
            // Rule C: {Project Name} (no area)
            return Some(project.name.clone());
        }
    }

    // Rule B: Task has area but no project
    if let Some(area_id) = task.area_id {
        if let Some(area) = store.get_area(area_id) {
            return Some(area.name.clone());
        }
    }

    // Rule D: No associations
    None
}

/// Render a single task line with ID, glyph, title, and right-aligned context
pub fn render_task_line(task: &Task, store: &Store, is_overdue: bool) {
    let terminal_width = get_terminal_width();

    // Build left section: ID + glyph + title
    let id_str = format!("{:>3}", task.task_number);
    let glyph = get_status_glyph(task, is_overdue);
    let title = &task.title;

    // Build right section: context (if any)
    let context = get_task_context(task, store);

    // Calculate spacing needed for right alignment
    // Format: "  ID  GLYPH  TITLE" + spaces + "CONTEXT"
    let left_section = format!("  {}  {}  {}", id_str, glyph, title);

    // Apply completed styling if needed
    let styled_left = if task.completed_at.is_some() {
        left_section.dimmed().strikethrough()
    } else {
        left_section.bold()
    };

    if let Some(ctx) = context {
        let ctx_dimmed = ctx.dimmed();

        // Calculate the visible length (without ANSI codes)
        let left_visible_len = format!("  {}  {}  {}", id_str, " ", title).len();
        let ctx_visible_len = ctx.len();

        // Calculate padding needed
        let total_content = left_visible_len + ctx_visible_len;

        if total_content + 4 < terminal_width {
            let padding = terminal_width - total_content - 2;
            println!("{}{}{}", styled_left, " ".repeat(padding), ctx_dimmed);
        } else {
            // Not enough space for right alignment, just print normally
            println!("{}", styled_left);
        }
    } else {
        println!("{}", styled_left);
    }
}

/// Render a view header with title and count
pub fn render_view_header(title: &str, count: usize) {
    let task_word = if count == 1 { "task" } else { "tasks" };
    println!("\n  {} ({} {})\n", title.cyan().bold(), count, task_word);
}

/// Render a section header (e.g., "Evening", "Tomorrow")
pub fn render_section_header(title: &str) {
    println!("\n  ─── {} ───\n", title.bold());
}

/// Render a section separator
pub fn render_section_separator() {
    println!();
}

/// Check if a task is overdue
pub fn is_overdue(task: &Task) -> bool {
    if task.completed_at.is_some() || task.deleted_at.is_some() {
        return false;
    }

    if let crate::models::task::When::Scheduled { date } = task.when {
        let today = jiff::Zoned::now().date();
        return date < today;
    }

    false
}

/// Check if a timestamp is within the last N days
pub fn is_within_days(timestamp: jiff::Timestamp, days: i64) -> bool {
    let now = jiff::Timestamp::now();
    let duration = jiff::SignedDuration::from_hours(days * 24);

    if let Ok(threshold) = now.checked_sub(duration) {
        timestamp >= threshold
    } else {
        false
    }
}

/// Format a date as a human-readable header (e.g., "Tomorrow", "Monday, Feb 17")
pub fn format_date_header(date: Date) -> String {
    let today = jiff::Zoned::now().date();

    if date == today {
        "Today".to_string()
    } else if date == today.tomorrow().expect("tomorrow should be valid") {
        "Tomorrow".to_string()
    } else {
        // Format as "Monday, Feb 17"
        date.strftime("%A, %b %d").to_string()
    }
}
