# sqlite-package-manager

The missing package manager for 3rd party SQLite extensions in the `sqlite3` CLI.

## TODO

- [ ] `spm activate` and `spm deactivate`
- [ ] `spm add [pkg]`
- [ ] `spm install [pkg]`
- [ ] `spm install`
- [ ] platform dependent dylib lookup
- [ ] binary distribution
- [ ] gh action setup

```yaml
- uses: asg017/setup-spm@v1
  with:
    spm-version: v0.2
- run: spm run -- sqlite3 ':memory:' '.read scrape.sql'
```
