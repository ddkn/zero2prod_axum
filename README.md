# zero2prod_axum

This is my journey of going through the book [Zero 2 Production 2e](https://www.zero2prod.com/) that uses [axum](https://docs.rs/axum/latest/axum/). My intention is to learn axum due to being created by the [tokio](https://tokio.rs/) developers, and forcing myself to look at the docs versus what is verbatim in the book. I may end this prematurely, depending on how comfortable I become with [rust](https://www.rust-lang.org) development. If I do, I will indicate below.

## Progress

- [x] 1. Getting Started
- [x] 2. Building An Email Newsletter
- [ ] 3. Sign Up A New Subscriber

## Useful external packages

```
cargo install cargo-watch cargo-expand
```

For the bare minimum CI check run the following command that,

1. watches the current directory for changes
2. checks code
3. runs tests
4. runs the app

```
cargo watch -x check -x test -x run
```

If at any point, one of those steps fails, the code does not compile. It should not be added to the git repository. This could be added as a git [pre-commit](https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks) hook if necessary.

## Build

```
cargo build
```

## Run

The command line options may change over time, but the basic usage should match to run on `127.0.0.1:9000`,

```
cargo run -- --port 3000
```

### Usage

```
Usage: zero2prod_axum [OPTIONS]

Options:
  -a, --addr <ADDR>  ip address [default: 127.0.0.1]
  -p, --port <PORT>  ip port [default: 9000]
  -h, --help         Print help
```

## Documentation

```
cargo doc --open
```
