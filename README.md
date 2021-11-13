# ndjson

Formats and colorizes newline delimited JSON for better readability.

Example:
```json
{"type":"json","value":42,"multiline":"line1\nline2","array":[1,2,3]}
```
```
type: json value: 42 multiline: line1
line2 array: [1, 2, 3]
```

## Usage

```sh
ndjson < file
tail -f file | ndjson
docker logs --tail 100 -f container 2>&1 | ndjson
kubectl logs --tail 100 -f pod | ndjson
```

## Install

### With cargo

```sh
cargo install ndjson
```

### From binaries

Download the prebuilt binaries from the [Releases](https://github.com/rojul/ndjson/releases) page.
