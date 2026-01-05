# QML Edit Examples

This file contains practical examples for common QML editing tasks using GnawTreeWriter.

## Contents

- [Setup](#setup)
- [Replace Property Value](#replace-property-value)
- [Add Property to Component](#add-property-to-component)
- [Change Window Title](#change-window-title)
- [Move Element Position](#move-element-position)
- [Delete Property](#delete-property)
- [Working with Complex Structures](#working-with-complex-structures)

---

## Setup

All examples use this test file:

```qml
import QtQuick 2.15
import QtQuick.Controls 2.15

ApplicationWindow {
    id: mainWindow
    title: "My Application"
    width: 800
    height: 600
    visible: true

    Rectangle {
        id: background
        anchors.fill: parent
        color: "white"
    }

    Button {
        id: myButton
        text: "Click Me"
        anchors.centerIn: parent
        onClicked: console.log("Button clicked")
    }
}
```

Save as `test_window.qml` for these examples.

---

## Replace Property Value

### Goal: Change window title from "My Application" to "GnawTreeWriter Demo"

#### Step 1: Find the title property

```bash
$ gnawtreewriter find test_window.qml --content "title:"
test_window.qml: root.2.0 [Property:5]: title: "My Application"
```

Result: The title property is at path `root.2.0`

#### Step 2: Preview the change

```bash
$ gnawtreewriter edit test_window.qml root.2.0 'title: "GnawTreeWriter Demo"' --preview
```

Output:
```qml
import QtQuick 2.15
import QtQuick.Controls 2.15

ApplicationWindow {
    id: mainWindow
    title: "GnawTreeWriter Demo"
    width: 800
    height: 600
    visible: true

    Rectangle {
        id: background
        anchors.fill: parent
        color: "white"
    }

    Button {
        id: myButton
        text: "Click Me"
        anchors.centerIn: parent
        onClicked: console.log("Button clicked")
    }
}
```

#### Step 3: Apply the change

```bash
$ gnawtreewriter edit test_window.qml root.2.0 'title: "GnawTreeWriter Demo"'
Edited successfully
```

---

## Add Property to Component

### Goal: Add `radius: 10` property to the Rectangle

#### Step 1: Find the Rectangle node

```bash
$ gnawtreewriter list test_window.qml --filter-type Rectangle
root.2 [Rectangle:9-13] 
```

Result: Rectangle is at path `root.2`

#### Step 2: Preview the addition

```bash
$ gnawtreewriter insert test_window.qml root.2 2 'radius: 10' --preview
```

Note: Position `2` means "as child" (at the end of existing properties)

Output:
```qml
    Rectangle {
        id: background
        anchors.fill: parent
        color: "white"
        radius: 10
    }
```

#### Step 3: Apply the change

```bash
$ gnawtreewriter insert test_window.qml root.2 2 'radius: 10'
Inserted successfully
```

---

## Change Window Title

### Goal: Update title from "GnawTreeWriter Demo" to "Updated Title"

#### Step 1: List all nodes to find title

```bash
$ gnawtreewriter list test_window.qml --filter-type Property
    root.2.0 [Property:5] : title: "GnawTreeWriter Demo"
    root.2.1 [Property:6] : width: 800
    root.2.2 [Property:7] : height: 600
    root.2.3 [Property:8] : visible: true
    root.3.2 [Property:12] : color: "white"
    root.4.0 [Property:15] : text: "Click Me"
    root.4.1 [Property:16] : anchors.centerIn: parent
```

Result: Title is at `root.2.0`

#### Step 2: Preview the change

```bash
$ gnawtreewriter edit test_window.qml root.2.0 'title: "Updated Title"' --preview
```

#### Step 3: Apply

```bash
$ gnawtreewriter edit test_window.qml root.2.0 'title: "Updated Title"'
Edited successfully
```

---

## Move Element Position

### Goal: Move Button before Rectangle in the file

#### Step 1: Find both elements

```bash
$ gnawtreewriter find test_window.qml --node-type Button
test_window.qml: root.4 [Button:14-17] : 

$ gnawtreewriter find test_window.qml --node-type Rectangle
test_window.qml: root.3 [Rectangle:9-13] : 
```

#### Step 2: Show Button content

```bash
$ gnawtreewriter show test_window.qml root.4

Button {
    id: myButton
    text: "Click Me"
    anchors.centerIn: parent
    onClicked: console.log("Button clicked")
}
```

#### Step 3: Delete Button from original position

```bash
$ gnawtreewriter delete test_window.qml root.4
Deleted successfully
```

#### Step 4: Insert Button before Rectangle (position 0 = before)

```bash
$ gnawtreewriter insert test_window.qml root.3 0 'Button {
    id: myButton
    text: "Click Me"
    anchors.centerIn: parent
    onClicked: console.log("Button clicked")
}'
Inserted successfully
```

---

## Delete Property

### Goal: Remove `visible: true` from ApplicationWindow

#### Step 1: Find the visible property

```bash
$ gnawtreewriter find test_window.qml --content "visible:"
test_window.qml: root.2.3 [Property:8] : visible: true
```

#### Step 2: Delete it

```bash
$ gnawtreewriter delete test_window.qml root.2.3
Deleted successfully
```

---

## Working with Complex Structures

### Goal: Add a new Button with different properties

#### Step 1: Find parent component

```bash
$ gnawtreewriter list test_window.qml | grep ApplicationWindow
root.2 [ApplicationWindow:4-18] 
```

#### Step 2: Insert new Button (as child, position 2)

```bash
$ gnawtreewriter insert test_window.qml root.2 2 'Button {
    id: closeButton
    text: "Close"
    anchors.top: parent.top
    anchors.right: parent.right
    onClicked: mainWindow.close()
}'
Inserted successfully
```

---

## Batch Operations

### Goal: Change all "Click Me" text to "Action Button"

#### Step 1: Find all instances

```bash
$ gnawtreewriter find test_window.qml --content "Click Me"
test_window.qml: root.4.0 [Property:15] : text: "Click Me"
```

#### Step 2: Update each found instance

```bash
$ gnawtreewriter edit test_window.qml root.4.0 'text: "Action Button"'
Edited successfully
```

---

## Restoration Example

### Goal: Undo the last edit using backup

#### Step 1: Check backups

```bash
$ ls -lt test_window.qml/.gnawtreewriter_backups/
-rw-r--r-- 1 user user 1234 Dec 26 12:30 test_window.qml_backup_20251226_123045_123.json
-rw-r--r-- 1 user user 1234 Dec 26 12:28 test_window.qml_backup_20251226_122815_456.json
```

#### Step 2: Inspect latest backup

```bash
$ jq -r '.source_code' test_window.qml/.gnawtreewriter_backups/test_window.qml_backup_20251226_123045_123.json
```

#### Step 3: Restore from backup

```bash
$ jq -r '.source_code' test_window.qml/.gnawtreewriter_backups/test_window.qml_backup_20251226_123045_123.json > restored_test_window.qml
$ mv restored_test_window.qml test_window.qml
```

---

## Tips for QML Editing

1. **Always preview first**: Use `--preview` to see changes before applying
2. **Use find instead of manual path hunting**: `find --content` is your friend
3. **List with filters**: `list --filter-type Property` shows only what you need
4. **Check backups**: Every edit creates a backup in `.gnawtreewriter_backups/`
5. **Work from root**: When adding new components, insert as child of the root ApplicationWindow or main Rectangle

---

## Common QML Node Types

When filtering or finding, these are common node types:
- `Import`: Import statements
- `ApplicationWindow`: Main window component
- `Rectangle`: Rectangle component
- `Button`: Button component
- `Text`: Text component
- `Property`: Property assignments (e.g., `width: 800`)
- `Signal`: Signal handlers (e.g., `onClicked: ...`)

---

## Example Script for Batch Renaming

```bash
#!/bin/bash
# Rename all "myButton" to "actionButton" in all QML files

for file in *.qml; do
    echo "Processing $file..."
    paths=$(gnawtreewriter find "$file" --content "id: myButton" | jq -r '.[] | .path')

    for path in $paths; do
        gnawtreewriter edit "$file" "$path" 'id: actionButton'
        echo "  Updated $path"
    done
done
```

---

For more workflows, see [RECIPES.md](RECIPES.md).
