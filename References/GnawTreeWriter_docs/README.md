# GnawTreeWriter

Tree-based code editor for LLM-assisted editing. Edit code files based on tree structure levels, avoiding bracket issues and structural problems from LLM code generation.

## Features

- **Multi-language support**: Python, Rust, TypeScript/TSX, PHP, HTML, QML
- **Tree-based editing**: Work at AST level, not raw text
- **Precise edits**: Target specific nodes with dot-notation paths
- **LLM-optimized**: Structured edit requests and detailed context
- **Batch operations**: Apply multiple edits simultaneously
- **Comprehensive parsing**: Full AST tree structure for all languages
- **Automatic backups**: Timestamped JSON backups before every edit
- **Safe editing**: Preview changes with `--preview` flag
- **Multi-file operations**: Analyze and lint multiple files at once
- **Smart search**: Find nodes by type and content
- **Revolutionary time travel**: Project-wide restoration to any timestamp
- **Session management**: Track and undo entire AI agent workflows
- **Interactive help system**: Examples, wizards, and comprehensive guidance
- **AI-native design**: Built specifically for AI-assisted development

## Why Use GnawTreeWriter?

### Problems with Traditional LLM Code Editing
- LLMs often struggle with matching brackets
- Indentation errors are common
- Structural changes can break code
- Hard to make precise, targeted edits

### How GnawTreeWriter Solves This
- **No bracket management**: AST handles structure automatically
- **No indentation worries**: Formatting is preserved with smart indentation
- **Syntax Validation**: proposed edits are checked against the parser before saving
- **Precise targeting**: Edit specific nodes at specific paths
- **Deterministic results**: Same input always produces same output
- **Context-aware**: LLM can understand surrounding code structure

---

## Installation

### From Source

```bash
git clone https://github.com/Tuulikk/GnawTreeWriter.git
cd GnawTreeWriter
cargo build --release
```

The binary will be at `target/release/gnawtreewriter`.

### Using cargo install (Recommended)

```bash
cargo install --git https://github.com/Tuulikk/GnawTreeWriter.git
```

### From Binary Release (Future)

Once releases are published:
```bash
# Download binary for your platform
chmod +x gnawtreewriter
sudo mv gnawtreewriter /usr/local/bin/
```

## Quick Start

### First Time? Get Interactive Help!

```bash
# Get comprehensive help and examples
gnawtreewriter --help
gnawtreewriter examples
gnawtreewriter wizard --task first-time
```

### Basic Usage

```bash
# Analyze a file to understand structure
gnawtreewriter analyze app.py

# List all available nodes with paths
gnawtreewriter list app.py

# Edit a specific node (with preview first)
gnawtreewriter edit app.py "0.1" 'def hello(): print("world")' --preview
gnawtreewriter edit app.py "0.1" 'def hello(): print("world")'

# Insert new content
gnawtreewriter insert app.py "0" 1 'def new_function(): pass'

# Start session tracking for AI workflows
gnawtreewriter session-start
```

### Time Travel Features

```bash
# Restore entire project to specific timestamp
gnawtreewriter restore-project "2025-12-27T15:30:00Z" --preview

# Undo an entire AI agent session
gnawtreewriter restore-session "session_id"

# View what happened and when
gnawtreewriter history
```

### For AI Agents

See [AI_AGENT_TEST_SCENARIOS.md](AI_AGENT_TEST_SCENARIOS.md) for comprehensive testing guide and [AGENTS.md](AGENTS.md) for quick reference.

## Supported Languages

| Language | Extension | Parser | Status |
|-----------|-----------|---------|---------|
| QML | `.qml` | TreeSitter | ✅ Stable |
| Go | `.go` | TreeSitter | ✅ Stable |
| Python | `.py` | TreeSitter | ✅ Stable |
| Rust | `.rs` | TreeSitter | ✅ Stable |
| TypeScript | `.ts`, `.tsx` | TreeSitter | ✅ Stable |
| JavaScript | `.js`, `.jsx` | TreeSitter | ✅ Stable |
| PHP | `.php` | TreeSitter | ✅ Stable |
| HTML | `.html`, `.htm` | TreeSitter | ✅ Stable |

