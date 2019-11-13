# rload

A stupid simple and configurable Rust Loadbalancer.
It starts a HTTP-Server, checks for requests and forwards them according to your configuration.


## Configuration

Configuration files have to be in a valid YAML format, like this sample:

```yaml
schema: rload
version: '1.0'
debug: true
listen : "127.0.0.1:8080"
routes:
  - catch: bla.127.0.0.1.nip.io
    nodes:
      - http://first.thecout.com
      - http://second.thecout.com
  - catch: blu.127.0.0.1.nip.io
    nodes:
      - http://third.thecout.com
```

## Usage

1) Clone
2) cargo build
3) target/debug/loadbalancer config.yaml

## Todo

1) HTTP Basic Auth 
2) TLS
3) Health Check for backend