# Http Echo Server
[![Build Status](https://travis-ci.com/dejankos/http-echo.svg?branch=master)](https://travis-ci.com/dejankos/http-echo)

Simple http echo server for push/poll http requests on some path.

## Usage
```
http-echo 0.1.0

USAGE:
    http-echo [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --ip <ip>              Server ip [default: 127.0.0.1]
    -p, --port <port>          Server port [default: 8080]
    -t, --ttl <ttl>            Cache TTL [default: 900000]
    -w, --workers <workers>    Server workers - default value is number of logical CPUs
```

#### Push

```http://localhost:8080/push/CUSTOM_PATH ```

Example:
```
> curl -d '{ "name": "value" }' -H 'Content-Type: application/json' http://localhost:8080/push/example/next?a=1&b=2

> {
    "http_version": "HTTP/1.1",
    "method": "POST",
    "headers": {
      "user-agent": "curl/7.68.0",
      "content-type": "application/json",
      "accept": "*/*",
      "content-length": "19",
      "host": "localhost:8080"
    },
    "query_string": "a=1",
    "path": "/example/next",
    "body": "{ \"name\": \"value\" }",
    "time": 1596271868195,
    "ip": "127.0.0.1:41992"
  }
```

Multiple requests can be cached on the same path.

#### Poll

```http://localhost:8080/poll/CUSTOM_PATH ```

Cached requests can be polled only once:

Example:
```
> curl http://localhost:8080/poll/example/next

> [
    {
      "http_version": "HTTP/1.1",
      "method": "POST",
      "headers": {
        "user-agent": "curl/7.68.0",
        "content-type": "application/json",
        "accept": "*/*",
        "content-length": "19",
        "host": "localhost:8080"
      },
      "query_string": "a=1",
      "path": "/example/next",
      "body": "{ \"name\": \"value\" }",
      "time": 1596271868195,
      "ip": "127.0.0.1:41992"
    }
  ]
```

## License

GM Utils is licensed under the [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)