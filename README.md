# `http-request-inspector`

Tiny "echo" HTTP server

## Usage

```sh
./http-request-inspector
```
Or if you prefer docker/podman/...

```sh
docker run -it --rm -p 8080:8080 ghcr.io/mhutter/http-request-inspector:main
```

And then access http://127.0.0.1:8080 or:

```sh
curl http://127.0.0.1:8080 --json '{"msg":"It works!"}'
```

## License

[MIT](LICENSE)
