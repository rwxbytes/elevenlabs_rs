An unofficial workspace for [ElevenLabs](https://elevenlabs.io/)

## OpenAPI coverage

Current snapshot coverage: **269 / 320 operations (84.1%)**.

All 269 local `ElevenLabsEndpoint` method/path constants match the normalized OpenAPI snapshot.
See [OPENAPI_COVERAGE.md](OPENAPI_COVERAGE.md) for coverage grouped by path.

## Feature flags

Default features are intentionally slim for `0.7.0`: only native TLS is enabled.

Minimum supported Rust version: **1.85**.

Opt into product areas and optional runtime support explicitly:

```toml
elevenlabs_rs = { version = "0.7", features = ["genai"] }
elevenlabs_rs = { version = "0.7", features = ["convai"] }
elevenlabs_rs = { version = "0.7", features = ["admin"] }
elevenlabs_rs = { version = "0.7", features = ["genai", "ws"] }
elevenlabs_rs = { version = "0.7", features = ["genai", "playback"] }
```

Available TLS choices are `native-tls`, `native-tls-vendored`, `rustls`, and
`rustls-webpki-roots`. Disable defaults when selecting a non-default TLS stack.

Maintainer commands:

```powershell
nu tools/openapi_coverage.nu fetch
nu tools/openapi_coverage.nu report
nu tools/openapi_coverage.nu check
```
