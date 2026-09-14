# Test suite coverage

The five-case target in the quality plan is a planning heuristic, not a reason
to merge independent contracts into opaque table tests. The larger suites stay
split by invariant so failures identify the rule that regressed:

- `campaign.rs`: authored map geometry, sources, schedules, and progression.
- `devices.rs`: placement, topology, device behavior, and showcase isolation.
- `mission.rs`: lifecycle, hazards, objectives, tutorial gates, and saves.
- `simulation.rs`: terrain, fluid transfer, reactions, capacity, and ledgers.
- `state.rs`: ticking, partition invariance, persistence, and compatibility.

The smaller suites use five focused cases where their controls or economy
contracts are naturally independent. Every suite remains deterministic and
avoids duplicating the same assertion under different names.
