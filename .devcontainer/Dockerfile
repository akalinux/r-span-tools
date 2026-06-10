FROM mcr.microsoft.com/devcontainers/rust:2-1-trixie

RUN --mount=type=cache,target=/var/cache/apt \
    --mount=type=cache,target=/var/lib/apt \
    apt-get update && \
    apt-get install -y vim wget curl llvm && \
    cargo install cargo-readme

RUN cd /;curl -L https://git.io/misspell | bash
 
ENV PATH=/usr/local/go/bin:$PATH
CMD ["/bin/bash"]
