const openapi_url = "https://api.elevenlabs.io/openapi.json"
const http_methods = [delete get patch post put]
const default_snapshot = "openapi/elevenlabs-openapi.paths.json"
const default_report = "OPENAPI_COVERAGE.md"
const endpoint_glob = "elevenlabs_rs/src/endpoints/**/*.rs"

def main [
    command: string = "report"
    --snapshot: path = $default_snapshot
    --report: path = $default_report
] {
    match $command {
        "fetch" => { write-snapshot $snapshot }
        "report" => { write-report $snapshot $report }
        "check" => { check-snapshot $snapshot }
        "all" => {
            write-snapshot $snapshot
            write-report $snapshot $report
            check-snapshot $snapshot
        }
        _ => {
            print --stderr $"unknown command: ($command)"
            exit 2
        }
    }
}

def write-snapshot [snapshot: path] {
    let spec = http get $openapi_url
    let operations = normalize-openapi $spec
    let payload = {
        source: $openapi_url
        openapi: $spec.openapi
        info: {
            title: $spec.info.title
            version: $spec.info.version
        }
        operation_count: ($operations | length)
        operations: $operations
    }

    mkdir ($snapshot | path dirname)
    $payload | to json --indent 2 | save --force $snapshot
    print $"wrote ($snapshot) with (($operations | length)) operations"
}

def write-report [snapshot: path, report: path] {
    let operations = load-snapshot $snapshot
    let local_endpoints = scan-local-endpoints
    let rendered = render-report $operations $local_endpoints $snapshot

    $rendered | save --force $report
    print $"wrote ($report)"
}

def check-snapshot [snapshot: path] {
    let operations = load-snapshot $snapshot
    let openapi_keys = ($operations | each { |operation| endpoint-key $operation })
    let local_endpoints = scan-local-endpoints
    let missing = ($local_endpoints | where { |endpoint| not ((endpoint-key $endpoint) in $openapi_keys) })

    if (($missing | length) == 0) {
        print $"checked (($local_endpoints | length)) local endpoints against ($snapshot)"
        return
    }

    print --stderr "local endpoint constants missing from OpenAPI snapshot:"
    for endpoint in $missing {
        print --stderr $"- ($endpoint.method) ($endpoint.path) \(($endpoint.name), ($endpoint.source):($endpoint.line))"
    }
    exit 1
}

def normalize-openapi [spec: record] {
    $spec.paths
    | transpose path item
    | each { |path_row|
        $path_row.item
        | transpose method operation
        | where { |method_row| $method_row.method in $http_methods }
        | each { |method_row|
            let tags = (try { $method_row.operation.tags } catch { [untagged] })
            {
                method: ($method_row.method | str upcase)
                path: (normalize-path $path_row.path)
                tags: $tags
                operation_id: (try { $method_row.operation.operationId } catch { null })
                summary: (try { $method_row.operation.summary } catch { null })
                primary_tag: ($tags | first)
            }
        }
    }
    | flatten
    | sort-by primary_tag path method
    | reject primary_tag
}

def load-snapshot [snapshot: path] {
    open $snapshot
    | get operations
    | each { |operation| $operation | upsert path (normalize-path $operation.path) }
}