## CLI Commands

### analyze
Analyze file and show tree structure in JSON format. Supports wildcards and directories.

```bash
# Analyze single file
gnawtreewriter analyze app.py

# Analyze multiple files (supports wildcards)
gnawtreewriter analyze *.qml

# Analyze directory recursively
gnawtreewriter analyze src/ --recursive

# Get summary format
gnawtreewriter analyze . --recursive --format summary
```

### add-property
QML-specific command to safely add a property to a component at the correct position.

```bash
gnawtreewriter add-property <file_path> <target_path> <name> <type> <value>

# Example: Add property to Rectangle
gnawtreewriter add-property app.qml "0.1" myProp string "'hello'"
```

### add-component
QML-specific command to safely add a child component.

```bash
gnawtreewriter add-component <file_path> <target_path> <component_name> [--content "props"]

# Example: Add a Button inside a Rectangle
gnawtreewriter add-component app.qml "0.1" Button --content "text: 'Click me'"
```



### list
List all nodes with their paths in a file.

```bash
# List all nodes
gnawtreewriter list <file_path>

# Filter by node type
gnawtreewriter list <file_path> --filter-type Property
```

### find
Find nodes matching criteria across files.

```bash
# Find by node type
gnawtreewriter find <file_path> --node-type Property

# Find by content
gnawtreewriter find <file_path> --content "mainToolbar"

# Find in directory
gnawtreewriter find app/ui/qml/ --content "width:"
```

### lint
Lint files and show issues with severity levels.

```bash
# Lint single file
gnawtreewriter lint app.py

# Lint directory recursively
gnawtreewriter lint src/ --recursive

# Get JSON output for CI
gnawtreewriter lint . --recursive --format json
```

### Time Travel & Restoration Commands

#### restore-project
Restore entire project to a specific point in time.

```bash
# Preview what would be restored
gnawtreewriter restore-project "2025-12-27T15:30:00Z" --preview

# Restore all files to timestamp
gnawtreewriter restore-project "2025-12-27T15:30:00Z"
```

#### restore-session
Undo all changes from a specific AI agent session.

```bash
# Find session ID from history
gnawtreewriter history

# Restore entire session
gnawtreewriter restore-session "session_1766859069329812591" --preview
```

#### restore-files
Selectively restore files modified since a timestamp.

```bash
# Restore Python files modified since timestamp
gnawtreewriter restore-files --since "2025-12-27T16:00:00Z" --files "*.py"
```

### Session Management

#### session-start
Start a new session to group related operations.

```bash
gnawtreewriter session-start
```

#### history
Show transaction history with timestamps.

```bash
# Show recent operations
gnawtreewriter history

# Show more with JSON format
gnawtreewriter history --limit 20 --format json
```

#### undo / redo
Session-based undo and redo operations.

```bash
# Undo last operation
gnawtreewriter undo

# Undo multiple steps
gnawtreewriter undo --steps 3

# Redo operations
gnawtreewriter redo --steps 2
```

### Help & Learning

#### examples
Show practical examples for common tasks.

```bash
# General examples
gnawtreewriter examples

# Topic-specific examples
gnawtreewriter examples --topic editing
gnawtreewriter examples --topic restoration
gnawtreewriter examples --topic qml
```

#### wizard
Interactive help for guided workflows.

```bash
# Start interactive wizard
gnawtreewriter wizard

# Task-specific guidance
gnawtreewriter wizard --task first-time
gnawtreewriter wizard --task editing
gnawtreewriter wizard --task troubleshooting
```

### Version and Help Commands

#### --version
Check your current GnawTreeWriter version.

```bash
gnawtreewriter --version
```

#### --help
Get comprehensive help for any command.

