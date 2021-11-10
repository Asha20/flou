## CLI

Usage:

    $ flou [FLAGS] [OPTIONS] <input>

Flags:

- `-h, --help` — Prints help information.
- `-V, --version` — Prints version information.
- `--no-default-css` — If present, the default CSS file won't be embedded. Read more [here](styling_flowchart.md).

Options:

- `--css <css>...` — Injects one or more CSS files into the generated SVG. Read more [here](styling_flowchart.md).
- `-g, --gap <size>` — Specifies the size of the grid gaps. Defaults to (50, 50).
- `-n, --node <size>` — Specifies the size of nodes in the grid. Defaults to (200, 100).
- `-o, --output <file>` — Specifies the output SVG file. Outputs to stdout if no output file is provided.

Args:
- `<input>` — The input file, written in Flou DSL. Use `-` to read from standard input instead.