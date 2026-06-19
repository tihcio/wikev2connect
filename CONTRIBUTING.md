# Contributing to WIKEv2 Connect

Thank you for your interest in contributing! This guide explains how to get involved.

## Ways to Contribute

### 1. Report Bugs

If you find a bug:

1. Check whether it has already been reported in the issue tracker
2. Open a new issue with:
   - A descriptive title
   - Fedora and Rust version you are using
   - Steps to reproduce
   - Expected vs actual behaviour
   - Output of `RUST_LOG=debug wikev2connect`

### 2. Suggest Improvements

- Open an issue with the `enhancement` label
- Describe the use case and the benefit
- Attach screenshots or mockups if relevant

### 3. Contribute Code

#### Setting Up the Development Environment

```bash
git clone https://github.com/YOUR_USERNAME/wikev2connect.git
cd wikev2connect
cargo build
```

#### Contribution Workflow

1. **Fork the repository**

2. **Create a branch**
   ```bash
   git checkout -b fix/issue-description
   # or
   git checkout -b feature/feature-description
   ```

3. **Make your changes**
   ```bash
   cargo test
   cargo fmt
   cargo clippy
   ```

4. **Commit with a descriptive message**
   ```bash
   git commit -m "fix: description of the fix (closes #123)"
   ```

   Recommended commit prefixes:
   - `fix:` — bug fixes
   - `feat:` — new features
   - `refactor:` — refactoring
   - `docs:` — documentation
   - `test:` — tests

5. **Sync with upstream**
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

6. **Push and open a Pull Request**
   ```bash
   git push origin fix/issue-description
   ```

#### Code Guidelines

- **Style**: follow `rustfmt` and `clippy` — no warnings
- **Naming**: `snake_case` for functions, `PascalCase` for types, `UPPER_SNAKE_CASE` for constants
- **Comments**: only when the *why* is non-obvious; no narrative comments
- **Tests**: add tests for new functionality
- **Doc comments**: `///` for public API items

### 4. Pull Request Template

```markdown
## Description
Brief description of the change.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Changes
- Point 1
- Point 2

## Testing Done
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manually tested

## Checklist
- [ ] `cargo fmt` and `cargo clippy` pass with no warnings
- [ ] Tests added or updated
- [ ] Documentation updated if needed

## Related Issues
Closes #123
```

---

## Build Commands

```bash
cargo check          # fast compilation check
cargo build          # debug build
cargo build --release
cargo test
cargo doc --no-deps --open
cargo fmt -- --check
cargo clippy -- -D warnings
```

## Releasing a New Version

1. Update the version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create a git tag: `git tag v0.2.0`
4. Push: `git push origin main --tags`

---

## Useful Resources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [egui docs](https://docs.rs/egui/)
- [Tokio Tutorial](https://tokio.rs/)
- [NetworkManager D-Bus API](https://developer.gnome.org/NetworkManager/stable/spec.html)
