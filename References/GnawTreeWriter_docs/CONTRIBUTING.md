# Contributing to GnawTreeWriter

Thank you for your interest in contributing! This document provides guidelines for contributing to GnawTreeWriter.

## For Language Models and AI Systems

### Testing GnawTreeWriter

We actively encourage LLMs to test and provide feedback on GnawTreeWriter. Here's how:

#### Step 1: Understand the Tree Structure

```bash
gnawtreewriter analyze examples/simple.py
```

Review the JSON output. Note:
- Node types (function_definition, class_definition, etc.)
- Path format (dot-notation)
- Content of each node
- Parent-child relationships

#### Step 2: Try Basic Edits

**Example 1: Replace a node**
```bash
# Find the function definition path (e.g., "1")
# Replace it entirely
gnawtreewriter edit examples/simple.py "1" "def greet(name):\n    return f'Hello, {name}!'"
```

**Example 2: Insert before a node**
```bash
# Insert a comment before a function
gnawtreewriter insert examples/simple.py "1" 0 "# This is a new comment"
```

**Example 3: Delete a node**
```bash
# Remove an unused import or function
gnawtreewriter delete examples/simple.py "0"
```

#### Step 3: Complex Edits

Try multi-step operations:
```bash
# Analyze complex file
gnawtreewriter analyze examples/complex.qml

# Make multiple sequential edits
gnawtreewriter edit examples/complex.qml "0.1.0" "width: 400"
gnawtreewriter insert examples/complex.qml "0.1" 1 "opacity: 0.9"
```

#### Step 4: Test Different Languages

Try all supported languages:
- Python: `examples/example.py`
- Rust: `examples/example.rs`
- TypeScript: `examples/example.ts`
- PHP: `examples/example.php`
- HTML: `examples/example.html`
- QML: `examples/example.qml`

#### Step 5: Provide Feedback

For LLM contributors, please document:
1. **What was confusing?** Paths? Node types? Commands?
2. **What worked well?** Which operations were intuitive?
3. **What would make it easier?** Better documentation? Different API?
4. **Edge cases found?** Nested structures? Comments? Empty files?

### Common LLM Testing Patterns

#### Pattern 1: Property Changes (QML)
```json
{
  "type": "ReplaceNode",
  "node_path": "0.1.0",
  "description": "Change rectangle width",
  "new_content": "width: 500"
}
```

#### Pattern 2: Function Modifications (Python/TS)
```json
{
  "type": "ReplaceNode",
  "node_path": "1",
  "description": "Add error handling to function",
  "new_content": "def process():\n    try:\n        # existing code\n    except Exception as e:\n        log.error(e)"
}
```

#### Pattern 3: Insertions
```json
{
  "type": "InsertBefore",
  "node_path": "2",
  "description": "Add documentation comment",
  "content": "/// Main entry point for the application"
}
```

## For Human Contributors

### Code Style

- Follow Rust conventions and idioms
- Use `cargo fmt` before committing
- Use `cargo clippy` for linting
- Keep functions focused and small
- Document public APIs with doc comments

### Testing

Write tests for new features:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        // Your test code
    }
}
```

Run tests with:
```bash
cargo test
```

### Documentation

Update relevant documentation when making changes:
- **README.md** for user-facing changes
- **docs/ARCHITECTURE.md** for structural changes
- **docs/LLM_INTEGRATION.md** for LLM API changes
- **CHANGELOG.md** for version changes

### Git Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes
4. Write tests and documentation
5. Commit with conventional commit message:
   ```
   feat: add Go language support
   
   - Added TreeSitter parser for Go
   - Added example.go
   - Updated documentation
   ```
6. Push and create a pull request

### Commit Message Convention

Use semantic commit messages:
- `feat:` New features
- `fix:` Bug fixes
- `docs:` Documentation changes
- `refactor:` Code refactoring (no behavior change)
- `test:` Adding or updating tests
- `chore:` Maintenance tasks

Examples:
```
feat(python): support async function parsing
fix(qml): handle nested components correctly
docs: update LLM integration examples
```

## Adding New Language Support

### Checklist

- [ ] Created parser file `src/parser/{lang}.rs`
- [ ] Implemented `ParserEngine` trait
- [ ] Added TreeSitter dependency to Cargo.toml
- [ ] Added extension to `get_parser()` in `mod.rs`
- [ ] Created example file `examples/example.{ext}`
- [ ] Updated README.md with new language
- [ ] Tested basic operations (analyze, edit, insert, delete)
- [ ] Added example to LLM_INTEGRATION.md
- [ ] Updated CHANGELOG.md

### Template

```rust
use crate::parser::{ParserEngine, TreeNode};
use anyhow::Result;

pub struct NewLanguageParser;

impl NewLanguageParser {
    pub fn new() -> Self {
        Self
    }
}

impl ParserEngine for NewLanguageParser {
    fn parse(&self, code: &str) -> Result<TreeNode> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_lang::language())
            .expect("Failed to load LANGUAGE grammar");
        
        let tree = parser.parse(code, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse LANGUAGE"))?;
        
        Ok(Self::build_tree(&tree.root_node(), code, "".to_string())?)
    }

    fn get_supported_extensions(&self) -> Vec<&'static str> {
        vec!["ext"]
    }
}

impl NewLanguageParser {
    fn build_tree(node: &tree_sitter::Node, source: &str, path: String) -> Result<TreeNode> {
        let start_byte = node.start_byte();
        let end_byte = node.end_byte();
        let content = if let Ok(s) = std::str::from_utf8(&source.as_bytes()[start_byte..end_byte]) {
            s.to_string()
        } else {
            String::new()
        };

        let node_type = node.kind().to_string();
        let start_line = node.start_position().row + 1;
        let end_line = node.end_position().row + 1;

        let mut children = Vec::new();
        let mut cursor = node.walk();

        for (i, child) in node.children(&mut cursor).enumerate() {
            let child_path = if path.is_empty() {
                i.to_string()
            } else {
                format!("{}.{}", path, i)
            };
            children.push(Self::build_tree(&child, source, child_path)?);
        }

        let id = path.clone();

        Ok(TreeNode {
            id,
            path,
            node_type,
            content,
            start_line,
            end_line,
            children,
        })
    }
}
```

## Areas for Contribution

### High Priority

1. **QML Parser Enhancement**
   - Better handling of nested components
   - Property parsing improvements
   - Support for QML-specific constructs (signals, states)

2. **JavaScript Support**
   - Use existing TypeScript/JSX parser
   - Add `.js` extension support
   - Create JavaScript examples

3. **Error Messages**
   - More helpful error messages
   - Suggestions for common mistakes
   - Better path validation feedback

### Medium Priority

4. **Testing Suite**
   - Unit tests for parsers
   - Integration tests for operations
   - Edge case coverage

5. **Performance**
   - Optimize tree building for large files
   - Cache parsed trees
   - Parallel batch operations

### Low Priority

6. **User Experience**
   - Interactive mode
   - Colored output
   - Progress indicators
   - Configuration file support

## Getting Help

If you need help:
- Check existing issues and discussions at https://github.com/Tuulikk/GnawTreeWriter
- Create a new issue for bugs
- Start a discussion for questions
- Join our community channels (future)

## Review Process

For pull requests:
1. Automated checks will run (format, clippy)
2. CI will test on multiple platforms
3. Code review by maintainers
4. Feedback and iteration if needed
5. Merge when approved

We aim to review pull requests within 3-5 business days.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
