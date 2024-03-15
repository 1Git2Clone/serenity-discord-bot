FROM rust:latest

ENV SQLX_OFFLINE=true

# Set Rust environment variable (optional)
# ENV RUST_BACKTRACE=1

WORKDIR /app

# Copy all of the project files
COPY . .

# Install dependencies
RUN cargo install --locked --path .

# Build the project in release mode
RUN cargo build --release

# Set the command to run the application
CMD ["cargo", "run", "--release"]