def scan-local-endpoints [] {
    mut endpoints = []

    for file in (glob $endpoint_glob | where { |path| ($path | path basename) != "tests.rs" } | sort) {
        let source = ($file | path relative-to (pwd) | str replace -a "\\" "/")
        let lines = (open --raw $file | lines)
        mut current = empty-endpoint

        for row in ($lines | enumerate) {
            let line = $row.item
            let line_no = ($row.index + 1)
            let impl_match = ($line | parse --regex "impl ElevenLabsEndpoint for (?P<name>[A-Za-z0-9_]+)")

            if not ($impl_match | is-empty) {
                $current = {
                    active: true
                    awaiting_path: false
                    name: $impl_match.0.name
                    path: ""
                    method: ""
                    source: $source
                    line: $line_no
                }
                continue
            }

            if $current.active {
                let path_match = ($line | parse --regex "const\\s+PATH\\s*:\\s*&\\x27static\\s+str\\s*=\\s*\\x22(?P<path>[^\\x22]+)\\x22")
                if not ($path_match | is-empty) {
                    # PATH and its value on the same line.
                    $current = ($current | upsert path (normalize-path $path_match.0.path) | upsert awaiting_path false)
                } else if ($current.path == "" and ($line | str replace --regex --all "\\s" "" | str ends-with "&'staticstr=")) {
                    # `const PATH: &'static str =` with the value wrapped onto the next line.
                    $current = ($current | upsert awaiting_path true)
                } else if ($current.awaiting_path and $current.path == "") {
                    let lone = ($line | parse --regex "\\x22(?P<path>[^\\x22]+)\\x22")
                    if not ($lone | is-empty) {
                        $current = ($current | upsert path (normalize-path $lone.0.path) | upsert awaiting_path false)
                    }
                }

                let method_match = ($line | parse --regex "const\\s+METHOD\\s*:\\s*Method\\s*=\\s*Method::(?P<method>[A-Z]+)")
                if not ($method_match | is-empty) {
                    $current = ($current | upsert method $method_match.0.method)
                }

                if ($current.path != "" and $current.method != "") {
                    $endpoints = ($endpoints | append ($current | reject active awaiting_path))
                    $current = empty-endpoint
                }
            }
        }
    }

    $endpoints | sort-by path method name
}

def empty-endpoint [] {
    {
        active: false
        awaiting_path: false
        name: ""
        path: ""
        method: ""
        source: ""
        line: 0
    }
}

def normalize-path [path: string] {
    mut normalized = ($path | str trim)

    if not ($normalized | str starts-with "/") {
        $normalized = $"/($normalized)"
    }

    $normalized = (
        $normalized
        | str replace --all --regex "/:([A-Za-z_][A-Za-z0-9_]*)" "/{${1}}"
        | str replace -r "/+" "/"
    )

    if ($normalized != "/" and ($normalized | str ends-with "/")) {
        $normalized | str substring 0..-2
    } else {
        $normalized
    }
}

def endpoint-key [endpoint: record] {
    $"($endpoint.method) ($endpoint.path)"
}

# Group key derived from the path: the version prefix plus the second segment,
# e.g. `/v1/music/video-to-music` -> `/v1/music`.
def path-group [path: string] {
    let segments = ($path | split row "/" | where { |segment| $segment != "" })
    if (($segments | length) >= 2) {
        $"/($segments.0)/($segments.1)"
    } else {
        $path
    }
}

