# Nostr Proof of Work (PoW) Service

**Note:** This is a pre-release and not production ready - unless you review the code and are happy.

To help add a base cost to Nostr spam events, relays can set a minimum proof of work required for an event (fixed or dynamic per pubkey, content, kind, etc) to be accepted. Mobiles and other devices can off-load this computation onto a `PoW Service Provider`, which leverages servers to remotely generate a target (or better) proof of work event id, before an event is signed.

This project provides an example for [NIP-XX - Proof of Work Service Provider](https://github.com/blakejakopovic/nostr_pow_service/blob/master/NIP-XX.md), which defines how a `PoW Service Provider` can offer this service using existing relay websockets using `supported_nips`, or connecting to a dedicated `PoW Service Provider`.

## Considerations
* Currently it's setup to accept requests from a pubkey whitelist only
* At present there is an incomplete lightning payment processor integration
* You may wish to generate the PoW on a different server than the host

## Getting Started

```
git clone https://github.com/blakejakopovic/nostr_pow_service
cd nostr_pow_service
cargo build --bin nostr_pow_service
```

Configure `.env` file (or use command line arguments)
```
LISTEN - binding host and port for service
RELAY_IDENTIFIER - relay identifier used for AUTH
PUBKEY_WHITELIST - comma separated hex pubkeys
MIN_POW_DIFFICILTY - minimum proof of work difficulty offered
MAX_POW_DIFFICILTY - maximum proof of work difficulty offered

or

./target/release/nostr_pow_service --help
```


Start service
```
./target/release/nostr_pow_service

```


## Usage

1. Start server
2. Connect via websocket
3. Reply to the AUTH request
4. Send POW request with event

## Development and Testing


Start service with console logging
```
RUST_LOG=info ./target/release/nostr_pow_service
```

Fetch Server Info
```
curl 'http://127.0.0.1:3030/' --header 'ACCEPT: application/nostr+json'
```

Benchmarking PoW (use as a guide only)
```
cargo run --release --bin pow_benchmark
```
