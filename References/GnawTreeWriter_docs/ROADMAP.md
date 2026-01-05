# GnawTreeWriter Roadmap

## Overview

GnawTreeWriter is a tree-based code editor optimized for LLM-assisted editing. This roadmap outlines the evolution from a precise CLI tool to an intelligent agent-integrated platform.

## Current Status: v0.2.1 (Released 2025-12-26)

### ✅ Completed Features

- **Multi-language support**: Python, Rust, TypeScript, PHP, HTML, QML, **Go**.
- **TreeSitter Foundation**: Robust parsing for all core languages.
- **Smart Indentation**: Automatic preservation of code style during insertions.
- **Syntax Validation**: In-memory re-parsing before saving changes.
- **QML Intents**: Dedicated commands for `add-property` and `add-component`.
- **Diff Preview**: Visual unified diff display using the `similar` library.
- **Automatic Backups**: Non-git safety net creating JSON snapshots before every edit.

---

## Phase 1: Reliability & Safety (The Non-Git Safety Net)
**Target: v0.3.0 - Q1 2026**

Focus on making the tool bulletproof and independent of Git for session-level recovery.

### **Core Safety & Recovery System**

- [x] **Transaction Log System**: 
  - JSON-based log file (`.gnawtreewriter_session.json`) tracking all operations
  - Human-readable format: `{"timestamp": "2025-01-02T15:30:45Z", "operation": "edit", "file": "app.py", "path": "0.1", "before_hash": "abc123", "after_hash": "def456", "description": "Updated function signature"}`
  - Session-scoped: cleared on explicit `gnawtreewriter session-start`, persists through crashes
  - Enables forensic analysis: "What happened to my code between 14:00 and 15:00?"
  - **STATUS**: ✅ COMPLETE - Integrated with all edit operations

- [x] **Multi-File Time Restoration System**:
  - `gnawtreewriter restore-project <timestamp> [--preview]` - Restore entire project to specific time
  - `gnawtreewriter restore-files --since <time> --files <patterns>` - Selective file restoration
  - `gnawtreewriter restore-session <session-id>` - Undo entire AI agent session
  - Project-wide time travel with atomic multi-file operations
  - Timestamp-based restoration with hash validation fallback
  - **STATUS**: ✅ COMPLETE - Working restoration engine implemented

- [x] **`undo` & `redo` Commands**:
  - `gnawtreewriter undo [--steps N]` - Reverse N operations (default 1)
  - `gnawtreewriter redo [--steps N]` - Re-apply N reversed operations  
  - `gnawtreewriter history [--format json/table]` - Show operation timeline
  - Navigate backup history without Git dependency
  - Atomic operation reversal: if undo fails, leave system in previous state
  - **STATUS**: Framework complete in `src/core/undo_redo.rs`, CLI commands added

- [x] **Enhanced Restore System**:
  - `gnawtreewriter restore <timestamp|operation-id> [--preview]`
  - Point-in-time recovery: "Restore app.py to state at 14:30"
  - Selective restoration: restore individual files or nodes
  - Preview system with comprehensive restoration planning
  - **STATUS**: ✅ COMPLETE - Full restoration engine with backup integration

- [x] **Interactive Help System**:
  - `gnawtreewriter examples [--topic <topic>]` - Practical workflow examples
  - `gnawtreewriter wizard [--task <task>]` - Interactive guidance system
  - Enhanced command help with detailed descriptions and use cases
  - Topic-specific help: editing, qml, restoration, workflow, troubleshooting
  - **STATUS**: ✅ COMPLETE - Revolutionary help system for AI agents and humans

- [x] **AI Agent Testing Framework**:
  - Comprehensive test scenarios document (AI_AGENT_TEST_SCENARIOS.md)
  - 8 detailed test scenarios from discovery to integration
  - Structured evaluation framework with rating system (1-5 scale)
  - Sample test files and complete environment setup
  - **STATUS**: ✅ COMPLETE - Ready for AI agent evaluation and feedback

- [ ] **Stable Node Addressing**:
  - Content-based node IDs: `node_abc123def` (hash of node content + position)
  - Graceful fallback to path-based addressing when content changes
  - Cross-edit stability: same logical node keeps same ID across minor edits
  - Migration tool: convert old path-based references to content-based IDs

