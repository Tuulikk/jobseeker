# AI Agent Test Scenarios for GnawTreeWriter

**Comprehensive Testing Guide for AI Agents using GnawTreeWriter**

Version: 1.0 | Created: 2025-12-27 | Target: AI Agents (Claude, Gemini, GPT, etc.)

---

## 🎯 **Purpose**

This document provides structured test scenarios for AI agents to evaluate GnawTreeWriter's functionality, usability, and integration potential. Each scenario includes expected outcomes, success criteria, and feedback questions.

---

## 🧪 **Test Environment Setup**

### **Prerequisites**
```bash
# Install GnawTreeWriter
cargo install --git https://github.com/Tuulikk/GnawTreeWriter.git

# Verify installation
gnawtreewriter --help

# Create test workspace
mkdir gnawtreewriter_test
cd gnawtreewriter_test
```

### **Sample Test Files**
Create these files for testing:

**test_python.py**:
```python
def hello_world():
    print("Hello, World!")

class Calculator:
    def add(self, a, b):
        return a + b
    
    def multiply(self, a, b):
        return a * b
```

**test_component.qml**:
```qml
import QtQuick

Rectangle {
    width: 100
    height: 100
    color: "blue"
    
    Text {
        text: "Hello QML"
        anchors.centerIn: parent
    }
}
```

---

## 📋 **Scenario 1: Discovery & Learning**

### **Objective**: Understand tool capabilities and help system quality

### **Test Steps**:
1. **Help System Exploration**:
   ```bash
   gnawtreewriter --help
   gnawtreewriter examples
   gnawtreewriter examples --topic editing
   gnawtreewriter wizard
   gnawtreewriter wizard --task first-time
   ```

2. **Command-Specific Help**:
   ```bash
   gnawtreewriter edit --help
   gnawtreewriter restore-project --help
   gnawtreewriter analyze --help
   ```

### **Success Criteria**:
- [ ] Help text is clear and actionable
- [ ] Examples are practical and working
- [ ] Wizard provides useful guidance
- [ ] Command help includes sufficient detail

### **AI Agent Feedback Questions**:
1. **Clarity**: Is the help system clear enough for an AI agent to use independently?
2. **Completeness**: Are there gaps in the documentation that would prevent usage?
3. **Discoverability**: Can you easily find the right command for a task?
4. **Examples**: Are the examples sufficient to understand usage patterns?

---

## 📋 **Scenario 2: Basic File Analysis**

### **Objective**: Test file parsing and structure understanding

### **Test Steps**:
1. **Analyze Python File**:
   ```bash
   gnawtreewriter analyze test_python.py
   ```

2. **List Available Nodes**:
   ```bash
   gnawtreewriter list test_python.py
   gnawtreewriter list test_python.py --filter-type function_definition
   ```

3. **Show Specific Content**:
   ```bash
   gnawtreewriter show test_python.py "0"
   gnawtreewriter show test_python.py "1.1"
   ```

### **Expected Outcomes**:
- Clear JSON output showing AST structure
- Node paths using dot notation (e.g., "0", "0.1", "1.2.1")
- Filtered listings showing only relevant node types
- Accurate content display for specific nodes

### **Success Criteria**:
- [ ] Output is parseable and well-structured
- [ ] Node paths are logical and consistent
- [ ] Content matches expected code sections
- [ ] Filter functionality works correctly

### **AI Agent Evaluation**:
1. **Usefulness**: Is the AST structure helpful for understanding code?
2. **Accuracy**: Does the parsed structure match your understanding?
3. **Navigability**: Are node paths intuitive to work with?
4. **Performance**: How responsive are the analysis commands?

---

## 📋 **Scenario 3: Safe Code Editing**

### **Objective**: Test editing workflow with validation and preview

### **Test Steps**:
1. **Preview-First Editing**:
   ```bash
   gnawtreewriter edit test_python.py "0.2.0" 'return "Hello from AI!"' --preview
   ```

2. **Apply the Edit**:
   ```bash
   gnawtreewriter edit test_python.py "0.2.0" 'return "Hello from AI!"'
   ```

3. **Verify Changes**:
   ```bash
   cat test_python.py
   gnawtreewriter history
   ```

4. **Test Validation**:
   ```bash
   # Try invalid syntax to test validation
   gnawtreewriter edit test_python.py "0" 'def broken(' --preview
   ```

### **Expected Outcomes**:
- Preview shows diff before applying changes
- Successful edits modify files correctly
- Automatic backup creation
- Syntax validation prevents invalid changes

### **Success Criteria**:
- [ ] Preview functionality works accurately
- [ ] Edits are applied correctly to target nodes
- [ ] Invalid syntax is rejected with helpful errors
- [ ] Backup files are created automatically

### **AI Agent Assessment**:
1. **Safety**: Does the preview system give you confidence in making changes?
2. **Accuracy**: Are edits applied to exactly the right location?
3. **Error Handling**: Are validation errors helpful for debugging?
4. **Workflow**: Is the edit process intuitive and efficient?

---

