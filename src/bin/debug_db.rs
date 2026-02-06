use tickit::db::Database;

fn main() {
    println!("Opening database...");
    let db = Database::open().expect("Failed to open database");

    println!("\nLists:");
    let lists = db.get_lists().expect("Failed to get lists");
    for list in &lists {
        println!("  - {} {} (inbox: {})", list.icon, list.name, list.is_inbox);
    }

    println!("\nTags:");
    let tags = db.get_tags().expect("Failed to get tags");
    for tag in &tags {
        println!("  - {} ({})", tag.name, tag.color);
    }

    println!("\nAll tasks (including completed):");
    let tasks = db
        .get_tasks_with_filter(None, None, None)
        .expect("Failed to get tasks");
    println!("  Count: {}", tasks.len());
    for task in &tasks {
        println!(
            "  - [{}] {:?} {}",
            if task.completed { "x" } else { " " },
            task.priority,
            task.title
        );
    }

    println!("\nIncomplete tasks only:");
    let incomplete = db
        .get_tasks_with_filter(None, Some(false), None)
        .expect("Failed to get incomplete tasks");
    println!("  Count: {}", incomplete.len());
    for task in &incomplete {
        println!("  - {:?} {}", task.priority, task.title);
    }
}
