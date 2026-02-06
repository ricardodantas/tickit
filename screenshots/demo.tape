# VHS tape for Tickit screenshots
# Database pre-populated via CLI

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 800
Set Theme "Dracula"

Type "/var/home/ricardodantas/dev/tickit/target/release/tickit"
Enter
Sleep 2s

# Screenshot: Tasks view
Screenshot screenshots/01-tasks.png
Sleep 500ms

# Toggle first task complete
Enter
Sleep 500ms

# Navigate down to show multiple tasks selected
Type "j"
Sleep 300ms

# Screenshot: Tasks with selection
Screenshot screenshots/01-tasks-selection.png
Sleep 300ms

# Open task editor
Type "n"
Sleep 800ms
Screenshot screenshots/07-task-editor.png
Escape
Sleep 500ms

# Go to Lists view
Type "2"
Sleep 800ms
Screenshot screenshots/02-lists.png
Sleep 300ms

# Go to Tags view
Type "3"
Sleep 800ms
Screenshot screenshots/03-tags.png
Sleep 300ms

# Open theme picker
Type "t"
Sleep 800ms
Screenshot screenshots/04-theme-picker.png
Escape
Sleep 500ms

# Open help
Type "?"
Sleep 800ms
Screenshot screenshots/05-help.png
Escape
Sleep 500ms

# Open about
Shift+A
Sleep 800ms
Screenshot screenshots/06-about.png
Escape
Sleep 500ms

# Quit
Type "q"
Sleep 500ms
