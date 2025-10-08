# redirect-rs

`redirect-rs` is a lightweight and efficient HTTP redirect server written in Rust. It allows you to configure multiple redirection rules using environment variables, making it highly flexible for various use cases, such as managing domain redirects, URL shorteners, or handling legacy URL structures.

## Use Case

This project is ideal for scenarios where you need a simple, performant, and easily configurable service to handle HTTP redirects. For example:

- **Domain Migration:** Redirecting old domain traffic to a new domain.
- **URL Shortening:** Creating custom short URLs that redirect to longer destinations.
- **Legacy URL Handling:** Ensuring old bookmarks and links continue to work after a site restructuring.

## Environment Variables

Redirection rules are defined using environment variables. Each rule follows the pattern `REDIRECT_<NAME>_FROM` and `REDIRECT_<NAME>_TO`.

- `REDIRECT_<NAME>_FROM`: A regular expression that the incoming request path will be matched against. The `<NAME>` part can be any identifier (e.g., `APLUS`, `SHORTURL`).
- `REDIRECT_<NAME>_TO`: The target URL for the redirect. This can include capture groups from the `_FROM` regex (e.g., `$1`, `$2`).
- `PORT`: (Optional) The port the server will listen on. Defaults to `8080`.

### Example

```bash
REDIRECT_APLUS_FROM="handymanservice.net/(.*)" \
REDIRECT_APLUS_TO='https://handymanservice.com/$1' \
PORT=80 \
./redirect-rs
```

In this example:

- Any request to `handymanservice.net/` followed by any path will be redirected.
- The captured path (e.g., `some/page`) will be appended to `https://handymanservice.com/`.
- The server will listen on port `80`.

## Setup and Running

### Prerequisites

- Rust (if building from source)
- Docker (if using the Docker image)

### Building from Source

1. Clone the repository:

   ```bash
   git clone https://github.com/darktohka/redirect-rs.git
   cd redirect-rs
   ```

2. Build the project in release mode:

   ```bash
   cargo build --release
   ```

3. Run the server with your desired environment variables:
   ```bash
   REDIRECT_APLUS_FROM="handymanservice.net/(.*)" \
   REDIRECT_APLUS_TO='https://handymanservice.com/$1' \
   PORT=80 \
   ./target/release/redirect-rs
   ```

### Using Docker

A Docker image is available at `darktohka/redirect-rs`.

1. Pull the Docker image:
   ```bash
   docker pull darktohka/redirect-rs
   ```
2. Run the Docker container with your environment variables:
   ```bash
   docker run -p 80:80 -e REDIRECT_APLUS_FROM="handymanservice.net/(.*)" \
   -e REDIRECT_APLUS_TO='https://handymanservice.com/$1' \
   darktohka/redirect-rs
   ```
   This command maps port 80 of the host to port 80 of the container and sets the redirection rules.

## Docker Compose Example

For easier management of multiple redirect rules or services, you can use `docker-compose`.

```yaml
# docker-compose.yaml
services:
  redirect-server:
    image: darktohka/redirect-rs
    ports:
      - '80:80'
    environment:
      REDIRECT_APLUS_FROM: 'handymanservice.net/(.*)'
      REDIRECT_APLUS_TO: 'https://handymanservice.com/$1'
      REDIRECT_SHORT_FROM: 'short.example.com/go/(.*)'
      REDIRECT_SHORT_TO: 'https://long.example.com/path/$1'
      PORT: 80
    restart: always
```

To run this `docker-compose` example:

1. Save the content above as `docker-compose.yaml` in your project directory.

2. Run:
   ```bash
   docker-compose up -d
   ```
   This will start the `redirect-rs` server in a detached mode, applying the specified redirect rules.
