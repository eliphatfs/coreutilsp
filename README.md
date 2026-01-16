# coreutilsp

This repository aims to implement certain GNU `coreutils` in parallel. In our tests, utilities usually run 2-100 times faster than the GNU single-thread version depending on the system. `padu` can scan 80 million files on a 300TB distributed file system in 28 minutes from a single node with 96 CPU cores.

While we exploit parallelism, we also aim to keep the memory usage low. Many parallel utilities keep lots of intermediate values in memory, leading to out of memory crashes on large, possibly distributed systems, where parallelism should be most effective and valueable! We strive to keep minimal information in memory. For example, `padu` only consumes 29.4MiB RSS on the large file system we mentioned before.

## Get Started

```bash
cargo build -r
```

You will usually find the binaries in `./target/release`.

## FAQ

### How to control parallelism of utilities?

The default parallelism is the number of available threads, which is the number of logical CPU cores on most BM and VM systems, and cgroupfs limits when inside containers like docker or kubernetes. To specify a number of threads, use the `RAYON_NUM_THREADS` environment variable.

## The Utilities

### `padu`

`padu` is `pa`-rallel `du`.

It is very helpful to find large directories on large file systems.

We currently support the following flags, with exactly the same meanings of the GNU `du`:

```
Usage: padu [OPTIONS] [FILES]...

Arguments:
  [FILES]...

Options:
  -a, --all
  -h, --human-readable
  -s, --summarize
  -d, --max-depth <MAX_DEPTH>
  -S, --separate-dirs
  -c, --total
  -t, --threshold <THRESHOLD>  [default: 0]
      --help                   Print help information
      --version                Print version information
```

`padu` prints rows in GNU `du` flavor: The default unit is `1K` or `1024` bytes.

A difference is that `padu` doesn't guarantee the order of the output. However, it does guarantee that a parent directory will be printed after its contents (post-order). For example, the following may be printed:

```
1       ./.git/objects/26
1       ./.git/refs/remotes
1       ./.git/objects/3f
1       ./.git/refs
2       ./.git/objects
```
