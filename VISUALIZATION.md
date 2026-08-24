# Module Visualization (rcvis)

`rcvis` is a binary in this crate (`src/bin/rcvis.rs`) that parses `mod` declarations
in Rust source files and renders the module tree as ASCII, PlantUML, or Graphviz.

Build once:

```sh
cargo build --bin rcvis
```

All commands below assume the working directory is the repo root
(`/Users/maxmallz/dev/tests/restaurant`) and visualize the existing
`front_of_house` module tree in `src/lib.rs`.

## Tree view (cargo-modules / cargo-tree style)

```sh
cargo run --bin rcvis -- src/lib.rs --format tree --out tree.out
cat tree.out
```

```
crate lib
├── mod tests
└── mod front_of_house
    ├── mod hosting
    └── mod serving
```

## PlantUML

```sh
cargo run --bin rcvis -- src/lib.rs --format plantuml --out modules.puml
```

Render with [PlantUML](https://plantuml.com/) (requires Java + the PlantUML jar,
or a `plantuml` CLI install):

```sh
plantuml modules.puml   # produces modules.png
```

## Graphviz

```sh
cargo run --bin rcvis -- src/lib.rs --format graphviz --out modules.dot
cargo run --bin rcvis -- /Users/maxmallz/dev/equipment-analyzer/src/lib.rs --format graphviz --out modules.dot
```

Render with [Graphviz](https://graphviz.org/) (`brew install graphviz`):

```sh
dot -Tsvg modules.dot -o modules.svg
dot -Tpng modules.dot -o modules.png
```

## Multiple source files

Each file passed becomes its own root node in the tree:

```sh
cargo run --bin rcvis -- src/lib.rs src/other.rs --format graphviz --out modules.dot
```
