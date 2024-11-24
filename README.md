# zero2prod_axum

This is my journey of going through the book [Zero 2 Production 2e](https://www.zero2prod.com/). However, I am translating the work from [actix](https://actix.rs/) which the book uses to my desired framework [axum](https://docs.rs/axum/latest/axum/). My intention is to learn axum due to being created by the [tokio](https://tokio.rs/) developers, and forcing myself to look at the docs versus what is verbatim in the book. I may end this prematurely, depending on how comfortable I become with [rust](https://www.rust-lang.org) development. If I do, I will indicate below.

Some notable changes include the following substitutions,

* **sqlite**: instead of `postgresql`
* **toml** instead of `config`

## Progress

- [x] 1. Getting Started
- [x] 2. Building An Email Newsletter
- [x] 3. Sign Up A New Subscriber
- [x] 4. Telemetry
  - ~[ ] 4.1 to 4.2~ Unecessary since it migrates from `log` -> `tracing`
  - ~[ ] 4.5.14~ Uncessary because of axum's setup
- [x] 5. Going Live
  - ~[ ] 5.3.7~ Using Sqlite so unecessary
  - ~[ ] 5.4~ Not deploying to digital ocean, not implementing secrets due to extra code for toml
- [x] 6. Reject Invalid Subscribers #1
- [ ] 7. Reject Invalid Subscribers #2
  - [ ] 7.7 Sending A Confirmation Email

### Warning

In 3.10.1 we create a tempoary named database with a uuid4 name. If the test completes it deletes the database. If it fails it does not delete the database at the moment. We could add a `panic` to clear up all uuid databases, if need be, but this is a minor detail for learning at the moment.

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

### Debian/Ubuntu

If you are on Debian/Ubuntu you will need the following external packages installed for sqlx with sqlite + TLS support,

```
sudo apt install pkg-config libssl-dev
```

You might also need a C/C++ compiler for sqlite linking

```
sudo apt install clang
```

## Build

```
cargo build
```

## Run

The command line options may change over time, but the basic usage should match to run on `127.0.0.1:9000`,

```
RUST_LOG=trace cargo run
```

### Usage

Specify the configuration in `settings.{local,production}.toml` file laid out as such:

```
addr = "127.0.0.1"
port = 9000

[database]
name = "demo.db"

[email_client]
base_url = "localhost"
sender_email = "test@example.com"
authorization_token = "my-secret-token"
timeout_milliseconds = 10000
```

### Docker

Build the docker image

```
docker build -t z2pa -f docker/Dockerfile .
```

Run the image and let's say port 9000 is already busy on the host,

```
docker run -e RUST_LOG=debug -p 9001:9000 z2pa
```

#### Settings

The settings file (settings.toml) is currently organized as such for a sqlite database.

```
addr = "127.0.0.1"
port = 9000

[database]
name = demo.db
```

## Documentation

```
cargo doc --open --lib --no-deps
```

## Database

We are using `sqlx` as per the books request, but using `sqlite` instead to keep the setup minimal, instead of `postgresql` used in the book.

```
cargo add sqlx -F sqlx/sqlite,runtime-tokio,rustls/sqlite,migrate,uuid
cargo install sqlx-cli --no-default-features -F sqlite,rustls
```

Create the Database

```
export DATABSE_URL=sqlite:./demo.db
sqlx database create
sqlx migrate add create_signup_table
```

Edit the newly created migration file to include the following,

```
CREATE TABLE IF NOT EXISTS subscriptions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    subscribed_at TEXT DEFAULT (datetime('now','utc')) -- Stores the timestamp in UTC
);
```

We are going to use `UUID`s for `id`, but `sqlite` does not natively support them so using `TEXT` instead. Also, we do not have timezones so we will save all the time as UTC for simplicity, this is easily achieved by using `uuid::Uuid::new_v4().to_string()`. We can use [chrono](https://docs.rs/chrono/latest/chrono/) for UTC and handling timezones if necessary. As with sqlite, we need to convert the chrono utc time to a sqlite compatible string, `"%Y-%m-%d %H:%M:%S"`.

Run the migrations

```
sqlx migrate run
```
