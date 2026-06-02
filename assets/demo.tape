Require stevedore

Output assets/demo.gif

Set Shell bash
Set FontSize 16
Set Width 1280
Set Height 720
Set Padding 20
Set Theme "Catppuccin Mocha"
Set TypingSpeed 80ms

# Launch
Type "stevedore"
Sleep 500ms
Enter
Sleep 2.5s

# Browse the container list
Down@600ms 3
Sleep 1s
Up@600ms 1
Sleep 1s

# Open logs for the selected container
Enter
Sleep 3s

# Scroll back through the logs, then jump to the bottom and follow again
Up@250ms 6
Sleep 1.5s
Type "G"
Sleep 1.5s

# Back to the list
Escape
Sleep 1s

# Show all containers (running + stopped)
Type "a"
Sleep 2s

# Help overlay
Type "?"
Sleep 2.5s
Type "?"
Sleep 500ms

# Quit
Type "q"
Sleep 1s
