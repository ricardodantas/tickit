//! Desktop notifications for task reminders

use crate::models::{Priority, Task};
use notify_rust::{Notification, Timeout};

/// Send a notification for a task that's due today
pub fn notify_task_due_today(task: &Task) -> Result<(), notify_rust::error::Error> {
    let priority_emoji = match task.priority {
        Priority::Urgent => "🔴",
        Priority::High => "🟠",
        Priority::Medium => "🟡",
        Priority::Low => "🟢",
    };

    Notification::new()
        .summary(&format!("{} Task Due Today", priority_emoji))
        .body(&task.title)
        .appname("Tickit")
        .timeout(Timeout::Milliseconds(10000))
        .show()?;

    Ok(())
}

/// Send a notification for a task due tomorrow (advance warning)
pub fn notify_task_due_tomorrow(task: &Task) -> Result<(), notify_rust::error::Error> {
    Notification::new()
        .summary("⏰ Task Due Tomorrow")
        .body(&task.title)
        .appname("Tickit")
        .timeout(Timeout::Milliseconds(8000))
        .show()?;

    Ok(())
}

/// Send a notification for overdue tasks
pub fn notify_task_overdue(task: &Task) -> Result<(), notify_rust::error::Error> {
    Notification::new()
        .summary("⚠️ Overdue Task")
        .body(&task.title)
        .appname("Tickit")
        .timeout(Timeout::Milliseconds(10000))
        .show()?;

    Ok(())
}

/// Send a generic notification
pub fn notify(title: &str, body: &str) -> Result<(), notify_rust::error::Error> {
    Notification::new()
        .summary(title)
        .body(body)
        .appname("Tickit")
        .timeout(Timeout::Milliseconds(5000))
        .show()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notify() {
        // Just test it doesn't panic - actual notification depends on system
        let _ = notify("Test", "Test body");
    }
}
