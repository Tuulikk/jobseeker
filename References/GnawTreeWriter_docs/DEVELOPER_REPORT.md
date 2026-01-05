# GnawTreeWriter — Developer Report

**Summary**

GnawTreeWriter is a promising AST-level editing tool for QML (and other languages). It reliably produces JSON ASTs and supports `analyze`, `show`, `edit`, `insert` and `delete` operations that allow precise, structural edits to files. However, the tool has some stability and usability gaps (CLI help crash, no multi-file analyze mode, lack of human-friendly linter output) that limit its immediate usefulness as a drop-in editor/linter in CI and pre-commit workflows.

---

## ✅ What works well

- **Stable AST generation** for QML files: `gnawtreewriter analyze <file>` returns a structured JSON AST with node paths and line numbers.
- **Edit primitives are functional**: `show`, `edit`, `insert`, `delete` operate correctly (we tested edit/insert on `MainWindowComplete.qml`).
- **Designed for LLM workflows**: structured paths, node-level edits, and explicit operation types (ReplaceNode, AddProperty, InsertBefore/After, DeleteNode) suit automated or assisted editing.
- **Language coverage**: QML parser marked as stable and other language support is available (Python/Rust/TS/etc.).

---

## ⚠️ Issues & things that work poorly or not at all

- **CLI stability**: `--help` or other invocations crashed (exit 134) in some tests. That undermines developer trust for CI/pre-commit use.
- **No multi-file analyze**: `gnawtreewriter analyze` expects a single path; passing multiple files at once failed. This makes batch/CI usage cumbersome.
- **Mismatch vs. runtime QML errors**: Files that `QQmlApplicationEngine` fails to instantiate (runtime "failed to load component" / "syntax error") can still parse fine with GnawTreeWriter. The parser does not (yet) emulate the QML engine's instantiation-time checks (e.g., unresolved component imports, bindings that only fail during instantiation).
- **No human-readable linter output**: The AST JSON is detailed but not easy to scan; there is no `lint` summary mode with severity, file/line, message and short fix recommendation.
- **Node selection ergonomics**: Paths use `root.*` notation; discovering the correct path requires an extra analyze + show loop. There is no smarter selector (e.g., `component:UngnawedToolbar@id=mainToolbar` or a simple `find` helper built into the CLI).
- **No dry-run/preview or atomicity guarantees documented**: Edits apply in-place (but a preview/diff flag would be very helpful).

---

## 💡 Suggested functional improvements

Prioritize stability and developer ergonomics to make the tool usable in CI and local workflows:

1. **Fix CLI stability and exit codes**
   - Ensure `gnawtreewriter --help` and other invocations do not crash. Use predictable exit codes (0 OK, non-zero for errors).

2. **Support multi-file operations & directory analyze**
   - Add `analyze-dir` or accept multiple paths so CI jobs can run a single command and collect results.

3. **Add a human-friendly `lint` mode**
   - Produce summary lines: `file:line:col severity message [suggestion]` and optional JSON machine-readable output (for CI parsing). Example: `app/ui/qml/MainWindowComplete.qml:131:1 error "Missing SettingsWindow declaration"`.

4. **Preview / dry-run & atomic edits**
   - `--preview` flag that prints a unified diff without modifying files.
   - Option for `--backup` or `--inplace=false` and an `--apply` flag to commit changes explicitly.

5. **Improve cross-file validation**
   - Add checks that emulate instantiation-time problems: missing component imports, invalid `id` references, invalid signal/slot signatures, or other errors that would occur at runtime.

6. **Add smarter selectors and helper commands**
   - Provide selectors like `find --id mainToolbar`, `find --node-type Property --content "width:"` to reduce manual path hunting.

7. **Add an `edit --preview` or `edit --diff` output**
   - Show the exact text change that `edit` will make and require confirmation in interactive mode.

8. **Safe-mode / transaction support**
   - If multiple edits are applied as a batch, guarantee atomic application (all or none) and create a revert file (or git stash) automatically.

9. **Editor & CI integrations**
   - Provide a sample VS Code extension (or LSP hook) that uses `analyze` + `show` and offers inline code actions.
   - Add a GitHub Action / CI job workflow example that runs the analyzer and optionally auto-fixes trivial issues in a branch.

---

## 📝 Documentation & help improvements

- **Document node-path discovery pattern**: show explicit examples for QML files (how to find the `ApplicationWindow` property path for `title`, `width`, etc.), including sample `jq` patterns to query AST JSON.
- **Add `lint` and `preview` examples** to README with sample outputs.
- **Add a `recipes.md`** with common dev tasks: "Add property to component", "Change Toolbar title", "Batch-check QML in CI", "Pre-commit hook example".
- **Explain exit codes and error handling** explicitly in README.
- **Include clear examples for QML** in the `examples/` folder: show a `replace property`, `add property`, and `move element` example.

---

## 🔗 Integration ideas (how project could use it)

1. **Pre-commit check (fast)**
   - A small hook that runs `gnawtreewriter analyze` on changed `.qml` files and aborts commit on parse errors.

2. **CI Lint job**
   - `gnawtreewriter analyze` + `gnawtreewriter lint` (once added) to fail pull requests with concrete messages.

3. **Automated code fixes workstream**
   - CI can open fix branches replacing simple structural issues (e.g., update property defaults) using `edit --preview` + `--apply` with review.

4. **Editor integration (high value)**
   - A VS Code extension that runs `analyze` in the background, lets devs see AST nodes, and offers `Replace node` or `Insert` code actions with a preview.

5. **LLM-assisted edits**
   - As intended, integrate with an LLM: craft structured edit intents (ReplaceNode/AddProperty) and show diffs to dev for approval.

6. **QML runtime validation helper**
   - Optionally add a headless QML instantiation pass (sandbox) to detect runtime errors beyond parse-time issues.

---

## ✅ Quick sample pre-commit script (suggestion)

```bash
#!/usr/bin/env bash
files=$(git diff --cached --name-only --relative | grep '\.qml$' || true)
if [ -z "$files" ]; then exit 0; fi
for f in $files; do
  if ! gnawtreewriter analyze "$f" >/dev/null 2>&1; then
    echo "gnawtreewriter: parse error in $f. Commit aborted."
    exit 1
  fi
done
exit 0
```

---

## 🔬 Testing suggestions

- Add `examples/` and tests for `edit`/`insert`/`delete` flows (verify both file contents and AST after the edit).
- Add regression test for the `--help` crash and for multi-file analyze handling.
- Add integration test that simulates a QML runtime error (if possible) so the tool can be extended to detect those cases.

---

## Final notes

GnawTreeWriter is already a powerful building block for LLM-powered code editing and structural automation. With modest investments in stability (no crashes), ergonomics (multi-file analyze, lint summary, preview), and documentation (recipes + examples), it will become a highly practical tool for this repository's QML development workflow.

If you want, I can:
- add the pre-commit wrapper script + README example to this repo, and/or
- open a PR to GnawTreeWriter suggesting multi-file analyze and `lint` mode, with the examples from this report.

---

*Report generated by GitHub Copilot (Raptor mini - Preview).*