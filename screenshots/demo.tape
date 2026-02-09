# VHS tape for Tickit screenshots
# Database pre-populated via CLI

Output demo.gif

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 800
Set Theme "Dracula"

Type "/var/home/ricardodantas/dev/tickit/target/release/tickit"
Enter
Sleep 2s

# Screenshot: Tasks view
Screenshot tasks.png
Sleep 500ms

# Navigate down to show multiple tasks selected
Type "j"
Sleep 300ms

# Screenshot: Tasks with selection
Screenshot tasks_selection.png
Sleep 300ms

# Open task editor
Type "n"
Sleep 800ms
Screenshot task_editor.png
Escape
Sleep 500ms

# Go to Lists view
Type "2"
Sleep 800ms
Screenshot lists.png
Sleep 300ms

# Go to Tags view
Type "3"
Sleep 800ms
Screenshot tags.png
Sleep 300ms

# Open settings
Type "s"
Sleep 800ms
Screenshot settings.png
Escape
Sleep 500ms

# Open theme picker
Type "t"
Sleep 800ms
Screenshot theme_picker.png
Escape
Sleep 500ms

# Open help
Type "?"
Sleep 800ms
Screenshot help.png
Escape
Sleep 500ms

# Open about
Type "A"
Sleep 800ms
Screenshot about.png
Escape
Sleep 500ms

# Quit
Type "q"
Sleep 500ms

# Note: After running vhs, rename screenshots to match convention:
# mv tasks.png 01-tasks.png
# mv tasks_selection.png 01-tasks-selection.png
# mv lists.png 02-lists.png
# mv tags.png 03-tags.png
# mv theme_picker.png 04-theme-picker.png
# mv help.png 05-help.png
# mv about.png 06-about.png
# mv task_editor.png 07-task-editor.png
# mv settings.png 08-settings.png
