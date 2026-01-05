## LLM Integration Guide

This guide explains how to integrate GnawTreeWriter with LLMs for precise, structured code editing.

## Basic Concepts

GnawTreeWriter operates on AST (Abstract Syntax Tree) level, not on raw text. This means LLMs don't need to worry about:
- Bracket matching
- Indentation
- Syntax errors
- Parent-child relationships

## Tree Paths

Nodes are addressed using dot-notation paths:
- `root` - Document root
- `0` - First child of root
- `0.1` - Second child of first root child
- `0.2.1` - Second child of third child of first root child

## LLM Workflow

### 1. Analyze Code Structure

```bash
gnawtreewriter analyze file.qml
```

Response: JSON tree with all nodes, their types, content, and paths.

### 2. Understand Context

LLM analyzes the tree to find target nodes:

```json
{
  "id": "0.1",
  "path": "0.1", 
  "node_type": "Rectangle",
  "content": "...",
  "children": [...]
}
```

### 3. Make Structured Edits

LLM generates edit requests using specific intents:

```json
{
  "file_path": "app.qml",
  "intent": {
    "type": "ReplaceNode",
    "node_path": "0.1",
    "description": "Change width from 200 to 300",
    "new_content": "width: 300"
  }
}
```

## Supported Edit Intents

### ReplaceNode
Replace entire content of a node.

```json
{
  "type": "ReplaceNode",
  "description": "Change button text",
  "node_path": "0.1.0",
  "new_content": "text: \"Click me\""
}
```

### AddProperty
Add a property to a component (QML-specific). Safely handles placement after existing properties.

```json
{
  "type": "AddProperty",
  "description": "Add opacity property",
  "component_path": "0.1",
  "property_name": "opacity",
  "property_value": "0.8"
}
```

### AddComponent
Add a child component to a parent (QML-specific). Handles brackets and indentation.

```json
{
  "type": "AddComponent",
  "description": "Add a button inside the layout",
  "parent_path": "0.1",
  "component_name": "Button",
  "content": "text: 'Submit'"
}
```


### InsertBefore
Insert content before a node.

```json
{
  "type": "InsertBefore",
  "description": "Add margin before text",
  "node_path": "0.1.0",
  "content": "anchors.margins: 10"
}
```

### InsertAfter
Insert content after a node.

```json
{
  "type": "InsertAfter",
  "description": "Add border after text",
  "node_path": "0.1.0",
  "content": "border.width: 2"
}
```

### DeleteNode
Remove a node entirely.

```json
{
  "type": "DeleteNode",
  "description": "Remove old comment",
  "node_path": "0.0"
}
```

## LLM Prompt Guidelines

When asking LLMs to use GnawTreeWriter, include:

```
You are a code editor using GnawTreeWriter tool. 
Work at AST tree level, not raw text.

Steps:
1. First analyze the file: gnawtreewriter analyze <file>
2. Find the target node path from the tree structure
3. Use the appropriate edit intent (ReplaceNode, AddProperty, etc.)
4. Specify exact content for the edit

Do NOT generate full file content.
Do NOT worry about brackets or indentation.
Only specify what to change at which node path.
```

## Example LLM Prompts

### Change a Property
```
I want to change the width of the Rectangle in app.qml from 200 to 300.

First analyze: gnawtreewriter analyze app.qml
Then create an edit request to change the width property.
```

### Add a New Property
```
Add a border to the Rectangle component in app.qml.

Use AddProperty intent on the Rectangle node.
```

### Insert Code Before a Node
```
Add a gradient before the color property in Rectangle.

Use InsertBefore intent with the gradient definition.
```

## Getting Node Context

To understand what surrounds a node:

```bash
gnawtreewriter show app.qml "0.1"
```

This returns the full content of that node for LLM analysis.

## Batch Operations

For multiple changes, use batch mode:

```json
{
  "file_path": "app.qml",
  "operations": [
    {"type": "Edit", "node_path": "0.1", "content": "..."},
    {"type": "Insert", "parent_path": "0.1", "position": 1, "content": "..."}
  ]
}
```

## Language-Specific Considerations

### QML
- Use `AddProperty` intent for component properties
- Properties are children of components
- Components can be nested

### Python
- Functions and classes are top-level nodes
- Use `ReplaceNode` for function bodies
- Indentation is handled automatically

### JavaScript/TypeScript
- Functions, classes, variables are distinct node types
- Use `InsertBefore/After` for adding statements
- Complex expressions have nested structure

### PHP
- Classes and functions are clearly separated
- Method definitions are nested in classes
- Use correct node paths for class methods

### HTML
- Elements are hierarchical
- Attributes are children of element nodes
- Use `InsertBefore` to add new elements

## Error Handling

Always include error messages in responses:

```json
{
  "success": false,
  "message": "Node '0.5' not found",
  "suggestion": "Check the tree structure with analyze command"
}
```

## Best Practices

1. **Always analyze first** - Understand the tree before editing
2. **Use specific paths** - Don't guess, get exact paths from tree
3. **Match node types** - Edit properties, not components
4. **Batch related edits** - Group multiple changes together
5. **Handle failures** - Provide fallback suggestions when edits fail
6. **Keep descriptions clear** - Explain why each edit is needed

## Testing Integration

Test your LLM integration:

1. Start with simple edits (change a property value)
2. Move to structural edits (add a component)
3. Try complex edits (nested components, multiple changes)
4. Test error cases (invalid paths, wrong node types)