## 📋 **Scenario 4: Multi-File Project Work**

### **Objective**: Test project-wide operations and session management

### **Test Steps**:
1. **Start New Session**:
   ```bash
   gnawtreewriter session-start
   ```

2. **Make Multiple Changes**:
   ```bash
   gnawtreewriter edit test_python.py "0.1.1.1" 'print("Modified by AI!")'
   gnawtreewriter edit test_component.qml "0.1" 'width: 200'
   gnawtreewriter add-property test_component.qml "0" opacity real 0.8
   ```

3. **Review Session Activity**:
   ```bash
   gnawtreewriter history
   gnawtreewriter status
   ```

4. **Test Project Restoration**:
   ```bash
   # Get timestamp from before changes
   gnawtreewriter restore-project "2025-12-27T18:00:00Z" --preview
   ```

### **Expected Outcomes**:
- Session tracking groups related changes
- History shows all operations with timestamps
- Project restoration can undo multiple file changes
- Status provides clear state information

### **Success Criteria**:
- [ ] Sessions properly group related operations
- [ ] Multi-file changes are tracked correctly
- [ ] Project restoration works across files
- [ ] History provides useful operation details

### **AI Agent Evaluation**:
1. **Coordination**: Does session management help organize AI workflows?
2. **Visibility**: Is it easy to track what operations occurred?
3. **Control**: Can you confidently undo complex multi-file changes?
4. **Reliability**: Do operations complete successfully across different file types?

---

## 📋 **Scenario 5: Time-Travel and Recovery**

### **Objective**: Test temporal project management capabilities

### **Test Steps**:
1. **Create Restore Point**:
   ```bash
   # Note current timestamp
   date --iso-8601=seconds
   ```

2. **Make Several Changes**:
   ```bash
   gnawtreewriter edit test_python.py "0" 'def greeting(): return "AI was here"'
   gnawtreewriter insert test_python.py "1" 1 'def new_method(self): pass'
   gnawtreewriter edit test_component.qml "0.2" 'color: "red"'
   ```

3. **Test Different Restoration Methods**:
   ```bash
   # Session-based restoration
   gnawtreewriter history --format json | grep session_id
   gnawtreewriter restore-session "session_id_here" --preview
   
   # Time-based restoration
   gnawtreewriter restore-project "your_timestamp_here" --preview
   
   # Selective file restoration
   gnawtreewriter restore-files --since "your_timestamp" --files "*.py" --preview
   ```

4. **Test Undo/Redo**:
   ```bash
   gnawtreewriter undo --steps 2
   gnawtreewriter redo --steps 1
   ```

### **Expected Outcomes**:
- Multiple restoration methods work correctly
- Preview shows exactly what will change
- Restoration is atomic (all or nothing)
- Undo/redo provides granular control

### **Success Criteria**:
- [ ] Session restoration undoes complete workflows
- [ ] Time-based restoration works accurately
- [ ] Selective restoration affects only intended files
- [ ] Undo/redo provides expected control

### **AI Agent Assessment**:
1. **Confidence**: Does the restoration system give you confidence to experiment?
2. **Precision**: Can you restore exactly what you intend to restore?
3. **Clarity**: Is it clear what each restoration method will do?
4. **Reliability**: Do restoration operations complete successfully?

---

## 📋 **Scenario 6: QML-Specific Workflow**

### **Objective**: Test QML component editing capabilities

### **Test Steps**:
1. **Analyze QML Structure**:
   ```bash
   gnawtreewriter analyze test_component.qml
   gnawtreewriter list test_component.qml --filter-type ui_property
   ```

2. **Add Properties**:
   ```bash
   gnawtreewriter add-property test_component.qml "0" borderWidth int 2 --preview
   gnawtreewriter add-property test_component.qml "0" borderColor string '"black"'
   ```

3. **Add Child Components**:
   ```bash
   gnawtreewriter add-component test_component.qml "0" Button --content 'text: "Click me"'
   ```

4. **Edit Existing Properties**:
   ```bash
   gnawtreewriter edit test_component.qml "0.3" 'text: "Modified by AI"'
   ```

### **Expected Outcomes**:
- QML-specific commands handle component structure properly
- Properties are added in correct locations
- Child components are nested properly
- Existing properties can be modified safely

### **Success Criteria**:
- [ ] QML structure is parsed correctly
- [ ] Properties are added in valid positions
- [ ] Component hierarchy is maintained
- [ ] QML syntax remains valid after edits

### **AI Agent Evaluation**:
1. **Domain Fit**: Do QML-specific commands match your workflow needs?
2. **Structure Understanding**: Does the tool understand QML component relationships?
3. **Correctness**: Are QML modifications syntactically and semantically correct?
4. **Efficiency**: Is QML editing more efficient than text-based editing?

---

## 📋 **Scenario 7: Error Handling and Edge Cases**

### **Objective**: Test robustness and error handling

### **Test Steps**:
1. **Test Invalid Paths**:
   ```bash
   gnawtreewriter edit test_python.py "999.999" 'invalid'
   gnawtreewriter show nonexistent.py "0"
   ```

