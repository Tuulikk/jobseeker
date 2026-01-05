# Changelog

All notable changes to GnawTreeWriter.

## [0.2.1] - 2025-12-27

### Added
- **Revolutionary Help System**: Interactive `examples` and `wizard` commands for guided learning
- **AI Agent Testing Framework**: Comprehensive test scenarios (AI_AGENT_TEST_SCENARIOS.md) with structured evaluation
- **Multi-File Time Restoration**: Complete project-wide time travel with `restore-project`, `restore-files`, `restore-session`
- **Transaction Logging**: Full audit trail of all operations with session management
- **Version Flag**: `--version` command to check current version
- **Lint Command**: `lint` command for basic file validation and issue detection
- **Directory Analysis**: `--recursive` flag for analyzing entire directories
- **Go Support**: Full TreeSitter-based parsing support for Go (`.go`)
- **Enhanced Preview**: `--preview` now shows a proper unified diff (using `similar` crate) instead of just the whole file
- **QML add-component**: New command to safely inject child components into QML objects
- **Core API**: Added `get_source()` to `GnawTreeWriter` for easier integration

### Changed
- **Documentation Overhaul**: Complete README and ROADMAP updates reflecting current capabilities
- **Error Handling**: Better error messages for directory analysis and invalid paths
- Improved CLI `preview` flags across all edit/insert/delete operations

### Fixed
- **Directory Analysis Bug**: Fixed "Is a directory" error with proper `--recursive` flag requirement
- **Documentation Inconsistencies**: Aligned all documentation with actual CLI behavior
- **Missing Commands**: Added previously documented but missing `lint` and `--version` commands

---

## [0.2.0] - 2025-12-26

### Added
- **TreeSitter QML Parser**: Replaced custom regex parser with a robust TreeSitter-based parser for QML.
- **Syntax Validation**: Automatic in-memory syntax validation before saving any edits. Prevents file corruption.
- **Smart Indentation**: `insert` command now automatically detects and applies parent/sibling indentation to new content.
- **Dedicated QML Add-Property**: New `add-property` command specifically optimized for QML AST structure.
- **Automatic FFI Linking**: Resolved version mismatch issues with `tree-sitter-qmljs` using dynamic language loading.

### Changed
- Improved `insert` logic to handle container braces correctly (e.g., inserting after `{`).
- Standardized node paths across all supported languages.
- Updated documentation with new command examples and technical details.

### Fixed
- Fixed a bug where nested braces in macros (like `serde_json::json!`) could cause code corruption during edits.
- Improved CLI stability and error reporting for missing nodes.

---

## [0.1.1] - 2025-12-26

### Added
- **Multi-file operations**: analyze, lint, find now support wildcards and directories
- **Automatic backup system**: timestamped JSON backups before every edit
- **Human-friendly lint mode**: `file:line:col severity message` format with JSON output
- **Fuzzy-edit command**: LLM-friendly editing with multi-strategy matching
- **QML add-property command**: Safe property injection for QML components
- **Diff preview**: `--preview` flag shows unified diff for all edit operations
- **List command**: Show all nodes with paths in a file
- **Smart selectors**: Find nodes by type and content

### Changed
- Updated README with comprehensive command documentation
- Added links to RECIPES.md and QML_EXAMPLES.md
- Added .gitignore for .gnawtreewriter_backups/ directory

### Fixed
- CLI stability issues (--help now works reliably)
- Multi-file analyze support (respects wildcards)
- Exit codes for lint errors

---

## [0.1.0] - 2025-12-26

### Initial Release

- Basic tree-based code editor for LLM-assisted editing
- Multi-language support: Python, Rust, TypeScript, PHP, HTML, QML
- Foundation for tree-level code manipulation