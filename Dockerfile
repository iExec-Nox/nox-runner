FROM rust:1.92.0-alpine3.23 AS builder

WORKDIR /app

# Install build dependencies with pinned versions
RUN apk add --no-cache openssl-dev

# Copy manifest and source files
COPY . .

# Build the application
RUN cargo build --release

FROM alpine:3.23 AS runtime

# Set working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/nox-runner .

# Run the application
ENTRYPOINT ["/app/nox-runner"]
