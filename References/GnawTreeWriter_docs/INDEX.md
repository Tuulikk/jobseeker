# GnawTreeWriter Documentation Index

**The Complete Handbook for AI-Native Temporal Code Editing**

Version: 0.2.1| Last Updated: 2025-12-27| Status: Phase 1 Complete

---

## 🚀 **Quick Start**

- **[README.md](README.md)** - Project overview, installation, and basic usage
- **[AGENTS.md](AGENTS.md)** - Quick reference for AI agents (Claude, GPT, Gemini, etc.)
- **[ROADMAP.md](ROADMAP.md)** - Development phases, completed milestones, and future vision

---

## 📚 **Core Documentation**

### **User Guides**
- **[LLM_INTEGRATION.md](LLM_INTEGRATION.md)** - Complete guide for AI agent integration
- **[MULTI_AGENT_DEVELOPMENT.md](MULTI_AGENT_DEVELOPMENT.md)** - Strategies for human-AI collaborative development
- **[RECIPES.md](RECIPES.md)** - Common tasks and workflow patterns
- **[QML_EXAMPLES.md](QML_EXAMPLES.md)** - Step-by-step QML editing examples

### **Technical References**
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System design and technical architecture
- **[TESTING.md](TESTING.md)** - Testing strategies and examples
- **[DEVELOPER_REPORT.md](DEVELOPER_REPORT.md)** - Feedback and improvement roadmap

### **Advanced Topics**
- **[FUTURE_CONCEPTS.md](FUTURE_CONCEPTS.md)** - Deep dive into planned features and concepts

### **Troubleshooting & Testing**
- **[AI_AGENT_TEST_SCENARIOS.md](AI_AGENT_TEST_SCENARIOS.md)** - Comprehensive testing scenarios for AI agents
- **[KNOWN_ISSUES.md](KNOWN_ISSUES.md)** - Known limitations, workarounds, and troubleshooting guide

### **Release Information**
- **[CHANGELOG.md](CHANGELOG.md)** - Version history and release notes

---

## 🎯 **Use Case Guides**

### **For AI Agents**
```
Start Here: AGENTS.md → AI_AGENT_TEST_SCENARIOS.md → LLM_INTEGRATION.md → RECIPES.md
```
Essential for understanding GnawTreeWriter commands and comprehensive testing.

### **For Developers**
```
Start Here: README.md → ARCHITECTURE.md → MULTI_AGENT_DEVELOPMENT.md
```
Perfect for understanding the system and contributing to development.

### **For Teams & Enterprise**
```
Start Here: README.md → ROADMAP.md → MULTI_AGENT_DEVELOPMENT.md
```
Understand business model, team features, and enterprise capabilities.

### **For QML Development**
```
Start Here: AGENTS.md → QML_EXAMPLES.md → RECIPES.md → KNOWN_ISSUES.md
```
Specialized workflow for QML component development with troubleshooting.

### **For Troubleshooting**
```
Start Here: KNOWN_ISSUES.md → AI_AGENT_TEST_SCENARIOS.md → AGENTS.md
```
When things aren't working as expected or you need comprehensive testing.

---

## ⭐ **Key Capabilities Overview**

### **✅ Phase 1 Complete - Temporal Project Management**
- **Project-wide time restoration**: `gnawtreewriter restore-project <timestamp>`
- **Session-based rollback**: `gnawtreewriter restore-session <session-id>`
- **Multi-file coordination**: Atomic restoration across multiple files
- **Transaction logging**: Complete audit trail of all changes
- **AI agent integration**: MCP-ready architecture

### **🔄 Phase 2 In Progress - Multi-Project & Commercial**
- **Multi-project support**: 5 concurrent projects (Community), 15 (Team Starter), unlimited (Professional)
- **Team coordination**: Shared sessions and cross-developer synchronization
- **Configurable limits**: Dynamic response to competitive pressure
- **Commercial features**: Team plans starting at $19/month

### **🎯 Future Phases - Enterprise & Universal Platform**
- **Enterprise policy engine**: Custom compliance and governance rules
- **Cloud-native architecture**: Multi-tenant SaaS platform
- **Universal tree editing**: Infrastructure, configuration, and data structure support

---

## 🛠️ **Quick Reference Commands**

### **Analysis & Navigation**
```bash
gnawtreewriter analyze <file>          # Parse and show AST structure
gnawtreewriter list <file>             # Show all nodes with paths
gnawtreewriter find <path> --type TYPE # Search for specific node types
```

### **Editing Operations**
```bash
gnawtreewriter edit <file> <path> <content>        # Edit specific node
gnawtreewriter insert <file> <path> <pos> <content> # Insert content
gnawtreewriter fuzzy-edit <file> <query> <content>  # Fuzzy search and edit
```

### **Time Travel & Restoration**
```bash
gnawtreewriter restore-project <timestamp>          # Restore entire project
gnawtreewriter restore-files --since <time> --files <pattern> # Selective restoration
gnawtreewriter restore-session <session-id>         # Undo AI agent session
gnawtreewriter history [--limit N]                  # Show transaction history
```

### **Session Management**
```bash
gnawtreewriter session-start           # Start new session
gnawtreewriter status                  # Show current state
gnawtreewriter undo [--steps N]        # Undo operations
gnawtreewriter redo [--steps N]        # Redo operations
```

---

## 🔗 **External Resources**

- **GitHub Repository**: https://github.com/Tuulikk/GnawTreeWriter
- **Issues & Support**: https://github.com/Tuulikk/GnawTreeWriter/issues
- **Discussions**: https://github.com/Tuulikk/GnawTreeWriter/discussions

---

## 🏆 **Why GnawTreeWriter?**

### **Problems Solved**
- **AI bracket matching issues**: Work at AST level, not raw text
- **Multi-file coordination**: Atomic operations across project
- **Temporal debugging**: "What did the AI change 2 hours ago?"
- **Session management**: Group and control AI agent workflows
- **Structural validation**: Syntax checking before changes

### **Unique Advantages**
- **World's first temporal project management system**
- **Built for AI-native development workflows**
- **Multi-agent collaborative development proven**
- **Open source foundation with commercial scaling**
- **Tree-based editing for all hierarchical structures**

---

## 📈 **Success Stories**

> *"GnawTreeWriter was built using multi-agent development - proving that human-AI collaborative development is not just possible, but superior to traditional approaches."*

> *"The ability to restore entire projects to specific timestamps has revolutionized how we work with AI agents. We can let them experiment freely, knowing we can always roll back safely."*

> *"Session-based restoration means we can undo an entire AI agent workflow with one command. This changes everything about AI-assisted development."*

---

## 🤝 **Contributing**

See **[CONTRIBUTING.md](../CONTRIBUTING.md)** in the main repository for guidelines on:
- Code contributions
- Documentation improvements
- Bug reports and feature requests
- Multi-agent development workflows

---

## 📄 **License**

GnawTreeWriter is released under the MIT License. See **[LICENSE](../LICENSE)** for details.

Community Edition features are free forever. Commercial features require paid licenses for teams and enterprises.

---

*This documentation represents the collaborative effort of human developers and AI agents (Claude, Gemini, GLM-4.7, Raptor Mini) working together to create the future of software development.*

**Last updated**: 2025-12-27| **Document version**: 1.0 | **Software version**: 0.2.1