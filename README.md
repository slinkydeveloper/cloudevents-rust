# Cloudevents Rust SDK

## Features

* Supports Cloudevents 1.0 spec
* Supports reading and writing to Http Request/Response for [actix-web](https://github.com/actix/actix-web)

## TODO

* Abstract event metadata to support different spec versions + conversion between them (0.3 and 1 are the first two targeted)
* Implement json marshal/unmarshal for different spec versions
* Abstract http envelope parsing from different http libraries
