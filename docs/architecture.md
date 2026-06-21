# WireSentinel-Anonymity Architecture

Phase 13 anonymity layer — multi-provider mixnets, federation, and privacy analytics.

## Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    WireSentinel Agent                        │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ anonymity-  │  │  federation  │  │  cover-traffic   │  │
│  │    core     │──│   manager    │──│     engine       │  │
│  └──────┬──────┘  └──────────────┘  └──────────────────┘  │
│         │                                                    │
│  ┌──────┴──────┬──────────┬──────────┐                      │
│  │    Nym      │ Katzenpost│  Loopix  │  (+ Plugin/Fed)    │
│  │  (adapter)  │  (stub)   │  (stub)  │                      │
│  └─────────────┴──────────┴──────────┘                      │
│         │                                                    │
│  ┌──────┴──────┐  ┌──────────┐  ┌───────────┐              │
│  │   entropy   │  │ analytics│  │ discovery │              │
│  └─────────────┘  └──────────┘  └───────────┘              │
└─────────────────────────────────────────────────────────────┘
                              │
                    AnonymityHeartbeatPayload
                              │
                              ▼
                   WireSentinel-Controller
```

## Core abstractions

### `AnonymityBackend`

Pluggable transport interface implemented by Nym (via mixnet adapter), Katzenpost, Loopix, and future plugins. Returns `AnonymitySession` with route metadata and local SOCKS port.

### `MixnetFederationManager`

Registers multiple backends, polls health, optimizes federated routes, and validates cross-mixnet paths.

### `AdaptiveCoverTrafficEngine`

Generates background cover traffic with `Conservative`, `Balanced`, `Aggressive`, or `Maximum` profiles.

### `RouteEntropyEngine`

Scores candidate paths by hop diversity and depth; selects optimal route via `optimize()`.

### `DecoyRoutingFramework`

**In-memory simulation only.** Supports `Research`, `Simulation`, and `Lab` modes. Never touches real networks.

## Controller integration

Agents push `AnonymityHeartbeatPayload` to WireSentinel-Controller for fleet inventory, privacy scores, and federated route visibility.

## Crate dependency graph

```
anonymity-core ──┬── katzenpost
                 ├── loopix
                 ├── federation
                 ├── entropy
                 ├── discovery
                 ├── analytics
                 └── sdk

controller (DTOs only, no runtime deps on backends)
cover-traffic, decoy-routing (standalone)
```

## External dependencies

| Repository | Used by |
|------------|---------|
| `WireSentinel/shared-types` | Error types, shared models |
| `WireSentinel-Mixnet/mixnet-core` | Nym adapter base trait |
| `WireSentinel-Mixnet/transports` | `NymBackend` process wrapper |
