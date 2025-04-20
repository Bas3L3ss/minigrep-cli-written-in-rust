# ✅ TODO for minigrep++

This is a feature roadmap for extending `minigrep` into a more powerful and interactive grep-like tool.

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
