# Inmóvil

Blazingly fast static file hosting from the CLI using [snowboard](https://github.com/Brian3647/snowboard).

## Installing

Requirement: Cargo

```sh
cargo install --git https://github.com/Brian3647/inmovil
```

## Usage

```sh
$ inmovil <dir> [port] [--no-logs]
```

### Examples:

`inmovil target/doc 8080`
`inmovil src --no-logs`
`inmovil .github 1234`

### Arguments

#### --no-logs

If this isn't active, the program will print this on every request: `METHOD /PATH`

#### port

The port to use. This must be a valid unsigned 16-bit int.
