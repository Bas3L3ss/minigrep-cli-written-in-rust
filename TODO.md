# ✅ TODO for minigrep++

This is a feature roadmap for extending `minigrep` into a more powerful and interactive grep-like tool.

---

## 🔍 1. Highlight the Searched Word

- [x] Parse and highlight matched keywords using ANSI escape codes (e.g., colorize matches).
- [x] Ensure it works for multiple matches per line.
- [x] Add a `--no-color` flag to optionally disable highlight (for piping or plain-text output).

### Notes:

Use `\x1b[31m` for red text and `\x1b[0m` to reset color.

---

## 🔢 2. Show Line Numbers

- [x] Prefix each matching line with its original line number.
- [ ] Add a `--line-number` or `-n` flag to toggle line numbers.
- [ ] Properly align output if line numbers go into hundreds/thousands.

---

## 📊 3. Show Metrics

- [ ] At the end of output, display:
  - [ ] Number of matching lines
  - [ ] Number of matching words (exact match count)
  - [ ] Total lines scanned
- [ ] Add `--stats` or `--summary` flag to enable metrics display
- [ ] Support optional quiet mode (`--quiet`) for scripting use

---

## 📜 4. Scroll Large Outputs

- [ ] Paginate output when matches exceed terminal height
- [ ] Integrate a pager-like tool (e.g., call `less` or similar)
- [ ] Detect terminal height dynamically (`term_size` crate or `tput lines`)
- [ ] Add `--pager` or `--scroll` flag to toggle this behavior

---

## 💡 Optional Nice-to-Haves

- [ ] Add support for regex-based search (`regex` crate)
- [ ] Add case-insensitive toggle (`-i` or `--ignore-case`)
- [ ] Highlight whole matching lines vs partial match (user toggle)
- [ ] Export results to file (`--output results.txt`)
- [ ] Add test coverage for all new features

---

## 🧪 Testing

- [ ] Write integration tests for all feature flags
- [ ] Test color and formatting in different terminals
- [ ] Benchmark performance with large files (1k+ lines)

---

## 🧼 Refactor

- [ ] Modularize search logic: separate match, highlight, and display
- [ ] Make `Config` struct support new flags (use `clap` or manually parse)
- [ ] Improve error messages (e.g., invalid file, empty search term)

---

Happy hacking! 🦀💻