---

## Phase 2: Multi-Project & Commercial Features  
**Target: v0.3.0 - Q1 2026**

Transform from single-project tool to multi-project development platform.

### **Multi-Project Architecture**

- [ ] **Project Manager System**:
  - Support for multiple concurrent projects per user
  - Community Edition: 5 concurrent projects with unlimited rotation
  - Team Starter: 15 concurrent projects ($19/month)
  - Professional: Unlimited projects ($49/month)
  - Project switching and archival without data loss

- [ ] **Configurable Limits System**:
  - Dynamic project limits based on user tier
  - Feature flag architecture for instant competitive response
  - A/B testing framework for optimal limit discovery
  - Graceful upgrade prompts and user experience

- [ ] **Basic Team Coordination**:
  - Shared session visibility across team members
  - Cross-developer project state synchronization
  - Team-scoped transaction history and restoration
  - Basic conflict resolution for concurrent edits

- [ ] **MCP Server Implementation**: 
  - Native Model Context Protocol support as built-in tool
  - Tool definitions for all major operations (edit, analyze, find, restore)
  - Context-aware responses optimized for LLM processing
  - Batch operation support: multiple edits in single MCP call

---

## Phase 3: AI Agent Integration & Intelligence
**Target: v0.4.0 - Q2 2026**

Transform into AI-native development platform.

### **Advanced AI Agent Features**

- [ ] **Smart Semantic Targeting**:
  - `--function "main"` instead of raw paths
  - `--class "UserController" --method "create"`  
  - `--property "width" --within "Rectangle"`
  - Natural language queries: `--find "the button that handles login"`
  - Fuzzy matching with confidence scores

- [ ] **LLM-Optimized Output**:
  - Token-compressed JSON formats for large ASTs
  - Hierarchical detail levels: summary → detailed → full AST
  - Context window management: smart truncation preserving important nodes
  - Streaming responses for large operations

- [ ] **Intent Extrapolation Engine**:
  - High-level commands: `gnawtreewriter refactor-extract-function app.py "calculate_total" --lines 45-60`
  - Pattern-based transformations: `gnawtreewriter apply-pattern observer app.py --class "DataModel"`
  - Architecture enforcement: `gnawtreewriter ensure-pattern repository database.py`

- [ ] **Cross-Project Intelligence**:
  - Dependency tracking across multiple projects
  - API contract monitoring and change impact analysis
  - Automated consistency checking across project boundaries
  - Smart suggestions based on cross-project patterns

---

## Phase 4: Autonomous Code Guardian & Enterprise Platform
**Target: v0.5.0 - Q3 2026**

Evolve into always-on enterprise development infrastructure.

### **Continuous Monitoring System**

- [ ] **File System Watcher & Daemon**:
  - `gnawtreewriter daemon start` - Background process monitoring all projects
  - Real-time AST updates when files change (even from external tools)
  - Change event streaming to connected AI agents
  - Conflict detection: "File changed outside GnawTreeWriter, merging changes"

- [ ] **Enterprise Policy Engine**:
  - Company-specific coding standards enforcement
  - Architecture decision record (ADR) compliance checking  
  - Automated security vulnerability patching
  - Custom policy DSL for organizational rules

- [ ] **Multi-Tenant Cloud Service**:
  - SaaS version with per-organization isolation
  - Enterprise SSO integration (SAML, OIDC)
  - Comprehensive audit logging for compliance (SOX, GDPR, HIPAA)
  - Global policy enforcement across teams and projects
  - Advanced analytics and reporting dashboard

- [ ] **Intelligent Structural Analysis**:
  - Architectural lint rules: "Controllers should not directly access database"
  - Security scanning: "No hardcoded secrets detected"
  - Performance monitoring: "Large function detected, suggest refactoring"
  - Cross-project dependency impact analysis

## Phase 5: Universal Tree Platform (2027+)
**Target: v0.6.0+ - Future Expansion**

Expand beyond code to all hierarchical systems.

### **Multi-Domain Tree Support**

- [ ] **Infrastructure as Code**:
  - Terraform/CloudFormation AST parsing and editing
  - `gnawtreewriter scale-service infrastructure.tf "web_servers" --count 5`
  - Cloud resource dependency visualization
  - Cost impact analysis for infrastructure changes

