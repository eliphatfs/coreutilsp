# coreutilsp

This repository aims to implement certain GNU `coreutils` in parallel. In our tests, utilities usually run 2-100 times faster than the GNU single-thread version depending on the system. `du-par` can scan 80 million files on a 300TB distributed file system in 28 minutes from a single node with 96 CPU cores.

While we exploit parallelism, we also aim to keep the memory usage low. Many parallel utilities keep lots of intermediate values in memory, leading to out of memory crashes on large, possibly distributed systems, where parallelism should be most effective and valueable! We strive to keep minimal information in memory. For example, `du-par` only consumes 29.4MiB RSS on the large file system we mentioned before.

## Get Started

Install from [crates.io](https://crates.io/crates/coreutilsp):

```bash
cargo install coreutilsp
```

This builds and installs `du-par`, `rm-par`, and `cp-par` into `~/.cargo/bin/`.

### Building from source

```bash
git clone https://github.com/eliphatfs/coreutilsp.git
cd coreutilsp
cargo build -r
```

Binaries will be in `./target/release`.

## FAQ

### How to control parallelism of utilities?

The default parallelism is the number of available threads, which is the number of logical CPU cores on most BM and VM systems, and cgroupfs limits when inside containers like docker or kubernetes. To specify a number of threads, use the `RAYON_NUM_THREADS` environment variable.

## The Utilities

### `du-par`

`du-par` is `du`, `par`-allelized.

It is very helpful to find large directories on large file systems.

We currently support the following flags, with exactly the same meanings of the GNU `du`:

```
Usage: du-par [OPTIONS] [FILES]...

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

`du-par` prints rows in GNU `du` flavor: The default unit is `1K` or `1024` bytes.

A difference is that `du-par` doesn't guarantee the order of the output. However, it does guarantee that a parent directory will be printed after its contents (post-order). For example, the following may be printed:

```
1       ./.git/objects/26
1       ./.git/refs/remotes
1       ./.git/objects/3f
1       ./.git/refs
2       ./.git/objects
```

### `rm-par`

Be careful when using the utility! It removes files really fast.

```
Usage: rm-par [OPTIONS] [FILES]...

Arguments:
  [FILES]...

Options:
  -f, --force
  -I               prompt once before removing more than three files, or
                     when removing recursively; less intrusive than -i,
                     while still giving protection against most mistakes
  -r, --recursive  [aliases: -R]
  -d, --dir
  -v, --verbose
      --help       Print help information
      --version    Print version information
```

### `cp-par`

`cp-par` is `cp`, `par`-allelized.

It copies directory trees in parallel, which is especially useful on distributed or high-IOPS file systems where single-threaded `cp -R` cannot saturate the available bandwidth.

```
Usage: cp-par [OPTIONS] [FILES]...

Arguments:
  [FILES]...

Options:
  -R, --recursive    [aliases: -r]
  -f, --force
  -i, --interactive
  -p
  -P
  -H
  -L
  -v, --verbose
      --help         Print help information
      --version      Print version information
```

Flags have the same meanings as GNU `cp`:

- `-R` — copy directories recursively, parallelizing across entries
- `-p` — preserve timestamps, permissions, and ownership
- `-P` — never follow symbolic links in source (default with `-R`)
- `-H` — follow symbolic links given as arguments only (with `-R`)
- `-L` — follow all symbolic links (with `-R`)
- `-f` — force: unlink destination and retry if it cannot be opened for writing
- `-i` — interactive: prompt before overwriting existing files

Like the other utilities, `cp-par` does not guarantee the order of operations within a directory, but it does guarantee that a parent directory is created before its contents are copied.
