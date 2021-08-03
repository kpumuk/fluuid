# fluuid: Faster than Light UUID Generation for PostgreSQL 

## Installation

## Usage

## Benchmark

## Implementation

The project is based on [pgx](https://github.com/zombodb/pgx/), a framework for developing PostgreSQL extensions in Rust that strives to be as idiomatic and safe as possible.

UUID generation is powered by the [uuid](https://github.com/uuid-rs/uuid) crate.

## UUID v6

Universally Unique Identifiers are useful in many scenarios. And [RFC 4122](https://tools.ietf.org/html/rfc4122) describes 5 versions, indicated for use in various cases.

[This document](http://gh.peabody.io/uuidv6/) describes a proposed sixth version which in the author's opinion addresses a relatively minor but nonetheless significant shortcoming in the UUID specification, specifically when compared to the Version 1 UUIDs from the RFC.

[IETF draft proposal](https://tools.ietf.org/html/draft-peabody-dispatch-new-uuid-format) has been created, and hopefully will become a standard way to generate lexicographically sortable UUIDs ([GitHub repository](https://github.com/uuid6/uuid6-ietf-draft)).

In the meanwhile, this library implements UUID v6 for PostgreSQL.

Alternatives to UUID v6:

* [ulid (Universally Unique Lexicographically Sortable Identifier)](https://github.com/mmacedoeu/rulid.rs)
* [fuuid (Functional Universally Unique IDentifier)](https://github.com/kpdemetriou/fuuid)
* [Elasticflake](https://github.com/ppearcy/elasticflake)
* [Firebase push IDs](https://firebase.googleblog.com/2015/02/the-2120-ways-to-ensure-unique_68.html)
* [Cassandra TimeUUID](https://docs.datastax.com/en/cql-oss/3.3/cql/cql_reference/timeuuid_functions_r.html)
* [KSUID k-sortable unique identifiers](https://github.com/segmentio/ksuid)
* [Sharding & IDs at Instagram](https://instagram-engineering.com/sharding-ids-at-instagram-1cf5a71e5a5c)
* [~~https://blog.twitter.com/engineering/en_us/a/2010/announcing-snowflake.html~~](https://blog.twitter.com/engineering/en_us/a/2010/announcing-snowflake) the initial release is retired by Twitter, next version is in the works