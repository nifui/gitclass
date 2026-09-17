# Rust Backend with PostgreSQL and Docker

This guide sets up a local Rust backend with PostgreSQL running through Docker Compose.

### Assumptions

* Rust is installed through `rustup`.
* Cargo manages the Rust project.
* PostgreSQL runs in Docker.
* Database: `classroom`
* User: `postgres`
* Password: `postgres`
* PostgreSQL: `localhost:5432`
* Backend: `127.0.0.1:3000`
* `sqlx` is optional; you may use another migration tool or PostgreSQL's SQL tooling.

> **Development credentials:** `postgres` / `postgres` are for local development only. Use secure credentials in production.

---

## 1. Prerequisites

Install:

1. Git, if the project uses Git.
2. Docker and Docker Compose.
3. Rust and Cargo.
4. Optionally, the PostgreSQL CLI (`psql`).

Installation steps vary by operating system.

---

## 2. Rust and Cargo

### 2.1 Install Rust with rustup

`rustup` installs and manages Rust toolchains, including:

* `rustc` — Rust compiler
* `cargo` — build system and package manager
* Rust standard libraries
* Additional toolchains and targets

Cargo is included with Rust, so it does not need to be installed separately.

Verify the installation:

```bash
rustc --version
cargo --version
rustup --version
```

Use the stable toolchain:

```bash
rustup default stable
```

You normally only need to configure `rustup` once. Day-to-day commands are:

```bash
cargo build
cargo run
cargo test
```

### 2.2 Windows

The default Windows Rust target uses MSVC. Building native Windows applications therefore requires Microsoft's native build tools.

Install **Visual Studio Build Tools** or Visual Studio with:

> **Desktop development with C++**

The full Visual Studio IDE is not required.

Then install Rust with `rustup`:

```bash
rustup default stable
```

Verify:

```bash
rustc --version
cargo --version
```

You can also use the Windows GNU target (`x86_64-pc-windows-gnu`), but it requires a separate MinGW/GCC toolchain. MSVC is generally simpler for a standard Windows setup.

### 2.3 macOS

Install Apple's command-line developer tools:

```bash
xcode-select --install
```

Install Rust with `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Load Cargo into the current shell:

```bash
source "$HOME/.cargo/env"
```

Verify:

```bash
rustc --version
cargo --version
```
Can use pacakage manager if desired.
The full Xcode IDE is not required.

### 2.4 Linux

Install Rust with `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Verify:

```bash
rustc --version
cargo --version
```

On Debian/Ubuntu, basic build tools are commonly required:

```bash
sudo apt update
sudo apt install build-essential pkg-config
```

Additional system packages may be required by individual Rust dependencies.
Can use package manager if desired.

### 2.5 QoL 
If you want less of a headache install the sqlx-cli.
```bash
cargo install sqlx-cli --no-default-features --features postgres
sqlx --version
```
The subsequent steps require having a .env with the DATABASE_URL set. 

Verifying information
```bash
sqlx migrate info
```

Running the migrations (Will run all)
```bash 
sqlx migrate run
```

Verify the tables
```bash 
psql -U postgres -d $DATABASE_NAME 
```



## 3. PostgreSQL with Docker

Docker lets you run PostgreSQL without installing the PostgreSQL server directly on your host.

A **compose.yaml** is already setup. 

Start PostgreSQL:

```bash
docker compose up -d
```

Check the container:

```bash
docker compose ps
```

View logs:

```bash
docker compose logs postgres
```

Follow the logs:

```bash
docker compose logs -f postgres
```

PostgreSQL is now available locally at:

```text
localhost:5432
```

Bring container down and clean: 
```bash
docker compose down -v
```

The development database is:

```text
Database: classroom
User:     postgres
Password: postgres
Host:     localhost
Port:     5432
```

Your Rust backend can connect to this database using this PostgreSQL connection string :

```text
postgres://postgres:postgres@localhost:5432/classroom
```

---
## 4. Build the project 

Since sqlx utilizes macros that resolve at compile time, the database must be setup prior to compiling the backend. 
sqlx also requires a .env file detailing the database url.
If you used Docker as described above, the database url should be the PostgreSQL connection string described. 
The serve path is where the API will expose an endpoint.
```
DATABASE_URL="url here"
SERVE_PATH="127.0.0.1:3000"
```
Once you've created the .env file, the project should be ready to build and run. 
If not it might be an issue with sqlx macros not having the environment configured properly. 

Build and run it:

```bash
cd backend
cargo run --release
```

---

