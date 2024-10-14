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

```sh
make
cargo run --release
```

Scripts with issues (different output) will be put into `./crashes`.

File names are base64 encoded scripts (using URL alphabet - __not valid for VM__!!!).

Contents of these files are __NOT__ valid script bytes (they are used internally by LibAFL for serialization).

Instead, find `*.metadata` files, where outputs for both VMs are saved and encoded base64 script can be found (and more info in the future).

## Issues Found

In case you find new VM bugs using this fuzzer, __please__ make an issue and add the link here!

| Name                                                         | Description                          | Link                                                    |
| ------------------------------------------------------------ | ------------------------------------ | ------------------------------------------------------- |
| MODMUL operation returns wrong results for negative numbers | [Description](./bugs/neo-go-3598.md) | [Link](https://github.com/nspcc-dev/neo-go/issues/3598) |
| MODPOW operation returns wrong results when base is negative | [Description](./bugs/neo-go-3612.md) | [Link](https://github.com/nspcc-dev/neo-go/issues/3612) |
| PACKMAP operation keeps duplicate entries | [Description](./bugs/neo-go-3613.md) | [Link](https://github.com/nspcc-dev/neo-go/issues/3613) |

## License

Licensed under "Mozilla Public License Version 2.0"

Copyright (c) 2024 Vyacheslav Kovalevsky