# GnawTreeWriter: Future Concepts & Idea Lab

This document explores experimental ideas and deep-dive concepts for future development.

## 1. The "Structural Safety Net" (Beyond Git)

Current AI agents often destroy code structure by misplacing braces. While Git is a safety net, it's too coarse-grained for rapid iteration.

### Concept: Tree Transactions
Every edit should be part of a structural transaction.
*   **Logical Undo**: Revert to the previous AST state, not just text.
*   **Snapshot Diffing**: Compare the current tree with a snapshot from 5 minutes ago to see which components were added/removed.
*   **Session Persistence**: Maintain a local database of tree-states during a coding session.

## 2. Agent Intelligence & "Much with Little"

To minimize the "textual burden" on LLMs, we want to allow complex actions with minimal input.

### Concept: Intent Extrapolation
Instead of telling the LLM to "Insert 5 lines at root.1.2", the LLM says `add-boilerplate --type QtObject`.
*   **Smart Defaults**: The tool knows that a `Button` in QML usually needs `width`, `height`, and `text`.
*   **Contextual Expansion**: If an agent says `add-import "fmt"`, the tool finds the `import` section (regardless of its path) and places it correctly according to language standards.

### Concept: Analysis Toggles (Auto-Analysis)
*   **Watch-Mode (Sentinel)**: A background process that monitors files. If a file is changed externally (by a human or another AI), the tool immediately attempts to parse it. 
    *   If the new state is valid, it creates a "Last Known Good" snapshot.
    *   If the new state is invalid, it triggers a **Structural Alarm**, pinpointing exactly where the hierarchy broke.
*   **AST Auto-Healing**: When a break is detected, instead of a full rollback, the tool analyzes the "error nodes" provided by TreeSitter. It can then propose minimal structural fixes (e.g., "Insert missing '}' at line 45") to restore integrity while preserving the user's/agent's new logical code.
*   **Pulse mode**: Periodically pings the Agent with a summary of the current AST: "You are currently working inside `Rectangle { id: main }`, which has 3 children."

## 3. Visualization: "Seeing the Tree"

Paths like `0.2.1.4` are hard for humans to keep in their head.

### Concept: TUI AST Explorer
A terminal-based interface where you can:
*   Navigate the tree with arrow keys.
*   Highlight a node to see its path and content.
*   Press a key to copy the node path for use in a command.

### Concept: Web-based Live View
A simple local web server that renders the AST as a dynamic graph. As the Agent edits the code, the graph updates in real-time.

## 4. MCP: The Bridge to Agent Autonomy

Implementing the **Model Context Protocol (MCP)** would allow GnawTreeWriter to:
*   Expose its commands as "Tools" that Claude/GPT can call natively.
*   Provide "Resources" (like the current AST) that the model can subscribe to.
*   Handle the communication overhead, leaving the LLM to focus on logic.

## 5. Standardizing the "AI Writing Layer"

The ultimate goal is to make GnawTreeWriter the standard "Driver" for AI code writing.
*   **Write-Only Lock**: A mode for agents where raw file writing is disabled, forcing them to use the safe, validated structural API.
*   **Grammar-Guided Generation**: Use the AST knowledge to constrain the LLM output to only valid syntax for the current language.
