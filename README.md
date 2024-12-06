# N3onDiff (NeonDiff)

Differential fuzzing for Neo (N3) blockchain virtual machine based on LibAFL

## Install

Clone repository:

```sh
git clone --recursive https://github.com/Slava0135/N3onDiff
```

Install dependencies:

- neo-go
  - make
  - go 1.22+
- neo
  - dotnet-sdk 8.0
  - aspnet-runtime 8.0
- N3onDiff
  - make
  - rust 1.80+ (__nightly__)

## Usage

To compile harness:

```sh
make
```

This will put VM executables in `./harness`

To run fuzzer:

```sh
cargo run --release --cores 0,1,2... (or 0-12 and etc.)
```

You can make a binary as well:

```sh
cargo build --release
```

For more options use:

```sh
./target/release/n3on-diff --help
```

Scripts with issues (different output) will be put into `./crashes`.

## Getting script bytecode

File names are base64 encoded scripts (__NOTE: base64 here uses URL alphabet and can't be used in VM__).

>Contents of these files are __NOT__ valid script bytes either (they are used internally by LibAFL for serialization), ignore them.

Instead, look for `*.metadata` files, where:

- Outputs for both VMs are saved
- Encoded base64 script can be found that can be used in VM.

Scripts can be run manually:

```sh
./harness/neo-go <BASE64>
```

Or you can load scripts with original [neo-go](https://github.com/nspcc-dev/neo-go) CLI for extra debug info:

```sh
# neo-go
make
./bin/neo-go vm # will launch interactive vm
```

```sh
loadbase64 <BASE64>
ops # print opcodes
run
```

## Coverage

Coverage for each client/runner is collected under `/tmp/N3onDiff/0/go-cover-merged/`, `/tmp/N3onDiff/1/go-cover-merged/`, etc...

>__NOTE: backup coverage data from /tmp, because it will be lost after reboot__.

When fuzzing is finished, you would want to merge coverage directories from each client together.

In case you are not familiar with new Golang `covdata` tool:

Merging:

```sh
go tool covdata merge -i=fst_dir,snd_dir -o merged
```

Profile in old format:

```sh
go tool covdata textfmt -i=merged -o profile.txt
```

## Issues Found

In case you find new VM bugs using this fuzzer, make an issue and add the link here!

| Name                                                         | Description                          | Link                                                    |
| ------------------------------------------------------------ | ------------------------------------ | ------------------------------------------------------- |
| MODMUL operation returns wrong results for negative numbers | [Description](./bugs/neo-go-3598.md) | [Link](https://github.com/nspcc-dev/neo-go/issues/3598) |
| MODPOW operation returns wrong results when base is negative | [Description](./bugs/neo-go-3612.md) | [Link](https://github.com/nspcc-dev/neo-go/issues/3612) |
| PACKMAP operation keeps duplicate entries | [Description](./bugs/neo-go-3613.md) | [Link](https://github.com/nspcc-dev/neo-go/issues/3613) |

## License

Licensed under "Mozilla Public License Version 2.0"

Copyright (c) 2024 Vyacheslav Kovalevsky
