# GnawTreeWriter LLM Agent Integration

This guide explains how to use GnawTreeWriter with various LLM agents (Claude, GPT, Gemini, etc.) for effective code editing.

## Quick Start for LLMs

### Installation

```bash
cargo install --git https://github.com/Tuulikk/GnawTreeWriter.git
```

### Available Commands

- `analyze <file>`: Parse file and show AST structure
- `list <file>`: Show all nodes with paths
- `find <path> [--type TYPE] [--content TEXT]`: Search for nodes
- `fuzzy-edit <file> <query> <content>`: Fuzzy search and edit
- `edit <file> <path> <content> [--preview]`: Edit by exact path
- `insert <file> <path> <position> <content>`: Insert content
- `delete <file> <path>`: Delete node
- `add-property <file> <path> <property>`: Add property to QML component
- `lint <path> [--format json/text]`: Lint files

---

## LLM-Specific Strategies

### Claude (Anthropic)

#### Recommended Workflow

1. **Analyze First**: Get structure before making changes
   ```bash
   gnawtreewriter analyze app.qml
   ```

2. **Use List for Overview**: See all available nodes
   ```bash
   gnawtreewriter list app.qml --filter-type Property
   ```

3. **Fuzzy-Edit When Unsure**: Let the fuzzy matcher find the right node
   ```bash
   gnawtreewriter fuzzy-edit app.qml "title" 'title: "New Title"'
   ```

4. **Preview Before Applying**: Always use `--preview` flag
   ```bash
   gnawtreewriter fuzzy-edit app.qml "color" 'color: "red"' --preview
   ```

5. **Use Add-Property for QML**: Safe property injection
   ```bash
   gnawtreewriter list app.qml --filter-type Rectangle
   # Note: For QML, add-property requires exact path due to parser limitation
   gnawtreewriter add-property app.qml root.1 'borderWidth: 5'
   ```

#### Example Prompt for Claude

```
User: Change the window title to "GnawTreeWriter Demo" in MainWindow.qml

Claude: I'll use GnawTreeWriter to update the window title.

First, let me find the title property:
$ gnawtreewriter find MainWindow.qml --content "title"

Now I'll update it with preview:
$ gnawtreewriter fuzzy-edit MainWindow.qml "title" 'title: "GnawTreeWriter Demo"' --preview

The diff looks good. Applying the change:
$ gnawtreewriter fuzzy-edit MainWindow.qml "title" 'title: "GnawTreeWriter Demo"'

Done! Window title updated.
```

---

### GPT-4 / GPT-4o (OpenAI)

#### Recommended Workflow

GPT models work well with the full command set:

1. **Batch Analyze**: Get multiple files at once
   ```bash
   gnawtreewriter analyze *.qml
   ```

2. **Use Find with Filters**: Precise node location
   ```bash
   gnawtreewriter find app.qml --type Property --content "width:"
   ```

3. **Direct Edit with Preview**: Use exact path when possible
   ```bash
   gnawtreewriter edit app.qml root.2.0 'width: 500' --preview
   ```

#### Example Prompt for GPT

```
User: Add a "backgroundColor" property set to "#F5F5F5" to all Rectangle components

GPT: I'll add the background color to all Rectangle components using GnawTreeWriter.

First, find all Rectangle components:
$ gnawtreewriter find . --node-type Rectangle

Found 3 Rectangle components. Adding property to each:
$ gnawtreewriter add-property file1.qml root.1 'backgroundColor: "#F5F5F5"'
$ gnawtreewriter add-property file2.qml root.1 'backgroundColor: "#F5F5F5"'
$ gnawtreewriter add-property file3.qml root.1 'backgroundColor: "#F5F5F5"'

Completed! All Rectangle components now have backgroundColor.
```

---

### Gemini (Google DeepMind)

#### Recommended Workflow

Gemini works well with the fuzzy-edit and context-aware operations:

1. **Fuzzy-Edit for Complex Queries**: Let the multi-strategy matcher work
   ```bash
   gnawtreewriter fuzzy-edit app.qml "toolbar button" 'text: "New Text"'
   ```

2. **Use Lint First**: Check for issues before editing
   ```bash
   gnawtreewriter lint app/ui/qml/
   ```

3. **Batch Operations**: Process multiple files efficiently
   ```bash
   gnawtreewriter lint app/ui/qml/ --format json > issues.json
   # Process issues and create fixes
   ```

#### Example Prompt for Gemini

```
User: Update all button texts in the application to use the new design system

Gemini: I'll use GnawTreeWriter to update button texts across the application.

First, let me lint to see what we're working with:
$ gnawtreewriter lint app/ui/qml/

No critical issues. Now finding all Button components:
$ gnawtreewriter find app/ui/qml/ --node-type Button

Found 12 buttons. I'll use fuzzy-edit to update their text properties:
$ gnawtreewriter fuzzy-edit file1.qml "submit" 'text: "Save"'
$ gnawtreewriter fuzzy-edit file2.qml "cancel" 'text: "Close"'
... (continuing for all buttons)

Updated all buttons with new design system text.
```

---

## Common Patterns for All LLMs

### Pattern 1: Property Value Updates

