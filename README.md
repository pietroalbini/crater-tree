# crater-tree

A small utility to generate a dependency tree of [crater][crater] regressions.
This tool is released under the MIT license.

[crater]: https://github.com/rust-lang-nursery/crater

## Usage

To use the tool, you simply need to get the experiment name (for example
`pr-44444`) and then run:

```
$ cargo run --release -- EXPERIMENT_NAME
```

The experiments' `config.json` and `results.json` will be saved in a `cache`
subdirectory to save download time in later runs.

The tool is not optimized in any way, and will take some time to output the
graph.
