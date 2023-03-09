 ![](frontend-auth/public/favicon.ico)
# Getting started
## Frontend Auth React UI
Frontend development requires [Node.js](https://nodejs.org/en/download/) toolchain installed (`npm`/`npx` etc). The root of the Auth React UI is under `fronted-end` directory.

```sh
➜  cd frontend-auth
➜  npm install
➜  npm run build
```

## Postgres server
A simple way to start a database server which is required by the backend server.

```sh
➜  docker run --rm -p 5432:5432 -e "POSTGRES_PASSWORD=postgres" --name pg postgres:15
```

## Running backend
[Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) toolchain is required to compile the server. Running a database server and a generated Auth UI build is required by the backend server.
```sh
➜   RUST_LOG=trace cargo run
```

### Tests
Make cargo watch the test don't break which the code is being changes, for instance `model_` tests:
```sh
➜  cargo watch -q -c -w src -x 'test model_ -- --nocapture --test-threads=1'
```