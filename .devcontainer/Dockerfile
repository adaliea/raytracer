# Start from a stable Rust image
FROM rust:1.91-slim

# Install the nightly toolchain, OIDN dependencies, and other useful dev tools
RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    curl \
    tar \
    libtbb12 \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Download and extract OIDN so it's available in the environment
ARG OIDN_URL=https://github.com/RenderKit/oidn/releases/download/v2.3.3/oidn-2.3.3.x86_64.linux.tar.gz
WORKDIR /opt
RUN curl -L ${OIDN_URL} | tar -xz

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

# Set up environment variables for OIDN
ENV LD_LIBRARY_PATH=/opt/oidn-2.3.3.x86_64.linux/lib:${LD_LIBRARY_PATH}
# Switch to the non-root user