```bash
# General help
gnawtreewriter --help

# Command-specific help
gnawtreewriter edit --help
gnawtreewriter restore-project --help
```

### show
Show content of a specific node.

```bash
gnawtreewriter show <file_path> <node_path>
```

**node_path**: Dot-notation path (e.g., "0.2.1")

### edit
Edit a node's content.

```bash
# Edit node directly
gnawtreewriter edit <file_path> <node_path> <new_content>

# Preview changes without applying
gnawtreewriter edit <file_path> <node_path> <new_content> --preview
```

**Backup**: Every edit automatically creates a timestamped JSON backup in `.gnawtreewriter_backups/`.

**Output**: Success message (or error if node not found).

Replaces entire content of node at `node_path` with `new_content`.
### insert
Insert new content relative to a node.

```bash
gnawtreewriter insert <file_path> <parent_path> <position> <content>
```

**position** values:
- `0`: Insert before the node at `parent_path`
- `1`: Insert after the node at `parent_path`
- `2`: Insert as a child of the node (where applicable)

### delete
Delete a node from the tree.

```bash
gnawtreewriter delete <file_path> <node_path>
```

Removes the node and all its children from the tree.

## Tree Paths

Nodes are addressed using dot-notation:
- `root` - Document root
- `0` - First child of root
- `0.1` - Second child of first root child
- `0.2.1` - Second child of third child of first root child

Example tree:
```
root
├── 0 (Import)
├── 1 (Function)
│   ├── 1.0 (function keyword)
│   ├── 1.1 (function name)
│   └── 1.2 (function body)
│       ├── 1.2.0 (statement 1)
│       └── 1.2.1 (statement 2)
└── 2 (Class)
```

## Architecture

See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed technical documentation.

### Additional Documentation

- [Recipes](docs/RECIPES.md) - Common tasks and workflows
- [QML Examples](docs/QML_EXAMPLES.md) - Step-by-step QML editing examples
- [LLM Integration](docs/LLM_INTEGRATION.md) - Guide for language model integration
- [Testing](docs/TESTING.md) - Testing strategies and examples
- [Developer Report](docs/DEVELOPER_REPORT.md) - Feedback and improvement roadmap

## Examples

### Python: Add a function to a module
```bash
# 1. Analyze to find the module path
gnawtreewriter analyze module.py

# 2. Insert new function
gnawtreewriter insert module.py "0" 1 "def new_function(x, y):\n    return x + y"
```

### QML: Change a property value
```bash
# 1. Find the property node path
gnawtreewriter analyze app.qml

# 2. Edit the property
gnawtreewriter edit app.qml "0.1.0" "width: 300"
```

### TypeScript: Add a method to a class
```bash
# 1. Analyze the file
gnawtreewriter analyze app.ts

# 2. Find the class block path
gnawtreewriter show app.ts "1.3"

# 3. Insert the new method
gnawtreewriter insert app.ts "1.3" 2 "newMethod(): void { console.log('hello'); }"
```

## AI Agent Integration

GnawTreeWriter is designed from the ground up for AI-native development workflows.

### Revolutionary Capabilities for AI Agents
- **Temporal Project Management**: Restore entire projects to specific timestamps
- **Session-based Workflows**: Track and undo complete AI agent sessions
- **Safe Experimentation**: Preview all changes before applying
- **Tree-based Editing**: No more bracket-matching or indentation errors
- **Comprehensive Help System**: Interactive learning and troubleshooting

### Perfect for AI Development Workflows
```bash
# Start AI session
gnawtreewriter session-start

# Make multiple changes safely
gnawtreewriter edit file.py "0.1" 'new code' --preview
gnawtreewriter add-component ui.qml "0" Button

# If something goes wrong, undo entire session
gnawtreewriter restore-session "session_id"
```

### Built-in Testing Framework
See [AI_AGENT_TEST_SCENARIOS.md](AI_AGENT_TEST_SCENARIOS.md) for comprehensive testing scenarios designed specifically for AI agents.

