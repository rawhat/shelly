# Shelly

A tool to generate/copy project templates, that you can *shell* into.

```bash
USAGE:
    shelly [FLAGS] [OPTIONS] [path]

ARGS:
    <path>    Path to create project [default: .]

FLAGS:
    -h, --help        Prints help information
        --no-cache
    -s, --shell       Drop into REPL after building
    -V, --version     Prints version information

OPTIONS:
    -c, --config <config>
    -t, --target <target>    A target is a language and dependencies pairing
```

## Templates

There are some default templates, and then also the option to set up a `git`
repository as a template.

#### Languages

`node` and `elixir` support the `Internal` target, wherein you specify the
language, name, and deps.

Otherwise, a GitHub repo can be provided, and it will be cloned and built with
the provided commands.  Hopefully the samples are explanatory.

The default template is generated at `~/.config/shelly/shelly.yml`.

```yaml
build_dir: /tmp/shelly
cache: true
default_target: elixir
targets:
  phoenix_react:
    Repo:
      build_args:
        - build
      build_command: docker-compose
      path: "https://github.com/rawhat/phoenix-react.git"
      shell:
        command: docker-compose
        args:
          - up
  node:
    Internal:
      language: node
      name: node
      deps:
        - name: axios
          version: 0.20.0
        - name: cheerio
          version: 1.0.0-rc.3
        - name: papaparse
          version: 5.3.0
  react:
    Repo:
      build_args:
        - install
      build_command: npm
      path: "https://github.com/rawhat/serve-react.git"
      shell:
        command: "./serve.sh"
        args: []
  rust:
    Internal:
      language: rust
      name: rust
      deps:
        - name: clap
          version: "1.0"
        - name: tokio
          version: "0.9"
  elixir:
    Internal:
      language: elixir
      name: elixir
      deps:
        - name: httpoison
          version: "1.7"
        - name: jason
          version: "1.2"
        - name: nimble_csv
          version: "1.1"
        - name: floki
          version: "0.29"
```
