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

## Found issues

In case you found new VM bugs using this fuzzer, __please__ make an issue and add the link here!

- [MODMUL operation returns wrong results for negative numbers](https://github.com/nspcc-dev/neo-go/issues/3598)

## License

Licensed under "Mozilla Public License Version 2.0"

Copyright (c) 2024 Vyacheslav Kovalevsky