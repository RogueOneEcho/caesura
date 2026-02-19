# Contributing

Contributions are welcome!

It's always worth [joining the discussion](https://github.com/RogueOneEcho/caesura/discussions) before starting work.

### Building from source

Refer to the [build guide](docs/BUILD.md) for details.

### LLM/AI

LLM and AI tools can be useful development tools, however, they often make obvious mistakes or write overcomplicated solutions.

When using LLMs:
- You are responsible for every line of code the tool outputs
- You must understand and be able to explain the code the tool generates
- Disclose significant use in your PR description or discussions
- Do not submit LLM output verbatim in PR descriptions, commit messages, or issue discussions
- Ensure all code is tested and reviewed before submission

Refer to the [Jellyfin policy](https://jellyfin.org/docs/general/contributing/llm-policies/) for more detailed guidance.

### Linting and Formatting

Always run clippy and format before committing:

```bash
cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features && cargo fmt --all
```

Clippy must produce zero warnings.

### Commit Messages

Commit messages follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification:

```
type(scope): description
```

Examples:
- `fix(verify): handle missing composer tag`
- `feat(batch): add wait interval option`
- `refactor(transcode): simplify the transcode logic`

### Sign your commits

You should always [sign your commits](https://docs.github.com/en/authentication/managing-commit-signature-verification/signing-commits).

### Design Principles

The following conventions are followed:
- [object oriented patterns](https://refactoring.guru/design-patterns/catalog)
- with SOLID principles
- and [dependency injection](https://en.wikipedia.org/wiki/Dependency_injection)

### Code Style

- **Never use `unwrap()`** - use `expect()` with a descriptive message
- Prefer `Target::from(value)` over `value.into()`
- Keep inline comments minimal - prefer well-named functions
- If there are only a few simple, focused tests they can be in inline `#[cfg(test)]` blocks
- If tests are numerous or complex they should be located in `tests/` subdirectories

### Testing

New features and bug fixes should be covered by tests.

Refer to the [testing guide](docs/TESTING.md) for details.

### Screencasts

Documentation includes GIF screencasts generated from VHS tape files in the [assets-caesura](https://github.com/RogueOneEcho/assets-caesura) repository.

## License

By contributing you agree that your contributions will be licensed under the [AGPL-3.0 license](LICENSE.md).
