use clap::{Parser, Subcommand};

use crate::models::create_when_from_command_flags;

mod models;
mod services;

#[derive(Parser)]
#[command(name = "tdo", about = "A Things 3-inspired task manager")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task
    Add {
        /// Task title
        title: String,

        /// Schedule for today
        #[arg(long)]
        today: bool,

        /// Schedule for today (evening)
        #[arg(long)]
        evening: bool,

        /// Defer to someday
        #[arg(long)]
        someday: bool,

        /// Available anytime (no specific date)
        #[arg(long)]
        anytime: bool,

        /// Schedule for a specific date (e.g., "friday", "2025-03-01")
        #[arg(short, long)]
        when: Option<String>,

        /// Set a hard deadline
        #[arg(short, long)]
        deadline: Option<String>,

        /// Assign to a project
        #[arg(short, long)]
        project: Option<String>,

        /// Add tags (can be used multiple times)
        #[arg(short, long, action = clap::ArgAction::Append)]
        tag: Vec<String>,

        /// Add notes
        #[arg(short, long)]
        notes: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Add {
            title,
            today,
            evening,
            someday,
            anytime,
            when,
            deadline,
            project,
            tag,
            notes,
        }) => {
            println!("Adding task: {}", title);

            if today {
                println!("  → Today");
            }
            if evening {
                println!("  → Evening");
            }
            if someday {
                println!("  → Someday");
            }
            if anytime {
                println!("  → Anytime");
            }
            if let Some(ref w) = when {
                println!("  → When: {}", w);
            }
            let when = create_when_from_command_flags(today, evening, someday, anytime, when);

            if let Some(d) = deadline {
                println!("  → Deadline: {}", d);
            }
            if let Some(p) = project {
                println!("  → Project: {}", p);
            }
            if !tag.is_empty() {
                println!("  → Tags: {:?}", tag);
            }
            if let Some(n) = notes {
                println!("  → Notes: {}", n);
            }
        }
        None => {
            // Default: show today view
            println!("Showing today's tasks...");
        }
    }
}
