# sqlite-package-manager

The missing package manager for 3rd party SQLite extensions in the `sqlite3` CLI.

- `init` - just make the spm.toml
- `install [package]`
  - if package: add to spm.toml, refresh spm.lock, download
  - if no package: refresh spm.lock, download
- `add pkg` ???
- `rm pkg` ???
- `ci`
  - if spm.toml and spm.lock don't match, exit
  - install from spm.lock,

```
"github.com/asg017/sqlite-path"
"https://github.com/asg017/sqlite-path"
"gh:asg017/sqlite-path"
"github.com/asg017/sqlite-path@v1.2.1"
"https://github.com/asg017/sqlite-path@v1.2.3"
"gh:asg017/sqlite-path@v1.2.3"
```

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
- [ ] upgrade
- [ ] audit

```yaml
- uses: asg017/setup-spm@v1
  with:
    spm-version: v0.2
- run: spm run -- sqlite3 ':memory:' '.read scrape.sql'
```
