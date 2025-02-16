# A Prototype Message Bus

## Usage

For the system to work a server must be running, so the
first step in using the system is to start the server.

### Server

Use the `RUST_LOG` environment variable to enable logging.

The only argument is a the path to a configuration file.

#### Start the server without TLS

The only argument is the location of the config file.

```bash
RUST_LOG=debug cargo run --bin server -- --authorizations-file /authorizations-simple.yaml
```

#### Start the server with TLS

The only argument is the location of the config file.

```bash
RUST_LOG=debug cargo run --bin server -- \
    --authorizations-file etc/authorizations-simple.yaml \
    --tls $HOME/.keys/server.crt $HOME/.keys/server.key
```

### Clients

#### Start the clients without TLS

```bash
cargo run --bin client -- -h localhost -p 8080
```

#### Start the clients with TLS

On Unix

```bash
cargo run --bin client -- -h beast.jetblack.net -p 8080 --tls --cafile /etc/ssl/certs/ca-certificates.crt
```

On Mac

```bash
cargo run --bin client -- -h brick.jetblack.net -p 8080 --tls --cafile /Users/rtb/.keys/ca-certificates.crt
```

#### Using the client

With one client subscribe to a topic starting with `PUB.`.

```bash
subscribe PUB.foo
```

With another client publish data on the topic. Note that the
second argument is the "entitlements" which is a command separated list of integers.

```bash
publish PUB.foo 0 hello
```

## Design

### Overview

This is a real time message bus for publish/subscribe communication.

Client applications connect to a single hub service. The clients send
messages to the hub. The hub processes the messages and routes new messages to
the appropriate clients.

For example a client might send a message requesting a subscription to a topic.
In the case of a subscription the server will simply remember the subscription.
Another client may then publish data on the same topic. The hub will forward
the data to the subscribing client.

### Structure

The server starts by creating a `hub` task to process messages, and then listens for clients connecting.

When a client connects an "interactor" is created, and two
tasks are started: one which reads messages from the client and forwards them to
the message processor, and a second which receives messages from the message processor
and forwards them to the client.

The client read tasks communicate with the message processor task through a multi-producer
single-consumer queue to  ensure synchronization.

## Things to do

### Timeouts

Slow consumers and producers should be handled, as well as slow authenticators.

On authentication a bad actor could simply never send the send of line token.
If many clients had this behaviour this would create a task for each client
that would never complete.

### Large payloads

Maximum sizes for payloads should be introduced.

### Authentication

This is currently hardcoded to a username/password (`nobody/trustno1`).

Alternatives include:

* Password file
* LDAP
* Pluggable authentication

### Wild-carding

This is currently regex, but it might be faster to use something more
like RabbitMQ.

### Entitlements

This could be made more efficient with caching,
however we might want to put in some code to prevent bad actors swamping
the cache and crashing the host.

A mechanism to reload entitlements (e.g. on SIGHUP).

Maybe entitlements should be exposed as a pluggable module with dynamic
access.

### Serialization

This was the first thing to be written and could do with reworking.

We could wrap the packet in a length delimited frame to simplify the
serialization of primitives (i.e. to make that part non-async).

### Configuration and Command Line

It would be nice to have to control over the configuration with command line params.

Should entitlements be split into a separate file?
