Install the repository hook after cloning:

```bash
ln -s ../../pre-commit .git/hooks/pre-commit
```

It checks formatting and Clippy on staged files. Run `cargo test` for tests and
doctests.
