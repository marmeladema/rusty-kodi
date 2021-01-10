# kodi-mpd-proxy

A rust async-aware [MPD](https://www.musicpd.org) to [Kodi](https://kodi.tv) proxy.

## Why

There aren't many good and open source Kodi client.
There are even less that are music oriented.
But there are plenty of pretty good and open source clients for MPD.
Also, why not?

## What

This crate provides a standalone application that exposes a TCP server that understands the MPD protocol.
It reads/parses MPD commands and forwards requests to Kodi's JSONRPC endpoint, and then reply to the client with the information extracted from Kodi:

```
+----------------+                 +----------------+                +----------------+
|                |                 |                |                |                |
|   MPD Client   |-----------------|      Proxy     |----------------|      Kodi      |
|                |                 |                |                |                |
+----------------+                 +----------------+                +----------------+
```

The proxy can live on a different machine than Kodi as long as it can reach the JSONRPC endpoint.

## Usage

As any rust crate, you can build it with cargo:

```
$ crate build
```

Then you can run it with:

```
$ crate run -- --help
kodi-mpd-proxy 0.1.0
marmeladema <xademax@gmail.com>

USAGE:
    kodi-mpd-proxy [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -k, --kodi <kodi>        Sets kodi JSON-RPC endpoint [default: http://127.0.0.1:8080/jsonrpc]
    -l, --listen <listen>    Sets listening socket address [default: 127.0.0.1:6600]
```

By default the proxy will listen on `127.0.0.1:6600` and try to reach Kodi at `http://127.0.0.1:8080/jsonrpc`.

## TODO

- Improve error handling to avoid panick'ing in tasks
- Improve documentation