2. **Test Syntax Validation**:
   ```bash
   gnawtreewriter edit test_python.py "0" 'def broken_syntax(' --preview
   ```

3. **Test File System Issues**:
   ```bash
   # Try editing read-only file (if possible)
   chmod 444 test_python.py
   gnawtreewriter edit test_python.py "0" 'def test(): pass'
   chmod 644 test_python.py
   ```

4. **Test Recovery Scenarios**:
   ```bash
   # Try restoring to non-existent timestamp
   gnawtreewriter restore-project "2020-01-01T00:00:00Z"
   
   # Try restoring non-existent session
   gnawtreewriter restore-session "fake_session_id"
   ```

### **Expected Outcomes**:
- Clear, helpful error messages
- Graceful failure without corruption
- Guidance on how to resolve issues
- No partial or invalid states

### **Success Criteria**:
- [ ] Error messages are clear and actionable
- [ ] Failed operations don't leave files in broken states
- [ ] Recovery suggestions are provided when possible
- [ ] Tool remains stable despite errors

### **AI Agent Assessment**:
1. **Error Quality**: Are error messages helpful for troubleshooting?
2. **Recovery**: Can you easily recover from mistakes or issues?
3. **Stability**: Does the tool remain usable after encountering errors?
4. **Guidance**: Do errors provide enough information to fix problems?

---

## 📋 **Scenario 8: Integration and Workflow**

### **Objective**: Evaluate integration potential with AI workflows

### **Test Steps**:
1. **Batch Operations Simulation**:
   ```bash
   # Simulate AI agent workflow
   gnawtreewriter session-start
   gnawtreewriter analyze test_python.py
   # Parse output (as AI would)
   gnawtreewriter edit test_python.py "0.1.1.1" 'print("Step 1")'
   gnawtreewriter edit test_python.py "1.1.2.1" 'return a + b + 1'
   gnawtreewriter history --format json
   ```

2. **Error Recovery Workflow**:
   ```bash
   # Make intentional mistake
   gnawtreewriter edit test_python.py "0" 'invalid syntax'
   # Recover using undo
   gnawtreewriter undo
   ```

3. **Complex Restoration Workflow**:
   ```bash
   # Simulate "AI went wrong" scenario
   gnawtreewriter edit test_python.py "0" 'def wrong(): pass'
   gnawtreewriter edit test_python.py "1" 'class Wrong: pass'
   # Restore previous session
   gnawtreewriter restore-session "previous_session_id" --preview
   ```

### **Expected Outcomes**:
- JSON output is parseable by AI systems
- Workflow supports AI agent decision making
- Recovery mechanisms work in AI contexts
- Tool integrates smoothly with AI workflows

### **Success Criteria**:
- [ ] Output formats are AI-friendly
- [ ] Workflows support AI agent patterns
- [ ] Integration points are clear and stable
- [ ] Error handling works well in automated contexts

### **AI Agent Assessment**:
1. **Automation**: How suitable is this tool for AI agent automation?
2. **Reliability**: Can an AI agent use this tool reliably without human intervention?
3. **Integration**: What would make integration easier for AI agents?
4. **Value Proposition**: Does this tool solve real problems for AI-assisted development?

---

## 📊 **Overall Assessment Framework**

### **AI Agent Final Evaluation**

Please provide ratings (1-5 scale) and comments for each category:

#### **Usability** (/5)
- How easy is it to discover and use features?
- Are commands intuitive and well-documented?
- Is the help system sufficient for autonomous use?

#### **Functionality** (/5)  
- Do the core features work as expected?
- Are the outputs accurate and useful?
- Does it solve real development workflow problems?

#### **Reliability** (/5)
- Are operations consistent and predictable?
- Is error handling robust and helpful?
- Can you trust the tool with important code changes?

#### **Integration Potential** (/5)
- How well would this integrate with AI agent workflows?
- Are the interfaces AI-friendly?
- What would improve integration?

#### **Value Proposition** (/5)
- Does this tool provide unique value for AI-assisted development?
- Would you recommend it to other AI agents/humans?
- What are the most compelling use cases?

### **Improvement Suggestions**
- What features would make this more valuable?
- What pain points did you encounter?
- How could the AI agent experience be improved?

---

## 🎯 **Expected Test Duration**

- **Quick Assessment**: 30-45 minutes (scenarios 1-3)
- **Comprehensive Test**: 90-120 minutes (all scenarios)
- **Integration Focused**: 60 minutes (scenarios 1, 2, 4, 8)

---

## 📞 **Feedback Submission**

Please submit feedback to:
- **GitHub Issues**: https://github.com/Tuulikk/GnawTreeWriter/issues
- **Discussion**: Include your AI agent type (Claude, GPT, Gemini, etc.)
- **Format**: Test scenario results with ratings and specific feedback

---

*This test suite was designed through multi-agent collaboration to ensure comprehensive evaluation from the AI agent perspective.*

**Version**: 1.0 | **Last Updated**: 2025-12-27 | **Next Review**: After first round of AI agent testing