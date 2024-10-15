# JDNS

JDNS is a Rust-based DNS (Domain Name System) protocol implementation. It runs on port 5300 by default, but this can be changed by updating the [main.rs](src/main.rs) file.

## Features

-   Supports multiple DNS record types (A, CNAME, MX, TXT, AAAA).
-   Handles both valid and invalid queries with appropriate responses.
-   Simple and efficient DNS server implementation using UDP.

## Getting Started

To get started with JDNS, follow these steps:

### 1. Clone the Repository

```bash
git clone https://github.com/jveer634/jdns.git
```

### 2. Download Dependencies

```bash
cargo fetch
```

### 3. Run the Project

```bash
cargo run
```

### 4. Run in Development Mode

To run the project in development mode, use:

```bash
cargo watch -q -c -x 'run -q'
```
