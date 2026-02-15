use std::path::PathBuf;

use clap::{Parser, Subcommand};
use colored::*;

use crate::{
    models::task::{When, WhenInstantiationError},
    services::{
        areas::{CreateAreaError, CreateAreaParameters, create_area},
        tasks::{
            AddTaskError, AddTaskParameters, CompleteTaskError, CompleteTaskParameters, add_task,
            complete_task,
        },
    },
    storage::{Storage, json::JsonFileStorage},
};

mod models;
mod services;
mod storage;

#[derive(Parser)]
#[command(
    name = "tdo",
    about = "A minimal and clean task manager for your terminal"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List tasks in the inbox
    Inbox,

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

        /// Assign to an area
        #[arg(short, long)]
        area: Option<String>,

        /// Add tags (can be used multiple times)
        #[arg(short, long, action = clap::ArgAction::Append)]
        tag: Vec<String>,

        /// Add notes
        #[arg(short, long)]
        notes: Option<String>,
    },

    /// Complete a task
    Done { task_number_or_fuzzy_name: String },

    /// Manage areas
    #[command(subcommand)]
    Area(AreaCommands),
}

#[derive(Debug, Subcommand)]
enum AreaCommands {
    /// Create a new area
    New { name: String },
    /// Delete an area
    Delete { name: String },
}

