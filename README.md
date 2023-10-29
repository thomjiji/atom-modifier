This is an experimental project and has not yet been fully implemented.

If you want to modify atom, here are some [other tools](https://thomjiji.github.io/posts/2023-09-18_parse_quicktime/#implementationstools) that I have tried. Currently, the most recommended is the CLI tool [mp4edit](https://www.bento4.com/documentation/mp4edit/) in the [bento4](https://github.com/axiomatic-systems/Bento4) project. But because the upstream doesn't merge [this PR](https://github.com/axiomatic-systems/Bento4/pull/694) which added support for colr atom, gama atom, etc, so you have to compile yourself with the commits in that PR to be able to modify colr atom, gama atom, etc. OR you can use the version I compiled, which added support for colr atom, gama atom based on that PR. The executable binary is located at [bento4/bin/mp4edit](bento4/bin/mp4edit) (only target for macOS). For the usage of it, you can check the markdown file located at [bento4/commands.md](/bento4/commands.md) and the help message of mp4edit by running `./mp4edit -h`.

I also posted my notes on hacking atoms of Quicktime File Format on [my blog](https://thomjiji.github.io/posts/2023-09-18_parse_quicktime), you can check it out if you like.

```
Usage: atom_modifier [OPTIONS] --input-file-path <FILE> --primary <INDEX_VALUE> --transfer-function <INDEX_VALUE> --matrix <INDEX_VALUE>

Options:
-i, --input-file-path <FILE>

-p, --primary <INDEX_VALUE>
        Change the "color primaries index" to <INDEX_VALUE>
-t, --transfer-function <INDEX_VALUE>
        Change the "transfer characteristics index" to <INDEX_VALUE>
-m, --matrix <INDEX_VALUE>
        Change the "matrix coeffients index" to <INDEX_VALUE>
-g, --gama-value <GAMA_VALUE>
        Change the Gamma value to <GAMA_VALUE> if gama atom present [default: -1]
-h, --help
        Print help
-V, --version
        Print version
```
