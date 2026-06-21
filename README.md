# WireSentinel-Anonymity

Advanced anonymity layer for the WireSentinel ecosystem — multi-provider mixnets, federation, cover traffic, entropy optimization, and controller heartbeat integration (Phase 13).

## Crates

| Crate | Purpose |
|-------|---------|
| `anonymity-core` | `AnonymityBackend` trait, types, security policy, Nym adapter |
| `katzenpost` | Katzenpost backend + gateway discovery |
| `loopix` | Loopix backend + provider discovery |
| `federation` | `MixnetFederationManager` for cross-mixnet routing |
| `cover-traffic` | `AdaptiveCoverTrafficEngine` with intensity profiles |
| `entropy` | `RouteEntropyEngine` for path scoring and optimization |
| `decoy-routing` | `DecoyRoutingFramework` (Research/Simulation/Lab only) |
| `discovery` | `AnonymousDiscoveryEngine` (in-memory registry) |
| `analytics` | Path diversity, federation diversity, anonymity set estimates |
| `sdk` | `AnonymityPlugin` trait and manifest |
| `controller` | `AnonymityHeartbeatPayload` DTOs for controller agents |

## Build & test

```bash
cargo test --workspace
```

## Dependencies

- [`WireSentinel/shared-types`](../WireSentinel/shared-types) — shared DTOs and error types
- [`WireSentinel-Mixnet`](../WireSentinel-Mixnet) — Nym transport via `mixnet-core` and `transports`

## Safety notes

- **Decoy routing** operates in-memory only (`Research`, `Simulation`, `Lab` modes). No real network traffic is emitted.
- Katzenpost and Loopix backends use local SOCKS stubs for CI/development, matching the Nym stub pattern.
