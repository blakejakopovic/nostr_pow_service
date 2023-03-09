NIP-XX
======

Proof of Work Service Provider
-----------------------------------

`draft` `optional` `author:wakoinc`

This NIP defines a way for clients to request a target proof of work calculation for an event prior to publishing.

## Motivation

As a extension of both [NIP-13 Proof of Work](https://github.com/nostr-protocol/nips/blob/master/13.md) and [NIP-42 Authentication of clients to relays](https://github.com/nostr-protocol/nips/blob/master/42.md), this service (now referred to as `PoW-SP`) would provide a way for a clients to easily add computational proof of work for events they wish to publish.

This provides key benefits, as an event with a higher proof of work:

  - Is less likely to be spam as it has a higher cost to produce

  - Lower or battery powered devices can still generate events with a meaningful proof of work using a `PoW-SP`

  - Can be accepted by multiple relays as lower risk for a single one time fee, instead of having to pay to relay individual relays, or for membershps

  - Once published, anyone can re-broadcast the event more easily, which increases network censorship resistance

  - Can have a fee that's selected based on the users situation. Lower proof of work costs less, while higher proof of work costs exponentially more

## Definitions

As proof of work can take time to calculate, and to ensure the greatest compatibility for existing Nostr client, this NIP adds a new command that can be used for existing relay and client websocket communication. However, it's important to note that this MAY be provided without other relay capabilities as a stand alone service as well.

This NIP defines a new message, `POW`, which clients can send when they seek to calculate a minimum level of proof of work for an event, and the `PoW-SP` can reply with the calculated event ready to sign and publish.

When requested by clients, the message is of the following form:

```
["POW", <pre-hashed-event-json>, <target-min-proof-of-work>]
```

And, in response from the `PoW-SP`, of the following form:

```
["POW", <unsigned-event-json>]
```

The `pre-hashed-event-json` can take two forms. Example A is minimal and only includes the required fields for an event id to be calcuated. Example B is a normal fully signed event with an `id` and `signature`, however most likely with a low proof of work. Example B makes it easier for Nostr apps and libraries to re-use existing code.

Example A: A minimal pre-hashed event

```json
{
  "kind": 1,
  "tags": [],
  "content": "I like Apples"
}
```

Example B: A normal hashed and signed event

```json
{
  "id": "63665abe09f0422e59e2cac96112939ed9c22cc7e40545912bd3079e21d77711",
  "pubkey": "28f2ca82246ff16227aaba5409c15612bbe7ae49c35fead404e93281681687bc",
  "kind": 1,
  "created_at": 1651794653,
  "tags": [],
  "content": "I like Apples",
  "sig": "31ca410348f1d98f0aec250cfd22963b842eec2b46b6f52fe8079dd4850c5aed938bb1949298e7adb09b57269eb3530dd2e80f49d89e6cdbfc6d92e2ea8128f5"
}
```


## Protocol flow

For a relay with support as a `PoW-SP`, they SHOULD include `NIP-XX` in their `supported_nips` ([NIP-11 Relay Information Document](https://github.com/nostr-protocol/nips/blob/master/11.md)) definition.

A client must first authenticate with the `PoW-SP` to prove their identity (as per NIP-42). This also allows a `PoW-SP` to deduct a fee from a pre-paid account, instead of requiring a new lightning invoice per event.

Next the client MUST send a `POW` request (as above) with a a target minimum proof of work. A `PoW-SP` may have limits set for minimum and maximum target proof of work. This helps prevent abuse and generation of expensive hashes - especially as they are exponentially more expensive to generate. If the target proof of work requested is outside the range, a notice will be sent in reply.

```
["NOTICE", "restricted: target difficulty must be between <min-pow-target> and <max-pow-target>"]
```

The `PoW-SP` may wish to perform a credit check and debt the pubkey's account the fee. Alternatively, this service MAY be provided as part of a membership package without additional cost. If the client doesn't has sufficient funds, the following notice can be sent.

```
["NOTICE", "restricted: sufficient funds. current balance: <n> sat. service fee: <n> sat"]
```

As per the [NIP-13 Proof of Work](https://github.com/nostr-protocol/nips/blob/master/13.md) document, the `PoW-SP` MAY update the `created_at` field to ensure the nonce doesn't run out of values while calculating. This is an added protective measure, as spammer cannot abuse the server to pre-create high proof of work events to publish as mass at a future time.

## Unresolved as yet

* Can a client request of get their balance or credits available?
* How can a relay share a price list or formular (e.g. target_pow^1.6 sats)?
* Should requests for events that are not the authenticated pubkey be allowed? They would be unsigned anyway..


## Never Asked Questions

How does the `PoW-SP` manage payments and pubkey bookkeeping?
  - All payment, account balance, and account keeping of a `PoW-SP` is outside of this NIPs definition

Question 2
  - Bla bla bla
