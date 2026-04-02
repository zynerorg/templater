# Templater

Takes providers and consumes them into different formats while filtering them.

## Building

### Compiling

```bash
git clone https://github.com/zynerorg/templater
cd templater
cargo build
```

### Docs

```bash
cargo doc --open
```

### Development

To test use Netbox. This can be setup easily as a docker container.

## Config 

The schema can be found in ./schema.yaml

Example config

```yaml
# yaml-language-server: $schema=https://github.com/zynerorg/templater/refs/heads/main/schema.yaml

providers:
  - config:
      type: Netbox
      endpoint: "http://localhost:8000/api"
      token: "QYbMKXE9Xq6xsROnybPyCIBxJRRBBS9bz1vK5EGe"
consumers:
  - config:
      type: Null
```
