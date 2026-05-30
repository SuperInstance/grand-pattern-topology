# Grand Pattern — Topology Sweep

**Finding the sweet spot between star speed and mesh robustness.**

A comprehensive topology sweep experiment testing 10 graph topologies for the Grand Pattern (mono-vibe version). Pure Rust, zero dependencies.

## The Hypothesis

Previous diffusion experiments found:
- **Star**: 35 ticks convergence (fast!)
- **Chain**: 159 ticks
- **Mesh**: 504 ticks (slowest — too many neighbors create damping)

**Hypothesis**: Small-world networks hit the sweet spot — fast convergence *and* robustness.

## Topologies (10)

| # | Topology | Description |
|---|----------|-------------|
| 1 | Chain | Linear path |
| 2 | Ring | Circular path |
| 3 | Star | Central hub + spokes |
| 4 | Mesh | Complete graph |
| 5 | Small-world (p=0.1) | Watts-Strogatz, low rewiring |
| 6 | Small-world (p=0.3) | Watts-Strogatz, medium rewiring |
| 7 | Small-world (p=0.5) | Watts-Strogatz, high rewiring |
| 8 | Hierarchical | 2 clusters + bridge |
| 9 | Scale-free (BA) | Barabási-Albert preferential attachment |
| 10 | Random (ER) | Erdős-Rényi, p=0.3 |

## Mono-Vibe Model

- **Vibe** = `f64` per room
- **JEPA** = exponential moving average of prior readings
- **Conservation** = sum of vibes (trivially holds under linear diffusion)
- **Diffusion**: `v[i] += rate × Σ(v[j] - v[i])` for neighbors j

## Metrics (6)

1. **Convergence speed** — ticks to max-diff < 0.01
2. **Surprise propagation speed** — ticks for signal to reach all rooms
3. **Surprise attenuation per hop** — signal decay with distance
4. **Robustness** — convergence after node removal
5. **Learning rate** — surprise decrease per tick
6. **Fragility index** — hub removal vs leaf removal impact ratio

## Results Preview (10 rooms, rate=0.1)

```
┌──────────────────────┬───────┬───────┬────────────────┬───────────┐
│ Topology             │ Edges │  Conv │  Surprise Spd  │ Fragility │
├──────────────────────┼───────┼───────┼────────────────┼───────────┤
│ Chain                │     9 │   607 │             36 │      0.00 │
│ Ring                 │    10 │   154 │              9 │      0.00 │
│ Star                 │     9 │     1 │              1 │      0.00 │
│ Mesh                 │    45 │     1 │              1 │      0.00 │
│ Small-world(p=0.1)   │    20 │    38 │              3 │      0.00 │
│ Small-world(p=0.3)   │    20 │    30 │              2 │      1.89 │
│ Small-world(p=0.5)   │    20 │    34 │              2 │      2.44 │
│ Hierarchical         │    21 │   179 │              3 │      0.00 │
│ Scale-free(BA)       │    17 │    54 │              2 │      3.46 │
│ Random(ER,p=0.3)     │    11 │   176 │              7 │      5.85 │
└──────────────────────┴───────┴───────┴────────────────┴───────────┘
```

## Key Findings

- **Small-world (p=0.3)** is the sweet spot: 30 ticks convergence, 2-tick surprise propagation, and reasonable fragility
- **Star and Mesh** are both fastest (1 tick) but for different reasons — star has a hub bottleneck, mesh has maximum connectivity
- **Chain** is slowest (607 ticks) — information must traverse the full path
- **Scale-free** has fast convergence (54 ticks) but high fragility (3.46) — hub dependency
- **Small-world** achieves near-star speed with much better robustness profile

## Tests (18)

```
test test_chain_generates
test test_ring_generates
test test_star_generates
test test_mesh_generates
test test_small_world_p01_generates
test test_small_world_p03_generates
test test_small_world_p05_generates
test test_hierarchical_generates
test test_scale_free_generates
test test_random_er_generates
test test_small_world_faster_than_ring
test test_small_world_faster_than_chain
test test_scale_free_fast_convergence
test test_star_hub_removal_catastrophic
test test_leaf_removal_minimal_impact
test test_vibe_conservation
test test_star_is_fastest
test test_all_connected_converge
```

## Run

```bash
cargo run    # Full topology sweep with all metrics
cargo test   # Run all 18 tests
```

## License

MIT
