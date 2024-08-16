convert-wc
==========
This tool converts textual worst case files (`.wc`) from [CORE-MATH] into
binary ones of binary floating-point numbers.

[CORE-MATH]: https://core-math.gitlabpages.inria.fr/

Usage
-----
```sh
convert-wc [OPTIONS] <OUTPUT> <COMMAND>
```

### Options
- **`-i, --input <FILE>`**: Textual `.wc` to read
- **`-h, --help`**: Print help
- **`-V, --version`**: Print version

### Commands
- **`f32`**: Output binary in [`f32`](https://en.wikipedia.org/wiki/Single-precision_floating-point_format)
- **`f64`**: Output binary in [`f64`](https://en.wikipedia.org/wiki/Double-precision_floating-point_format)