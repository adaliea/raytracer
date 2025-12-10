# ---- Builder Stage ----
# Use a specific rust version for reproducibility
FROM rust:1.91-slim as builder

# Install the nightly toolchain and build dependencies
RUN apt-get update && \
    apt-get install -y curl tar && \
    rm -rf /var/lib/apt/lists/*

# Download and extract OIDN
ARG OIDN_URL=https://github.com/RenderKit/oidn/releases/download/v2.3.3/oidn-2.3.3.x86_64.linux.tar.gz
WORKDIR /opt
RUN curl -L ${OIDN_URL} | tar -xz

ENV LD_LIBRARY_PATH=/opt/oidn-2.3.3.x86_64.linux/lib:${LD_LIBRARY_PATH}

# Create a non-root user for development to avoid permission issues
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID
RUN groupadd --gid $USER_GID $USERNAME && \
    useradd -s /bin/bash --uid $USER_UID --gid $USER_GID -m $USERNAME && \
    mkdir -p /etc/sudoers.d && \
    echo "$USERNAME ALL=(root) NOPASSWD:ALL" > /etc/sudoers.d/$USERNAME && \
    chmod 0440 /etc/sudoers.d/$USERNAME

USER $USERNAME

RUN rustup toolchain install nightly && \
        rustup default nightly

# Set up Rust build area
WORKDIR /usr/src/app

# Copy manifests to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create a dummy project to build and cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release

# Remove dummy source and copy real source
RUN rm -rf src
COPY src ./src


# Build the actual project
# This will reuse the cached dependencies from the previous step
RUN cargo build --release

# ---- Final Stage ----
# Use a small base image
FROM debian:bookworm-slim

# Install libtbb2, a runtime dependency for OIDN
RUN apt-get update && \
    apt-get install -y libtbb12 && \
    rm -rf /var/lib/apt/lists/*

# Copy OIDN libraries from the builder stage
COPY --from=builder /opt/oidn-2.3.3.x86_64.linux/lib /usr/local/lib

# Update the linker cache to include the new libraries
RUN ldconfig

# Create a non-root user for development to avoid permission issues
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID
RUN groupadd --gid $USER_GID $USERNAME && \
    useradd -s /bin/bash --uid $USER_UID --gid $USER_GID -m $USERNAME && \
    mkdir -p /etc/sudoers.d && \
    echo "$USERNAME ALL=(root) NOPASSWD:ALL" > /etc/sudoers.d/$USERNAME && \
    chmod 0440 /etc/sudoers.d/$USERNAME

USER $USERNAME

# Copy the application binary from the builder stage
COPY --from=builder /usr/src/app/target/release/raytracer /usr/local/bin/raytracer

# Set the working directory
WORKDIR /app

# Set the command to run the application
ENTRYPOINT ["/usr/local/bin/raytracer"]
