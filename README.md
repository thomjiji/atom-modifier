This is an experimental project and has not yet been fully implemented.

If you want to modify atom, here are some [other tools](https://thomjiji.github.io/blogs/2023/09/parse-quicktime/#implementationstools) that I have tried. Currently, the most recommended is the CLI tool [mp4edit](https://www.bento4.com/documentation/mp4edit/) in the [bento4](https://github.com/axiomatic-systems/Bento4) project. But because the upstream doesn't merge [this PR](https://github.com/axiomatic-systems/Bento4/pull/694) which added support for colr atom, gama atom, etc, so you have to compile yourself with the commits in that PR to be able to modify colr atom, gama atom, etc. OR you can use the version I compiled, which added support for colr atom, gama atom based on that PR. The executable binary is located at [bento4/bin/mp4edit](bento4/bin/mp4edit) (only target for macOS). For the usage of it, you can check the markdown file located at [bento4/commands.md](/bento4/commands.md) and the help message of mp4edit by running `./mp4edit -h`.

I also posted my notes on hacking atoms of Quicktime File Format on [my blog](https://thomjiji.github.io/blogs/2023/09/parse-quicktime/), you can check it out if you like.

```
This program allows you to modify the color primaries, transfer characteristics, matrix coefficients, and gamma value of QuickTime file. Before do the modification, it will create a backup of the input file.

Usage: atom_modifier [OPTIONS] --input-file-path <FILE> --color-primaries <INDEX_VALUE> --transfer-characteristics <INDEX_VALUE> --matrix-coefficients <INDEX_VALUE>

Options:
  -i, --input-file-path <FILE>
          The path to the input file

  -p, --color-primaries <INDEX_VALUE>
          Change the "color primaries index" to <INDEX_VALUE>

  -t, --transfer-characteristics <INDEX_VALUE>
          Change the "transfer characteristics index" to <INDEX_VALUE>

  -m, --matrix-coefficients <INDEX_VALUE>
          Change the "matrix coefficients index" to <INDEX_VALUE>

  -g, --gama-value <GAMA_VALUE>
          The gamma value to set. If not present, defaults to -1.0

          [default: -1]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