fn main() {
    let cli = Cli::parse();

    // Initialize storage
    let storage_path = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("tdo")
        .join("store.json");

    // Create parent directory if it doesn't exist
    if let Some(parent) = storage_path.parent() {
        std::fs::create_dir_all(parent).unwrap_or_else(|e| {
            eprintln!("Error: Failed to create data directory: {}", e);
            std::process::exit(1);
        });
    }

    let storage = JsonFileStorage::new(storage_path);

    let mut store = match storage.load() {
        Ok(store) => store,
        Err(e) => {
            eprintln!("Error: Failed to load store: {}", e);
            std::process::exit(1);
        }
    };

    match cli.command {
        Some(Commands::Inbox) => {
            // Filter inbox tasks
            let inbox_tasks: Vec<_> = store
                .get_active_tasks()
                .filter(|t| matches!(t.when, When::Inbox))
                .filter(|t| t.completed_at.is_none())
                .collect();

            // Display
            if inbox_tasks.is_empty() {
                println!("Inbox is empty");
            } else {
                println!("{} ({} tasks)\n", "INBOX".cyan(), inbox_tasks.len());
                for task in inbox_tasks {
                    println!(
                        "{} {} {}",
                        "[ ]".green(),
                        format!("#{}", task.task_number).dimmed(),
                        task.title.bold()
                    );

                    // Build metadata line: Project/Area • tags
                    let mut meta_parts = vec![];

                    // Show Project (Area) if task has project, else show Area if task has area
                    if let Some(project_id) = task.project_id {
                        if let Some(project) = store.get_project(project_id) {
                            if let Some(area_id) = project.area_id {
                                if let Some(area) = store.get_area(area_id) {
                                    meta_parts.push(
                                        format!("{} ({})", project.name, area.name)
                                            .blue()
                                            .to_string(),
                                    );
                                } else {
                                    meta_parts.push(project.name.blue().to_string());
                                }
                            } else {
                                meta_parts.push(project.name.blue().to_string());
                            }
                        }
                    } else if let Some(area_id) = task.area_id
                        && let Some(area) = store.get_area(area_id)
                    {
                        meta_parts.push(area.name.blue().to_string());
                    }

                    // Add tags
                    if !task.tags.is_empty() {
                        meta_parts.push(task.tags.join(", "));
                    }

                    // Print metadata line if there's anything to show
                    if !meta_parts.is_empty() {
                        println!("    {}", meta_parts.join(&format!(" {} ", "•".dimmed())));
                    }

                    // Print separator
                    println!("    {}", "─".repeat(30).dimmed());
                    println!();
                }
            }
        }
        Some(Commands::Add {
            title,
            today,
            evening,
            someday,
            anytime,
            when: when_str,
            deadline,
            project,
            area,
            tag,
            notes,
        }) => {
            // Parse when flags
            let when = match When::from_command_flags(today, evening, someday, anytime, when_str) {
                Ok(w) => w,
                Err(WhenInstantiationError::ScheduleAtIncorrect(date_str)) => {
                    eprintln!("Error: Invalid schedule date format: '{}'", date_str);
                    eprintln!(
                        "\nExpected format: YYYY-MM-DD (e.g., 2025-03-01) or relative dates like 'friday', 'next monday'"
                    );
                    std::process::exit(1);
                }
                Err(WhenInstantiationError::ConflictingFlags(flags)) => {
                    eprintln!("Error: Cannot use multiple scheduling flags together");
                    eprintln!("\nConflicting flags provided: {}", flags.join(", "));
                    eprintln!("\nPlease use only one of:");
                    eprintln!("  --today       Schedule for today");
                    eprintln!("  --someday     Defer to someday");
                    eprintln!("  --anytime     Available anytime");
                    eprintln!("  --when DATE   Schedule for a specific date");
                    std::process::exit(1);
                }
                Err(WhenInstantiationError::EveningWithoutToday) => {
                    eprintln!("Error: The --evening flag can only be used with --today");
                    eprintln!("\nExample: tdo add 'Review PRs' --today --evening");
                    std::process::exit(1);
                }
            };

            // Build parameters
            let params = AddTaskParameters {
                title: title.clone(),
                notes,
                when,
                deadline,
                project,
                area,
                tags: tag,
            };

            // Call service
            match add_task(&mut store, &storage, params) {
                Ok(task) => {
                    println!("✓ Task added: {}", task.title);
                    println!("  #{}", task.task_number);
                    if let Some(project_id) = task.project_id
                        && let Some(project) = store.get_project(project_id)
                    {
                        println!("  Project: {}", project.name);
                    }
                }
                Err(AddTaskError::ProjectNotFound(name)) => {
                    eprintln!("Error: Project '{}' not found", name);

                    // Suggest existing projects if any
                    let projects: Vec<_> = store.projects.values().collect();
                    if !projects.is_empty() {
                        eprintln!("\nAvailable projects:");
                        for project in projects {
                            eprintln!("  - {}", project.name);
                        }
                    } else {
                        eprintln!("\nNo projects exist yet. Create one first or omit --project.");
                    }
                    std::process::exit(1);
                }
                Err(AddTaskError::AmbiguousProjectName(names)) => {
                    eprintln!("Error: Project name is ambiguous. Multiple projects found:");
                    for name in names {
                        eprintln!("  - {}", name);
                    }
                    eprintln!("\nPlease be more specific.");
                    std::process::exit(1);
                }
                Err(AddTaskError::AreaNotFound(name)) => {
                    eprintln!("Error: Area '{}' not found", name);

                    // Suggest existing areas if any
                    let areas: Vec<_> = store.areas.values().collect();
                    if !areas.is_empty() {
                        eprintln!("\nAvailable areas:");
                        for area in areas {
                            eprintln!("  - {}", area.name);
                        }
                    } else {
                        eprintln!("\nNo areas exist yet. Create one first or omit --area.");
                    }
                    std::process::exit(1);
                }
                Err(AddTaskError::AmbiguousAreaName(names)) => {
                    eprintln!("Error: Area name is ambiguous. Multiple areas found:");
                    for name in names {
                        eprintln!("  - {}", name);
                    }
                    eprintln!("\nPlease be more specific.");
                    std::process::exit(1);
                }
                Err(AddTaskError::InvalidDeadline(date_str, error)) => {
                    eprintln!("Error: Invalid deadline '{}': {}", date_str, error);
                    eprintln!("\nExpected format: YYYY-MM-DD (e.g., 2025-03-01)");
                    std::process::exit(1);
                }
                Err(AddTaskError::Storage(e)) => {
                    eprintln!("Error: Failed to save task: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Done {
            task_number_or_fuzzy_name,
        }) => {
            // Build parameters
            let params = CompleteTaskParameters {
                task_number_or_fuzzy_name,
            };

            // Call service
            match complete_task(&mut store, &storage, params) {
                Ok(task) => {
                    println!("✓ Task completed: {}", task.title);
                    println!("  #{}", task.task_number);
                }
                Err(CompleteTaskError::TaskNotFound(identifier)) => {
                    eprintln!("Error: Task '{}' not found", identifier);
                    std::process::exit(1);
                }
                Err(CompleteTaskError::AmbiguousTaskName(titles)) => {
                    eprintln!("Error: Task name is ambiguous. Multiple tasks found:");
                    for title in titles {
                        eprintln!("  - {}", title);
                    }
                    eprintln!("\nPlease be more specific or use the task number.");
                    std::process::exit(1);
                }
                Err(CompleteTaskError::Storage(e)) => {
                    eprintln!("Error: Failed to save task: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Area(AreaCommands::New { name })) => {
            let params = CreateAreaParameters { name };
            match create_area(&mut store, &storage, params) {
                Ok(area) => {
                    println!("✓ Area {} created with slug {}", area.name, area.slug);
                }
                Err(CreateAreaError::AreaAlreadyExists(name)) => {
                    eprintln!("Error: Area with name '{}' already exists", name);
                    std::process::exit(1);
                }
                Err(CreateAreaError::Storage(e)) => {
                    eprintln!("Error: Failed to create area: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Area(AreaCommands::Delete { name })) => {
            todo!()
        }
        None => {
            // Default: show today view
            println!("Showing today's tasks...");
        }
    }
}
