# portit demo
Output static/demo.gif

Require portit

Set Shell "bash"
Set FontSize 16
Set Width 1000
Set Height 500
Set Theme "Catppuccin Mocha"
Set Padding 20
Set Framerate 30
Set TypingSpeed 80ms
Set PlaybackSpeed 1

# Launch portit
Type "portit"
Sleep 500ms
Enter
Sleep 2s

# Navigate down through the port list
Down
Sleep 600ms
Down
Sleep 600ms
Down
Sleep 600ms

# Navigate back up
Up
Sleep 600ms
Up
Sleep 600ms

# Open action menu on selected entry
Enter
Sleep 1.5s

# Close action menu
Escape
Sleep 1s

# Enter filter mode
Type "/"
Sleep 500ms

# Type a filter query
Type "node"
Sleep 1s

# Apply filter
Enter
Sleep 1.5s

# Clear filter with Esc in filter mode
Type "/"
Sleep 300ms
Escape
Sleep 1.5s

# Refresh the list
Type "r"
Sleep 1.5s

# Quit
Type "q"
Sleep 1s
