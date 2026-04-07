# Claude Code Notes

## Running commands

This project uses a Docker devcontainer. All build, test, and run commands must be executed **inside the dev container**, not on the host.

When working in this repo, prefix shell commands with `docker compose run --rm dev` if not already attached to the container, or simply run them in the VS Code integrated terminal (which is already inside the container).

### Common commands

```bash
# Build (debug)
cargo build

# Build (release)
cargo build --release

# Run a render
cargo run --release -- --scene-path scenes/<scene>.ray --width <w> --height <h> --samples <n>

# Run tests
cargo test

# Run the test-render service
docker compose run --rm test-render
```

### Container setup

- Dev container defined in `.devcontainer/` using `docker-compose.yml`
- Build artifacts are cached in the `target-cache` Docker volume at `/home/vscode/target`
- Cargo registry is cached in the `cargo-cache` Docker volume at `/usr/local/cargo/registry`
- OIDN (denoiser) is installed at `/opt/oidn-2.3.3.x86_64.linux`
- If volume permission errors occur after reopening the container, the `postAttachCommand` in `devcontainer.json` will re-chown the volumes automatically
