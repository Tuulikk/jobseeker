# Testing Guide for Language Models

This guide provides step-by-step instructions for LLMs to test and provide feedback on GnawTreeWriter.

## Getting Started

### Step 1: Explore the Codebase

Review the documentation:
1. **README.md** - Overview, installation, basic usage
2. **docs/LLM_INTEGRATION.md** - LLM-specific integration guide
3. **docs/ARCHITECTURE.md** - Technical architecture details
4. **examples/** - Sample files in different languages

### Step 2: Build and Install

```bash
# Clone the repository (or use current directory)
cd GnawTreeWriter

# Build the release binary
cargo build --release

# Test the binary
./target/release/gnawtreewriter --help
```

## Testing Checklist

### 1. Basic Functionality

#### Test 1: Analyze Command

For each language, test file analysis:

**Python:**
```bash
./target/release/gnawtreewriter analyze examples/example.py
```

**QML:**
```bash
./target/release/gnawtreewriter analyze examples/example.qml
```

**TypeScript:**
```bash
./target/release/gnawtreewriter analyze examples/example.ts
```

**Expected Output:** JSON tree with nodes, paths, types, and content

**Questions to Consider:**
- Is the tree structure clear and understandable?
- Are node types meaningful (function_definition, class_definition, etc.)?
- Is the path format intuitive (0, 0.1, 0.1.0)?

#### Test 2: Show Command

```bash
./target/release/gnawtreewriter show examples/example.py "1"
```

**Expected Output:** Content of node at path "1"

**Questions to Consider:**
- Does the output match what you expected?
- Is the content accurate?

#### Test 3: Edit Command

```bash
./target/release/gnawtreewriter edit examples/example.py "1" "def greet(name):\n    return f'Hi, {name}!'"
```

**Expected Output:** "Edited successfully"

**Questions to Consider:**
- Did the edit apply correctly?
- Is the indentation preserved?
- Did brackets match correctly?

#### Test 4: Insert Command

```bash
./target/release/gnawtreewriter insert examples/example.py "" 0 "print('New line')"
```

**Expected Output:** "Inserted successfully"

**Questions to Consider:**
- Did the content insert at the right location?
- Was position 0 (before) vs 1 (after) correct?

#### Test 5: Delete Command

```bash
./target/release/gnawtreewriter delete examples/example.py "0"
```

**Expected Output:** "Deleted successfully"

**Questions to Consider:**
- Was the correct node removed?
- Did it handle the file correctly after deletion?

### 2. Language-Specific Testing

#### Python Testing

**Scenario 1: Modify a function**
```bash
# 1. Analyze to find function path
./target/release/gnawtreewriter analyze examples/python_complex.py

# 2. Edit the function body
./target/release/gnawtreewriter edit examples/python_complex.py "1.2" "return self.result + 10"

# 3. Verify the change
./target/release/gnawtreewriter show examples/python_complex.py "1.2"
```

**Scenario 2: Add a new function**
```bash
# Insert after last function
./target/release/gnawtreewriter insert examples/python_complex.py "2" 1 "def subtract(x, y):\n    return x - y"
```

**Questions for Python:**
- Are function definitions parsed correctly?
- Does indentation work correctly?
- Are classes and methods structured properly?

#### QML Testing

**Scenario 1: Change a property**
```bash
# 1. Check structure first
./target/release/gnawtreewriter analyze examples/example.qml

# 2. Edit a property (look for "width: 200" at path "root.2.0")
./target/release/gnawtreewriter edit examples/example.qml "root.2.0" "width: 300"

# 3. Verify
./target/release/gnawtreewriter show examples/example.qml "root.2.0"
```

**Scenario 2: Add a property to a component**
```bash
# Add opacity to Rectangle (parent path "root.2")
./target/release/gnawtreewriter insert examples/example.qml "root.2" 1 "opacity: 0.8"
```

**Questions for QML:**
- Are properties separated from components correctly?
- Does it handle nested components (Rectangle containing Text)?
- Are imports handled properly?

#### TypeScript Testing

**Scenario 1: Modify an interface**
```bash
./target/release/gnawtreewriter analyze examples/typescript_complex.ts

# Edit a property in interface (find correct path)
./target/release/gnawtreewriter edit examples/typescript_complex.ts "0.0.1" "id: number | string"
```

**Scenario 2: Add a method to a class**
```bash
# Insert into class body
./target/release/gnawtreewriter insert examples/typescript_complex.ts "1.2" 2 "validate(): boolean {\n        return this.id !== null;\n    }"
```

**Questions for TypeScript:**
- Are interfaces and classes distinct?
- Do method signatures parse correctly?
- Is type information preserved?

#### PHP Testing

```bash
# Analyze PHP file
./target/release/gnawtreewriter analyze examples/example.php

# Edit a method
./target/release/gnawtreewriter edit examples/example.php "1.2.3" "echo 'Updated message'"
```

**Questions for PHP:**
- Are classes and functions separated clearly?
- Are method definitions nested correctly in classes?
- Is PHP syntax (like $this->) handled?

#### HTML Testing

```bash
# Analyze HTML file
./target/release/gnawtreewriter analyze examples/example.html

# Add a new element
./target/release/gnawtreewriter insert examples/example.html "0" 1 "<p>New paragraph</p>"
```

**Questions for HTML:**
- Are elements hierarchical?
- Are attributes parsed as children?
- Does it handle inline CSS/JS?

### 3. Edge Cases

#### Test 1: Empty Files

```bash
# Create empty test file
echo "" > test_empty.py

# Try to analyze
./target/release/gnawtreewriter analyze test_empty.py
```

**Questions:**
- Does it handle empty files gracefully?
- Is the error message helpful?

#### Test 2: Invalid Paths

```bash
# Try to access non-existent node
./target/release/gnawtreewriter show examples/example.py "999"
```

**Questions:**
- Is the error message clear?
- Does it suggest what went wrong?

#### Test 3: Large Files

```bash
# If you have large example files, test them
./target/release/gnawtreewriter analyze large_file.py
```

**Questions:**
- Is performance acceptable?
- Does the output get truncated?

#### Test 4: Special Characters

```bash
# Test with Unicode, emojis, special characters
./target/release/gnawtreewriter analyze file_with_unicode.py
```

**Questions:**
- Does it handle UTF-8 correctly?
- Are special characters preserved?

### 4. Complex Scenarios

#### Scenario 1: Sequential Edits

```bash
# Make multiple edits in sequence
./target/release/gnawtreewriter edit examples/example.py "1" "def greet(name):\n    return f'Hello, {name}!'"
./target/release/gnawtreewriter insert examples/example.py "1" 1 "    print(greet('Test'))"
./target/release/gnawtreewriter show examples/example.py "1"
```

**Questions:**
- Do sequential edits work correctly?
- Does the file remain consistent?

#### Scenario 2: Nested Structures

```bash
# Test with deeply nested code
./target/release/gnawtreewriter analyze examples/qml_complex.qml

# Edit a deeply nested property
./target/release/gnawtreewriter edit examples/qml_complex.qml "root.2.2.4" "color: '#ff0000'"
```

**Questions:**
- Can it navigate deep nesting?
- Are paths accurate at all levels?

#### Scenario 3: Mixed Content

Test files with:
- Multiple classes/functions
- Imports/exports
- Comments
- Different code patterns

**Questions:**
- Is all content represented?
- Are comments preserved?
- Is structure consistent?

## Feedback Template

When providing feedback, use this structure:

### What Worked Well

1. [Specific feature] worked great because...
2. [Specific operation] was intuitive because...
3. The error messages for [situation] were clear because...

### What Needs Improvement

1. [Specific feature] was confusing because...
2. [Specific operation] didn't work as expected because...
3. The documentation for [topic] was unclear because...

### Specific Suggestions

1. For [language], suggest [improvement]
2. The [command] would be better if...
3. Consider adding [feature] for [use case]

### Bugs or Issues

1. Error message when [action]: "[full error text]"
2. Expected behavior: [what should happen]
3. Actual behavior: [what actually happened]
4. Reproduction steps:
   - Step 1: [command]
   - Step 2: [command]
   - Step 3: [command]

### LLM-Specific Feedback

1. As an LLM, I found it [easy/difficult] to...
2. The tree structure was [clear/confusing] because...
3. The path system [worked/was challenging] for...
4. Suggestions for making it more LLM-friendly:
   - [suggestion 1]
   - [suggestion 2]
   - [suggestion 3]

## Common Issues to Watch For

### Path Navigation

- **Issue**: Paths like "0.1" are not intuitive
- **Feedback needed**: How would you prefer to specify nodes?

### Node Types

- **Issue**: Too granular (every token is a node) or too coarse (entire file is one node)
- **Feedback needed**: What's the right balance?

### Error Messages

- **Issue**: "Node not found" without context
- **Feedback needed**: What additional information would help?

### Performance

- **Issue**: Large files take too long to parse
- **Feedback needed**: What's your tolerance for speed?

## Quick Reference

### Common Commands

```bash
# Analyze any file
gnawtreewriter analyze <file>

# Show any node
gnawtreewriter show <file> <path>

# Replace node content
gnawtreewriter edit <file> <path> <new_content>

# Insert before (position 0)
gnawtreewriter insert <file> <path> 0 <content>

# Insert after (position 1)
gnawtreewriter insert <file> <path> 1 <content>

# Delete node
gnawtreewriter delete <file> <path>
```

### Testing All Languages Quickly

```bash
for lang in py qml ts php html rs; do
    echo "Testing .$lang"
    ./target/release/gnawtreewriter analyze examples/example.$lang | head -20
done
```

## Next Steps After Testing

1. **Review your findings** - What worked? What didn't?
2. **Categorize issues** - Is it parsing, editing, documentation, or UX?
3. **Create a report** - Use the Feedback Template above
4. **Open an issue** - Submit your findings to GitHub
5. **Contribute fixes** - Submit pull requests if you're able

## Example Test Session

Here's a complete test session example:

```bash
# 1. Start with analysis
$ ./target/release/gnawtreewriter analyze examples/example.py
# Output: JSON tree with Python structure

# 2. Identify target
# I want to modify the greet() function
# It's at path "1"

# 3. Make the edit
$ ./target/release/gnawtreewriter edit examples/example.py "1" "def greet(name):\n    return f'Hi, {name}!'"
# Output: Edited successfully

# 4. Verify
$ ./target/release/gnawtreewriter show examples/example.py "1"
# Output: def greet(name):\n    return f'Hi, {name}!'

# 5. Success check
$ cat examples/example.py | head -3
# Output: #!/usr/bin/env python3
# 
# def greet(name):
#     return f'Hi, {name}!'
```

## Conclusion

Your testing and feedback is invaluable for making GnawTreeWriter truly useful for LLM-assisted code editing. 

Thank you for taking the time to test thoroughly!
