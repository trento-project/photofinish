# photofinish - a little, handy tool to replay events

This tiny CLI tool aims to fulfill the need to replay some events and get fixtures.

Photofinish reads a `.photofinish.toml` file in the current working directory and:

- It outputs the fixture sets in the TOML file;
- It issues POST requests against the endpoint we give (default: `http://localhost:8081/api/collect`) with the content of the fixture files as request body.

## Usage

```sh
$ photofinish run --help
photofinish-run 
injects a specific set of events

USAGE:
    photofinish run [OPTIONS] <SET> [API_KEY]

ARGS:
    <SET>        name of the events set
    <API_KEY>    API key for the remote endpoint [default: ]

OPTIONS:
    -h, --help         Print help information
    -u, --url <url>    [default: http://localhost:8081/api/collect]
    -w <wait>          Wait interval between http requests, in milliseconds [default: 0]
```

Please refer to `photofinish help` for more commands.

## Example of `.photofinish.toml`

```toml
[first-test-scenario]
files = [
  "../../test/fixtures/discovery/host/expected_published_host_discovery.json",
  "../../test/fixtures/discovery/sap_system/sap_system_discovery_application.json",
  "../../test/fixtures/discovery/subscriptions/expected_published_subscriptions_discovery.json",
]
directories = ["target"]

[second-test-scenario]
files = [
  "third file",
  "fourth-file"
]
```

## "How do I run a fixture set?"

```sh
$ photofinish run first-test-scenario
Successfully POSTed file: ../../test/fixtures/discovery/host/expected_published_host_discovery.json
Successfully POSTed file: ../../test/fixtures/discovery/sap_system/sap_system_discovery_application.json
Successfully POSTed file: ../../test/fixtures/discovery/subscriptions/expected_published_subscriptions_discovery.json
```
