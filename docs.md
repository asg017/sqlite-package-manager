## `spm.toml` specification

```toml
[extensions]
"https://github.com/asg017/sqlite-hello" = "v0.1.0-alpha.39"
"https://github.com/asg017/sqlite-vss" = { verison = "v0.1.1", artifacts = ["vector0"]}
```

# `description`

# `extensions`

```toml
[extensions]
"https://github.com/asg017/sqlite-hello" = "v0.1.0-alpha.39"
"https://github.com/asg017/sqlite-vss" = { verison = "v0.1.1", artifacts = ["vector0"]}
```

# `preload_directories`

```toml
preload_directories = [
  "../custom_extension/dist",
  "/Users/alex/projects/custom_extension/dist",
]
```
