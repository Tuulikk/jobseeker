# GnawTreeWriter Documentation

## Overview

GnawTreeWriter is a tree-based code editor designed for LLM-assisted editing. It works at the AST (Abstract Syntax Tree) level, allowing LLMs to make precise edits without worrying about syntax errors, mismatched brackets, or structural issues.

## How It Works

1. **Parse**: Source files are parsed into tree structures using TreeSitter.
2. **Validate**: Proposed edits are validated against the parser in memory before saving.
3. **Navigate**: Tree nodes are accessible via dot-notation paths (e.g., "1.2.0").
4. **Edit**: Operations target specific nodes, preserving structure and indentation.
5. **Write**: Changes are applied deterministically back to source files with automatic backups.

## Architecture

### Parser Engines

Each language has a dedicated parser implementing the `ParserEngine` trait:

```rust
pub trait ParserEngine {
    fn parse(&self, code: &str) -> Result<TreeNode>;
    fn get_supported_extensions(&self) -> Vec<&'static str>;
}
```

#### Supported Languages (TreeSitter based)
- **QML**: Robust TreeSitter-based parsing
- **Go**: Full AST parsing for Go source files
- **Python**: Full AST parsing
- **Rust**: Full AST parsing
- **TypeScript/TSX**: Support for modern web development
- **JavaScript/JSX**: Support via TypeScript parser
- **PHP**: Server-side script parsing
- **HTML**: Document structure parsing

### Tree Structure

```rust
pub struct TreeNode {
    pub id: String,
    pub path: String,           // Dot-notation path like "1.2.0"
    pub node_type: String,      // AST node type (e.g., "function_definition")
    pub content: String,        // Source code for this node
    pub start_line: usize,
    pub end_line: usize,
    pub children: Vec<TreeNode>,
}
```

### Core Operations

#### Edit
Replace the content of a specific node.

```bash
gnawtreewriter edit file.rs "0.2.1" "fn new() -> Self { ... }"
```

#### Insert
Add new content at a specific position relative to a node. Supports smart indentation.

```bash
# Position 0: before the node (or at the top of a container)
# Position 1: after the node (or at the bottom of a container)
# Position 2: after existing properties (QML specific)
gnawtreewriter insert file.qml "1.1" 2 "property int x: 10"
```

#### Delete
Remove a node and its content from the tree.

```bash
gnawtreewriter delete file.py "1.2"
```

## LLM Integration

### Approach

The tool is designed to work with LLMs in two main ways:

#### 1. Structure-Aware Editing
LLMs can focus on logic rather than syntax. The tool handles:
- Correct bracket placement
- Automatic indentation
- Syntax validation before saving
- Parent-child relationships

#### 2. Query-Response Pattern
LLMs can query the tree structure to understand code context, then request specific edits using precise paths.

## API Reference

### CLI Commands

#### analyze
Analyze file and show tree structure in JSON format.

```bash
gnawtreewriter analyze <file_path>
```

#### list
List all nodes with their paths and line numbers.

```bash
gnawtreewriter list <file_path> [--filter-type <type>]
```

#### add-property (QML)
Safely add a property to a QML component.

```bash
gnawtreewriter add-property <file> <target_path> <name> <type> <value>
```

#### add-component (QML)
Safely add a child component to a QML object.

```bash
gnawtreewriter add-component <file> <target_path> <name> [--content "props"]
```

## Safety Features

- **In-Memory Validation**: Every edit is re-parsed in memory. If the resulting code is syntactically invalid, the edit is aborted and no files are changed.
- **Automatic Backups**: Every successful edit creates a timestamped JSON backup of the original state in `.gnawtreewriter_backups/`.
- **Diff Preview**: Use the `--preview` flag to see a unified diff of the changes before applying them.

## Limitations

- The tool is currently optimized for block-level and statement-level edits. Very fine-grained expression edits may require more precise path targeting.
- TreeSitter grammars must be compiled into the binary.

## Future Enhancements

- Undo/redo functionality using the backup system.
- Support for more languages (C++, Java).
- VSCode extension for visual tree navigation.
- Move-node and Rename-node operations.