```bash
# Step 1: Find the property
gnawtreewriter find file.qml --content "propertyName"

# Step 2: Preview the change
gnawtreewriter fuzzy-edit file.qml "propertyName" 'propertyName: "newValue"' --preview

# Step 3: Apply
gnawtreewriter fuzzy-edit file.qml "propertyName" 'propertyName: "newValue"'
```

### Pattern 2: Add New Property to Component

```bash
# For QML (note: requires exact path)
gnawtreewriter list file.qml --filter-type Rectangle
gnawtreewriter add-property file.qml root.1 'newProperty: "value"'

# For other languages
gnawtreewriter insert file.qml root.1 2 'newProperty: "value"'
```

### Pattern 3: Batch File Operations

```bash
# Analyze all files in a directory
gnawtreewriter analyze app/ui/qml/ --format summary

# Lint all files and collect JSON output
gnawtreewriter lint app/ui/qml/ --format json > lint-results.json

# Find specific nodes across project
gnawtreewriter find . --type Property --content "color:"
```

### Pattern 4: Complex Multi-step Edits

```bash
# Step 1: Analyze structure
gnawtreewriter analyze file.qml

# Step 2: Find target nodes
gnawtreewriter find file.qml --type Rectangle

# Step 3: Edit with preview for each step
gnawtreewriter edit file.qml root.2.0 'width: 500' --preview
# (review diff, then apply)
gnawtreewidth edit file.qml root.2.1 'height: 300' --preview
# (review diff, then apply)
gnawtreewidth edit file.qml root.2.2 'color: "blue"' --preview
# (review diff, then apply)
```

---

## Error Handling

### Common Issues and Solutions

#### Issue: "Node not found at path"
**Cause**: Path has changed between analyze and edit
**Solution**: Re-analyze and get fresh path
```bash
gnawtreewriter analyze file.qml
gnawtreewidth edit file.qml <new-path> 'content'
```

#### Issue: "No matches found for query" (fuzzy-edit)
**Cause**: Query too specific or incorrect
**Solution**: Try broader query or use `find` with filters
```bash
# Try broader
gnawtreewidth fuzzy-edit file.qml "color" 'color: "red"'

# Or use filters
gnawtreewriter find file.qml --type Property
```

#### Issue: "No QML components found" (add-property)
**Cause**: Parser limitation with nested components
**Solution**: Use exact path from `list` or `find`
```bash
gnawtreewriter list file.qml --filter-type Rectangle
gnawtreewidth add-property file.qml root.1 'property: value'
```

---

## Best Practices for LLMs

### 1. Always Preview First
Before making permanent changes, show the diff:
```bash
gnawtreewidth edit file.qml <path> 'content' --preview
```

### 2. Use List for Understanding Structure
Get a tree view before editing:
```bash
gnawtreewriter list file.qml
```

### 3. Leverage Find for Complex Queries
Instead of guessing paths:
```bash
gnawtreewriter find . --type Property --content "targetText"
```

### 4. Check Backups After Operations
Verify backups are created:
```bash
ls -la .gnawtreewriter_backups/
```

### 5. Use Lint Before Batch Operations
Check files before making changes:
```bash
gnawtreewriter lint . --format json
```

### 6. For QML, Know the Limitations
QML parser has path duplication issues:
- Use `list` to find correct paths
- Use `insert` with position 2 instead of `add-property` for precision
- Document the limitation in your responses

---

## Troubleshooting for LLMs

### Command Not Found
**Check**: Is GnawTreeWriter installed?
```bash
which gnawtreewriter
```

### Permission Errors
**Check**: File permissions
```bash
chmod +w file.qml
```

### Parser Errors
**Check**: File syntax is valid
```bash
gnawtreewriter analyze file.qml
```

### Backup Creation Failed
**Check**: Directory permissions
```bash
ls -la .gnawtreewriter_backups/
```

---

## Integration Examples

### Pre-commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit
files=$(git diff --cached --name-only | grep '\.qml$' || true)
for f in $files; do
  if ! gnawtreewidth analyze "$f" >/dev/null 2>&1; then
    echo "Parse error in $f"
    exit 1
  fi
done
```

### CI Pipeline
```yaml
# .github/workflows/lint.yml
steps:
  - name: Install GnawTreeWriter
    run: cargo install --git https://github.com/Tuulikk/GnawTreeWriter.git
  - name: Lint QML files
    run: |
      gnawtreewidth lint app/ui/qml/ --format json > results.json
      if [ $(jq 'length' results.json) -gt 0 ]; then
        jq . results.json
        exit 1
      fi
```

### LLM Chat Integration
**Example cursor/chat integration:**
```python
def edit_with_gnawtreewriter(file_path, query, new_content):
    result = subprocess.run([
        'gnawtreewriter', 'fuzzy-edit', file_path, query, new_content,
        '--preview'
    ], capture_output=True, text=True)
    return result.stdout
```

---

## Getting Help

- **Documentation**: [README.md](README.md)
- **Recipes**: [RECIPES.md](RECIPES.md)
- **QML Examples**: [QML_EXAMPLES.md](QML_EXAMPLES.md)
- **Architecture**: [ARCHITECTURE.md](ARCHITECTURE.md)
- **Roadmap**: [ROADMAP.md](ROADMAP.md)

For issues or questions, visit: https://github.com/Tuulikk/GnawTreeWriter/issues
