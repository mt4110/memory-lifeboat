# Architecture

Memory Lifeboat separates recovery from projection.

```text
Sources -> Quarantine -> Normalization -> Review -> User-owned store -> Adapters
```

M0 only implements the recovery side. Records imported from Atlas or exports are candidates with `instruction_class = observation`.

The tool deliberately avoids direct writes to native agent memories. Future adapters may project reviewed records into `AGENTS.md`, lifecycle hooks, or a read-only MCP server.
