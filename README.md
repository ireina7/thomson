# Thomson
Toml to Json with key rules.

Tired of using `settings.json` to manage VSCode configurations?
Let's write modular *TOML* configurations which can be later compiled into valid json files!

## JSON key rules
VSCode's JSON keys cannot be interrupted.
For example, key `terminal.integrated.fontSize` is an atomic key, but in *TOML*, we like the ability to rearrange these keys(`terminal`, `integrated`, `fontSize`). For example, a typical *TOML* config can be like:
```toml
# Includes other modules
include = [
    "vim", "latex", "etc"
]

[window]
newWindowDimensions = "maximized"
zoomLevel = 0.8

[editor]
fontSize = 14
fontFamily = "'JetBrains Mono', Monaco, Menlo, 'Courier New', monospace"
fontLigatures = true
tabSize = 4
renderWhitespace = "selection"
detectIndentation = true
cursorStyle = "block"
bracketPairColorization.enabled = true
guides.bracketPairs = "active"
cursorSmoothCaretAnimation = "explicit"
unicodeHighlight.ambiguousCharacters = false
smoothScrolling = true
wordWrap = "on"
semanticTokenColorCustomizations.enabled = true

# ...
```
which can be compiled to (ignoring included tomls for now):
```json
{
  "editor.bracketPairColorization.enabled": true,
  "editor.cursorSmoothCaretAnimation": "explicit",
  "editor.cursorStyle": "block",
  "editor.detectIndentation": true,
  "editor.fontFamily": "'JetBrains Mono', Monaco, Menlo, 'Courier New', monospace",
  "editor.fontLigatures": true,
  "editor.fontSize": 14,
  "editor.guides.bracketPairs": "active",
  "editor.renderWhitespace": "selection",
  "editor.semanticTokenColorCustomizations": { "enabled": true },
  "editor.smoothScrolling": true,
  "editor.tabSize": 4,
  "editor.unicodeHighlight.ambiguousCharacters": false,
  "editor.wordWrap": "on",
  
  "window.newWindowDimensions": "maximized",
  "window.zoomLevel": 0.8
}
```

**Thomson** utilize *JSON rules* to compile your *TOML* files into single valid `settings.json`.

## Modular includings
You can write you *TOML* files in multiple files. **Thomson** can include them recurrently(see examples).

## Example
```
bash ./examples/vscode.bash
```