### Multi-Agent Development Proven
This tool was built using multi-agent collaboration (Claude, Gemini, GLM-4.7, Raptor Mini), proving that human-AI collaborative development is not just possible, but superior.

For complete integration guide, see [docs/LLM_INTEGRATION.md](docs/LLM_INTEGRATION.md).

## Development

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Development Workflow

1. Make changes to parser or core logic
2. Test with example files in `examples/` directory
3. Run `cargo check` for compilation errors
4. Commit with descriptive message following conventional commits
5. Update CHANGELOG.md with user-facing changes

### Adding New Languages

1. Create new parser file in `src/parser/{language}.rs`
2. Implement `ParserEngine` trait
3. Add to `src/parser/mod.rs` in `get_parser()`
4. Update Cargo.toml with TreeSitter dependency
5. Add example file in `examples/`
6. Update README and documentation

## Contributing

We welcome contributions! Areas of interest:

- **More languages**: Add parsers for JavaScript, Go, Java, C++, etc.
- **Better QML parsing**: Improve nested component handling
- **Diff preview**: Show what will change before applying edits
- **Undo/redo**: Track and revert changes
- **LSP integration**: Provide language server protocol support
- **VSCode extension**: Create editor plugin
- **Testing**: Add test suite with edge cases

### For Language Models

If you're testing GnawTreeWriter with an LLM:

1. Start with the example files in `examples/`
2. Try basic edits (property changes, simple insertions)
3. Move to complex edits (nested structures, multiple changes)
4. Report issues or confusing behavior
5. Suggest improvements to the edit intents or API

Your feedback is crucial for making this tool truly LLM-friendly!

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## Roadmap

### v0.2.0
- [ ] JavaScript parser (using existing TypeScript parser)
- [ ] Go language support
- [ ] Improved QML parser with better nesting
- [ ] Diff generation and preview

### v0.3.0
- [ ] Batch undo/redo
- [ ] Context-aware suggestions
- [ ] VSCode extension
- [ ] Python API/SDK

### Future
- [ ] More languages (Java, C++, C#, etc.)
- [ ] LSP server
- [ ] Web interface
- [ ] AI-powered refactoring suggestions

## Known Limitations

- **Directory analysis**: Requires `--recursive` flag for directory arguments (`analyze dir/` fails, use `analyze dir/ --recursive`)
- **QML instantiation**: Parse success doesn't guarantee runtime QML instantiation success  
- **Large projects**: Very large projects may require patience for full analysis
- **Hash matching**: Occasional backup hash mismatches resolved with timestamp fallback

## Version History & Feature Availability

- **v0.2.1**: Complete time restoration system, interactive help system (`examples`, `wizard`), AI testing framework, `lint` command, `--version` flag
- **v0.2.0**: Multi-file support, transaction logging, session management (`restore-project`, `restore-session`)  
- **v0.1.x**: Basic tree editing and QML support (`analyze`, `edit`, `add-property`)

**Current version check**: `gnawtreewriter --version`  
**Feature verification**: All examples in this README are tested with v0.2.1+

## License

MIT License - see LICENSE file for details

## Getting Help

- **Interactive Help**: `gnawtreewriter wizard` for guided assistance
- **Examples**: `gnawtreewriter examples` for practical workflows  
- **Command Help**: `gnawtreewriter <command> --help` for detailed usage
- **AI Agent Testing**: See [AI_AGENT_TEST_SCENARIOS.md](AI_AGENT_TEST_SCENARIOS.md)
- **Issues**: Report bugs on GitHub Issues
- **Discussions**: Use GitHub Discussions for questions
- **Documentation**: Complete handbook in [GnawTreeWriter_docs/](GnawTreeWriter_docs/)

## Acknowledgments

- TreeSitter for excellent parser grammar framework
- Rust community for the amazing tooling
- All contributors and testers
