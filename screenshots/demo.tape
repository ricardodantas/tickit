# VHS tape for Tickit screenshots
Output screenshots/01-tasks.png

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 800
Set Theme "Dracula"

Type "cd ~/dev/ricardodantas/tickit && cargo run --release 2>/dev/null"
Enter
Sleep 3s

# Screenshot 1: Tasks view (empty initially, will populate)
Screenshot screenshots/01-tasks.png
Sleep 500ms

# Add sample tasks - press 'n' to create new task
Type "n"
Sleep 500ms
Screenshot screenshots/07-task-editor.png
Type "Review pull request for authentication module"
Sleep 200ms
Tab
Type "Check the OAuth implementation and security headers"
Sleep 200ms
Tab
Sleep 100ms
# Set priority to High
Type "j"
Type "j"
Sleep 200ms
Tab
Sleep 200ms
Enter
Sleep 500ms

# Add another task
Type "n"
Sleep 300ms
Type "Buy groceries"
Tab
Type "Milk, eggs, bread, cheese"
Tab
Tab
Tab
Enter
Sleep 500ms

# Add a completed task
Type "n"
Sleep 300ms
Type "Send weekly report"
Tab
Type "Q4 summary for stakeholders"
Tab
Type "j"
Sleep 100ms
Tab
Tab
Enter
Sleep 500ms

# Toggle it complete
Type "x"
Sleep 300ms

# Add urgent task
Type "n"
Sleep 300ms
Type "Fix production bug"
Tab
Type "Memory leak in auth service"
Tab
Type "jjj"
Sleep 100ms
Tab
Tab
Enter
Sleep 500ms

# Screenshot updated tasks view
Type "g"
Sleep 300ms
Screenshot screenshots/01-tasks.png

# Go to Lists view
Type "2"
Sleep 500ms
Screenshot screenshots/02-lists.png

# Add a list
Type "n"
Sleep 300ms
Type "Work"
Enter
Sleep 300ms

Type "n"
Sleep 300ms
Type "Personal"
Enter
Sleep 300ms

Type "n"
Sleep 300ms
Type "Shopping"
Enter
Sleep 500ms

Screenshot screenshots/02-lists.png

# Go to Tags view
Type "3"
Sleep 500ms

# Add tags
Type "n"
Sleep 300ms
Type "urgent"
Enter
Sleep 300ms

Type "n"
Sleep 300ms
Type "bug"
Enter
Sleep 300ms

Type "n"
Sleep 300ms
Type "feature"
Enter
Sleep 500ms

Screenshot screenshots/03-tags.png

# Theme picker
Type "t"
Sleep 500ms
Screenshot screenshots/04-theme-picker.png
Escape
Sleep 300ms

# Help dialog
Type "?"
Sleep 500ms
Screenshot screenshots/05-help.png
Escape
Sleep 300ms

# About dialog
Type "A"
Sleep 500ms
Screenshot screenshots/06-about.png
Escape
Sleep 300ms

# Back to tasks
Type "1"
Sleep 500ms
Screenshot screenshots/01-tasks.png

# Quit
Type "q"
Sleep 500ms
