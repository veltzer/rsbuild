# Installation

## Download pre-built binary (x86_64 Linux)

Using the GitHub CLI:

```bash
gh release download latest --repo veltzer/rsb --pattern 'rsb' --output rsb --clobber
chmod +x rsb
sudo mv rsb /usr/local/bin/
```

Or with curl:

```bash
curl -Lo rsb https://github.com/veltzer/rsb/releases/download/latest/rsb
chmod +x rsb
sudo mv rsb /usr/local/bin/
```

## Build from source

```bash
cargo build --release
```

The binary will be at `target/release/rsb`.