- [ ] **Configuration Management**:
  - Docker Compose, Kubernetes YAML, CI/CD pipelines
  - `gnawtreewriter add-service docker-compose.yml "redis" --image "redis:7"`
  - Environment-specific configuration templating
  - Secret management integration

- [ ] **AI-Native Development Ecosystem**:
  - Autonomous refactoring agents continuously improving architecture
  - Cross-language translation preserving tree structure
  - Predictive code evolution and architectural suggestions
  - Natural language programming: "Create REST API" → Full implementation

### **Integration Ecosystem**

- [ ] **LSP Server**: Universal structured editing for all IDEs
- [ ] **GitHub App**: Automated PR reviews and suggestions
- [ ] **IDE Extensions**: Native plugins for VS Code, IntelliJ, Neovim  
- [ ] **API Gateway**: RESTful API for third-party tool integration

---

## Implementation Priorities

### **Phase 1 Quick Wins** (Next 3 months)
1. Transaction logging system (foundation for all future features)
2. Undo/redo commands (immediate developer value)
3. Enhanced restore with preview (safety & confidence)
4. Content-based node IDs (stability for AI agents)

### **Phase 2 AI Integration** (Months 4-9)  
1. MCP server implementation (unlock AI ecosystem)
2. Semantic targeting (make AI agents more effective)
3. LLM-optimized output formats (performance & usability)
4. Intent extrapolation engine (high-level automation)

### **Multi-Agent Development Strategy**

Based on AI agent strengths observed:

- **Gemini 3**: Architecture decisions, documentation, long-term planning
- **Claude**: Implementation details, error handling, user experience  
- **GLM-4.7**: Fast iteration on specific features (with careful monitoring)
- **Raptor Mini**: User experience feedback, edge case identification

### **Success Metrics**

- **Phase 1**: Zero data loss incidents, <5 second recovery time
- **Phase 2**: 80% of AI editing tasks use GnawTreeWriter instead of raw text
- **Phase 3**: 90% reduction in structural code errors across team
- **Phase 4**: Enterprise adoption with measurable productivity gains

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Documentation

- [ARCHITECTURE.md](docs/ARCHITECTURE.md) - Technical design  
- [FUTURE_CONCEPTS.md](docs/FUTURE_CONCEPTS.md) - Deep dive into planned features
- [LLM_INTEGRATION.md](docs/LLM_INTEGRATION.md) - Guide for AI agents
- [MULTI_AGENT_DEVELOPMENT.md](docs/MULTI_AGENT_DEVELOPMENT.md) - Collaboration strategies ✓

## Recent Progress (2025-12-27)

### ✅ **Phase 1 HISTORIC MILESTONE - COMPLETE!**
- **Revolutionary multi-file time restoration system** implemented and working
- **Transaction logging fully integrated** with all edit operations  
- **Complete restoration engine** with backup file integration working
- **Project-wide time travel** verified: `restore-project`, `restore-files`, `restore-session`
- **Timestamp-based restoration** with hash validation fallback
- **Multi-agent development documentation** complete
- **Revolutionary help system** with examples, wizards, and interactive guidance
- **AI agent test framework** with comprehensive evaluation scenarios
- **Professional documentation** ready for community and enterprise adoption

### 🎉 **VERIFIED WORKING CAPABILITIES:**
- ✅ Project restoration: Successfully restored files to specific timestamps
- ✅ Multi-file coordination: Atomic restoration operations across multiple files
- ✅ Session-based restoration: Undo entire AI agent workflow sessions
- ✅ Backup file integration: Robust backup parsing and content restoration
- ✅ Error handling: Graceful fallbacks and detailed error reporting
- ✅ Preview system: Safe restoration planning before execution
- ✅ Help system: Interactive wizards, examples, and comprehensive command help
- ✅ AI testing framework: 8 detailed scenarios with structured evaluation
- ✅ GitHub publication: Complete repository ready for community adoption

### 🔄 **Phase 1 Enhancement Tasks**
- [ ] Add content-based node ID system for enhanced stability
- [ ] Polish hash-matching algorithm for optimal restoration performance
- [ ] Collect and incorporate AI agent feedback from test scenarios
- [ ] Add restoration statistics and analytics dashboard