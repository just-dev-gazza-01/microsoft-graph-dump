# microsoft-graph-reportee-dump

# Pre-requisites

- Cargo (Rust) installed (https://www.rust-lang.org/tools/install)

# How to run

- Setup environment variable `ACCESS_TOKEN` before running the program.

    + Access token can be generated from the graph-explorer. URL - https://developer.microsoft.com/en-us/graph/graph-explorer
    + In Linux/Unix systems -> `export ACCESS_TOKEN=<token_value>`
    + For windows -> `set ACCESS_TOKEN=<token_value>`

- Run `cargo run --release > output_dump.csv` in the root directory of the project.

    + In the prompt, enter the full or part of the user's display name to start traversing the graph.
    + This shall write the output to the file `output_dump.csv` in the root directory of the project. You may provide an alternate path to write the output to.

## Install the CLI permanently

- Run `cargo install --path .` in the root directory of the project.

    + Now you may run `microsoft-graph-reportee-dump > output_dump.csv` from any directory to generate the report.

## Use graph-explorer to gather token and test APIs.

URL: https://developer.microsoft.com/en-us/graph/graph-explorer
