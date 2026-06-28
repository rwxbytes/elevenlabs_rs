An unofficial workspace for [ElevenLabs](https://elevenlabs.io/)

## OpenAPI coverage

Current snapshot coverage: **260 / 320 operations (81.3%)**.

All 260 local `ElevenLabsEndpoint` method/path constants match the normalized OpenAPI snapshot.
See [OPENAPI_COVERAGE.md](OPENAPI_COVERAGE.md) for coverage grouped by path.

Maintainer commands:

```powershell
nu tools/openapi_coverage.nu fetch
nu tools/openapi_coverage.nu report
nu tools/openapi_coverage.nu check
```
