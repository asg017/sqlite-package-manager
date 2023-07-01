# sqlite-package-manager

**Warning**

> `spm` is in active development! Also consider Anton Zhiyanov's [sqlpkg](https://github.com/nalgeon/sqlpkg).

`sqlite-package-manager`, or spm for short, will be an npm-like tool for downloading and manage SQLite extensions. Unlike the pip/npm/gem bindings I described above, spm will be a language-agnostic tool, distributed as a single-binary CLI.

It's similar to npm, pip, gem, and cargo, but with a few key differences. You'll start a spm project like so:

```bash
spm init
```

Which creates an `spm.toml` file in your current directory, similar to a `requirements.txt`, `package.json`, `Gemfile`, or `Cargo.toml` file. It contains a list of "dependencies" (SQLite extensions) that you can edit by hand or with the spm CLI, like with the "spm add" command:

```bash
spm add github.com/asg017/sqlite-http
spm add github.com/asg017/sqlite-html
spm add github.com/asg017/sqlite-vss
```

Which would generate the following config in your `spm.toml`:

```toml
[extensions]
"https://github.com/asg017/sqlite-http" = "v0.1.0-alpha.7"
"https://github.com/asg017/sqlite-html" = "v0.1.2-alpha.7"
"https://github.com/asg017/sqlite-vss" = "v0.1.1-alpha.19"
```

Additionally, an `spm.lock` file will be generated alongside your `spm.toml` file, which contains detailed checksums and URLs of the downloaded SQLite extensions (similar to `package-lock.json`, `yarn.lock`, `Pipfile.lock`, `Gemfile.lock`, `Cargo.lock`).

You'll find these downloaded pre-compiled extensions in the new `sqlite_extensions/` directory alongside your project! Think of this like a `node_modules/`, `site-packages`, or `target/` directory, which contains the downloaded code of your dependencies. The extensions that are downloaded are for your current operating system and CPU architecture - in this case, the MacOS x86_64 `.dylib` files for my computer.

```
$ tree .
.
├── spm.lock
├── spm.toml
└── sqlite_extensions
    ├── html0.dylib
    ├── http0.dylib
    ├── vector0.dylib
    └── vss0.dylib

1 directory, 6 files
```

To use these extensions, you could load them manually from the `sqlite_extensions/` directory, like `.load ./sqlite_extensions/html0`.

Alternatively, use the `spm run` command and reference the extensions you want by name, without the `sqlite_extensions/` prefix. for example, for the following `build.sql` file:

```sql
.load html0
.load http0

create table foo(bar text);

insert into foo
values(html_extract(http_get('https://...')));
```

You can run the above with the `sqilte3` CLI like so:

```bash
spm run -- sqlite3 data.db '.read build.sql'
```

The `.load html0` statement will automatically load in from your `spm` environment.

You can additionally use spm activate/deactivate to create a light "spm environment" in your current shell.

```bash
$(spm activate)
sqlite3 data.db '.read build.sql'
$(spm deactivate)
```
