# sqlite-package-manager

The missing package manager for 3rd party SQLite extensions in the `sqlite3` CLI.

```toml
[extensions]
#"github.com/asg017/sqlite-path" = "v0.2.0-alpha.1"
#"github.com/asg017/sqlite-url" = "v0.1.0-alpha.3"
#"github.com/asg017/sqlite-html" = "v0.1.2-alpha.4"
#"github.com/asg017/sqlite-http" = "v0.1.0-alpha.2"
```

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