def render-report [operations: list, local_endpoints: list, snapshot: path] {
    let openapi_keys = ($operations | each { |operation| endpoint-key $operation })
    let local_keys = ($local_endpoints | each { |endpoint| endpoint-key $endpoint })
    let implemented_keys = ($openapi_keys | where { |key| $key in $local_keys } | uniq)
    let missing_local = ($local_endpoints | where { |endpoint| not ((endpoint-key $endpoint) in $openapi_keys) })
    let missing_openapi = ($operations | where { |operation| not ((endpoint-key $operation) in $local_keys) })
    let total = ($operations | length)
    let implemented = ($implemented_keys | length)
    let percent = coverage-percent $implemented $total
    let generated = (date now | date to-timezone UTC | format date "%Y-%m-%dT%H:%M:%SZ")

    mut lines = [
        "# OpenAPI coverage"
        ""
        $"Generated: ($generated)"
        $"Snapshot: `($snapshot | path expand | path relative-to (pwd) | str replace -a "\\" "/")`"
        $"Source: ($openapi_url)"
        ""
        "## Summary"
        ""
        "| Metric | Value |"
        "| --- | ---: |"
        $"| OpenAPI operations | ($total) |"
        $"| Implemented method/path pairs | ($implemented) |"
        $"| Coverage | ($percent)% |"
        $"| Local endpoint constants checked | (($local_endpoints | length)) |"
        $"| Local constants missing from snapshot | (($missing_local | length)) |"
        ""
        "## Coverage By Path"
        ""
        "| Path | Implemented | Total | Coverage |"
        "| --- | ---: | ---: | ---: |"
    ]

    let groups = ($operations | each { |operation| path-group $operation.path } | uniq | sort)
    for group in $groups {
        let group_operations = ($operations | where { |operation| (path-group $operation.path) == $group })
        let group_implemented = (
            $group_operations
            | where { |operation| (endpoint-key $operation) in $implemented_keys }
            | length
        )
        let group_total = ($group_operations | length)
        let group_percent = coverage-percent $group_implemented $group_total
        $lines = ($lines | append $"| (escape-pipe $group) | ($group_implemented) | ($group_total) | ($group_percent)% |")
    }

    $lines = ($lines | append ["" "## Local Endpoint Constants Missing From Snapshot" ""])
    if (($missing_local | length) == 0) {
        $lines = ($lines | append "All local endpoint method/path constants match the snapshot.")
    } else {
        $lines = ($lines | append ["| Endpoint | Method | Path | Source |" "| --- | --- | --- | --- |"])
        for endpoint in $missing_local {
            $lines = ($lines | append $"| `($endpoint.name)` | `($endpoint.method)` | `($endpoint.path)` | `($endpoint.source):($endpoint.line)` |")
        }
    }

    $lines = ($lines | append [
        ""
        "## Coverage Notes"
        ""
        "The official OpenAPI snapshot includes legacy or deprecated operations"
        "that are intentionally excluded from the typed endpoint roadmap:"
        ""
        "- `GET /v1/convai/conversation/get_signed_url` is the legacy underscore-path"
        "  signed URL route. The current hyphen-path route,"
        "  `GET /v1/convai/conversation/get-signed-url`, is implemented."
        "- `GET /v1/voices` is the legacy V1 voice-list route. The crate exposes"
        "  the current V2 voice-list endpoint."
        "- Dubbing paths containing `/resource/` are legacy beta-era routes."
        "  The `/v1/dubbing/project` family is the current Dubbing Project API"
        "  and remains visible below as intentionally deferred work."
        ""
    ])

    $lines = ($lines | append ["" "## OpenAPI Operations Not Yet Implemented" ""])
    if (($missing_openapi | length) == 0) {
        $lines = ($lines | append "Every OpenAPI operation has a local method/path match.")
    } else {
        for group in $groups {
            let group_missing = ($missing_openapi | where { |operation| (path-group $operation.path) == $group })
            if (($group_missing | length) == 0) {
                continue
            }

            $lines = ($lines | append [$"### ($group)" "" "| Method | Path | Summary |" "| --- | --- | --- |"])
            for operation in $group_missing {
                let summary = if $operation.summary == null {
                    if $operation.operation_id == null { "" } else { $operation.operation_id }
                } else {
                    $operation.summary
                }
                $lines = ($lines | append $"| `($operation.method)` | `($operation.path)` | (escape-pipe $summary) |")
            }
            $lines = ($lines | append "")
        }
    }

    $lines = ($lines | append [
        "## Maintainer Commands"
        ""
        "```powershell"
        "nu tools/openapi_coverage.nu fetch"
        "nu tools/openapi_coverage.nu report"
        "nu tools/openapi_coverage.nu check"
        "```"
        ""
    ])

    $lines | str join "\n"
}

def coverage-percent [implemented: int, total: int] {
    if $total == 0 {
        100.0
    } else {
        (($implemented / $total) * 100 | math round --precision 1)
    }
}

def escape-pipe [value] {
    if $value == null {
        ""
    } else {
        $value | into string | str replace -a "|" "\\|" | str replace -a "\n" " "
    }
}
