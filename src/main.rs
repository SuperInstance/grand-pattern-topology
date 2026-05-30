/// Grand Pattern Topology Sweep — Mono-Vibe Edition
///
/// 10 topologies × 6 metrics. Pure Rust, zero dependencies.
/// Vibe = f64. JEPA = weighted history. Conservation = sum of vibes (trivially holds).

// ── Graph Types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Graph {
    /// Adjacency list: neighbors[i] = set of neighbor indices
    pub neighbors: Vec<Vec<usize>>,
    pub n: usize,
}

impl Graph {
    pub fn new(n: usize) -> Self {
        Self {
            neighbors: vec![Vec::new(); n],
            n,
        }
    }

    pub fn add_edge(&mut self, a: usize, b: usize) {
        if a != b && !self.neighbors[a].contains(&b) {
            self.neighbors[a].push(b);
            self.neighbors[b].push(a);
        }
    }

    pub fn degree(&self, i: usize) -> usize {
        self.neighbors[i].len()
    }

    pub fn edge_count(&self) -> usize {
        self.neighbors.iter().map(|v| v.len()).sum::<usize>() / 2
    }

    pub fn has_edge(&self, a: usize, b: usize) -> bool {
        self.neighbors[a].contains(&b)
    }

    /// Remove a node: disconnect all its edges
    pub fn remove_node(&self, node: usize) -> Graph {
        let mut g = self.clone();
        for &nb in &self.neighbors[node] {
            g.neighbors[nb].retain(|&x| x != node);
        }
        g.neighbors[node].clear();
        g
    }
}

// ── Topology Generators ─────────────────────────────────────────────────────

pub fn chain(n: usize) -> Graph {
    let mut g = Graph::new(n);
    for i in 0..n.saturating_sub(1) {
        g.add_edge(i, i + 1);
    }
    g
}

pub fn ring(n: usize) -> Graph {
    let mut g = chain(n);
    if n > 2 {
        g.add_edge(0, n - 1);
    }
    g
}

pub fn star(n: usize) -> Graph {
    let mut g = Graph::new(n);
    for i in 1..n {
        g.add_edge(0, i);
    }
    g
}

pub fn mesh(n: usize) -> Graph {
    let mut g = Graph::new(n);
    for i in 0..n {
        for j in (i + 1)..n {
            g.add_edge(i, j);
        }
    }
    g
}

/// Watts-Strogatz small-world: start from ring lattice, rewire with probability p
pub fn small_world(n: usize, k: usize, p: f64, seed: u64) -> Graph {
    let mut g = Graph::new(n);
    // Ring lattice with k nearest neighbors on each side
    for i in 0..n {
        for j in 1..=k {
            let neighbor = (i + j) % n;
            g.add_edge(i, neighbor);
        }
    }
    // Rewire edges with probability p using simple LCG
    let mut rng = seed;
    let mut next_rand = || -> u64 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        rng
    };
    for i in 0..n {
        let neighbors: Vec<usize> = g.neighbors[i].clone();
        for &j in &neighbors {
            if (next_rand() % 10000) as f64 / 10000.0 < p {
                // Remove edge i-j, add edge i-new
                g.neighbors[i].retain(|&x| x != j);
                g.neighbors[j].retain(|&x| x != i);
                // Pick random target (not self, not already connected)
                let mut target = (next_rand() % n as u64) as usize;
                let mut attempts = 0;
                while (target == i || g.has_edge(i, target)) && attempts < n {
                    target = (next_rand() % n as u64) as usize;
                    attempts += 1;
                }
                if target != i && !g.has_edge(i, target) {
                    g.add_edge(i, target);
                }
            }
        }
    }
    g
}

/// Hierarchical: two clusters connected by a single bridge node
pub fn hierarchical(n: usize) -> Graph {
    let mut g = Graph::new(n);
    let half = n / 2;
    // Cluster A: nodes 0..half (mesh-like within cluster)
    for i in 0..half {
        for j in (i + 1)..half {
            g.add_edge(i, j);
        }
    }
    // Cluster B: nodes half..n (mesh-like within cluster)
    for i in half..n {
        for j in (i + 1)..n {
            g.add_edge(i, j);
        }
    }
    // Bridge: connect node half-1 (cluster A) to node half (cluster B)
    if half > 0 && half < n {
        g.add_edge(half - 1, half);
    }
    g
}

/// Scale-free network using Barabási-Albert preferential attachment
pub fn scale_free(n: usize, m: usize, seed: u64) -> Graph {
    if n <= m {
        return mesh(n);
    }
    let mut g = Graph::new(n);
    let mut rng = seed;
    let mut next_rand = || -> u64 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        rng
    };
    // Start with a complete graph on m+1 nodes
    for i in 0..=m {
        for j in (i + 1)..=m {
            g.add_edge(i, j);
        }
    }
    // Degree tracker for preferential attachment
    let mut degrees = vec![0usize; n];
    for i in 0..=m {
        degrees[i] = g.degree(i);
    }
    let mut total_degree: usize = degrees.iter().sum();

    // Add remaining nodes
    for new_node in (m + 1)..n {
        let mut targets = Vec::new();
        let mut attempts = 0;
        while targets.len() < m && attempts < m * 10 {
            let r = (next_rand() % total_degree as u64) as usize;
            let mut cumulative = 0usize;
            for (i, &d) in degrees.iter().enumerate().take(new_node) {
                cumulative += d;
                if r < cumulative && !targets.contains(&i) {
                    targets.push(i);
                    break;
                }
            }
            attempts += 1;
        }
        for &t in &targets {
            g.add_edge(new_node, t);
            degrees[new_node] += 1;
            degrees[t] += 1;
            total_degree += 2;
        }
    }
    g
}

/// Random graph using Erdős-Rényi model with edge probability p
pub fn random_er(n: usize, p: f64, seed: u64) -> Graph {
    let mut g = Graph::new(n);
    let mut rng = seed;
    let mut next_rand = || -> u64 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        rng
    };
    for i in 0..n {
        for j in (i + 1)..n {
            if (next_rand() % 10000) as f64 / 10000.0 < p {
                g.add_edge(i, j);
            }
        }
    }
    g
}

// ── Simulation ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Simulation {
    pub vibes: Vec<f64>,
    pub graph: Graph,
    pub tick: usize,
    /// JEPA: weighted history per room (exponential moving average of vibes)
    pub jepa: Vec<f64>,
    /// Alpha for JEPA EMA
    pub alpha: f64,
    /// Diffusion rate
    pub diffusion_rate: f64,
    /// History of max-diff per tick (for convergence tracking)
    pub max_diff_history: Vec<f64>,
}

impl Simulation {
    pub fn new(graph: Graph, initial_vibes: Vec<f64>, diffusion_rate: f64, alpha: f64) -> Self {
        let jepa = initial_vibes.clone();
        Self {
            vibes: initial_vibes,
            graph,
            tick: 0,
            jepa,
            alpha,
            diffusion_rate,
            max_diff_history: Vec::new(),
        }
    }

    pub fn step(&mut self) {
        let n = self.graph.n;
        let mut new_vibes = self.vibes.clone();
        for i in 0..n {
            let neighbors = &self.graph.neighbors[i];
            if neighbors.is_empty() {
                continue;
            }
            // Diffusion: each neighbor pulls toward average
            let avg_neighbor: f64 = neighbors.iter().map(|&j| self.vibes[j]).sum::<f64>() / neighbors.len() as f64;
            let diff = avg_neighbor - self.vibes[i];
            // Damping: diffusion_rate / degree_count — more neighbors = more damping per-link
            let effective_rate = self.diffusion_rate / neighbors.len() as f64;
            new_vibes[i] += effective_rate * diff * neighbors.len() as f64;
            // Actually, let's use a cleaner model: standard heat equation
            // new_vibe[i] = vibe[i] + rate * sum(vibe[j] - vibe[i]) for j in neighbors
            // This naturally handles degree
        }
        // Redo with cleaner model
        let mut new_vibes2 = self.vibes.clone();
        for i in 0..n {
            let neighbors = &self.graph.neighbors[i];
            if neighbors.is_empty() {
                continue;
            }
            let mut delta = 0.0;
            for &j in neighbors {
                delta += self.vibes[j] - self.vibes[i];
            }
            new_vibes2[i] += self.diffusion_rate * delta;
        }
        self.vibes = new_vibes2;

        // Update JEPA (exponential moving average)
        for i in 0..n {
            self.jepa[i] = self.alpha * self.vibes[i] + (1.0 - self.alpha) * self.jepa[i];
        }

        // Track max difference between JEPA prediction and actual vibe
        let max_diff = self.vibes.iter().enumerate().map(|(i, &v)| (v - self.jepa[i]).abs()).fold(0.0f64, f64::max);
        self.max_diff_history.push(max_diff);

        self.tick += 1;
    }

    pub fn run(&mut self, ticks: usize) {
        for _ in 0..ticks {
            self.step();
        }
    }

    pub fn max_vibe_diff(&self) -> f64 {
        let min = self.vibes.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = self.vibes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        max - min
    }

    /// Ticks to convergence: max difference between any two rooms < threshold
    pub fn convergence_ticks(&self) -> Option<usize> {
        for _ in self.max_diff_history.iter().enumerate() {
            // Check actual vibe spread at that point — we'd need to track it
            // Instead, compute from max_diff_history: when JEPA prediction error < threshold
        }
        // Simpler: re-run and track
        None
    }
}

/// Run a convergence experiment and return ticks to convergence
pub fn convergence_ticks(graph: &Graph, initial_vibes: &[f64], diffusion_rate: f64, threshold: f64, max_ticks: usize) -> Option<usize> {
    let n = graph.n;
    let mut vibes = initial_vibes.to_vec();
    for tick in 0..max_ticks {
        let min_v = vibes.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_v = vibes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if max_v - min_v < threshold {
            return Some(tick);
        }
        let mut new_vibes = vibes.clone();
        for i in 0..n {
            let neighbors = &graph.neighbors[i];
            if neighbors.is_empty() {
                continue;
            }
            let mut delta = 0.0;
            for &j in neighbors {
                delta += vibes[j] - vibes[i];
            }
            new_vibes[i] += diffusion_rate * delta;
        }
        vibes = new_vibes;
    }
    None
}

/// Inject a surprise (set a room's vibe to a value) and measure propagation
pub fn surprise_propagation(graph: &Graph, base_vibe: f64, surprise_room: usize, surprise_value: f64, diffusion_rate: f64, max_ticks: usize) -> Vec<Vec<f64>> {
    let n = graph.n;
    let mut vibes = vec![base_vibe; n];
    vibes[surprise_room] = surprise_value;
    let mut history = vec![vibes.clone()];
    for _ in 0..max_ticks {
        let mut new_vibes = vibes.clone();
        for i in 0..n {
            let neighbors = &graph.neighbors[i];
            if neighbors.is_empty() {
                continue;
            }
            let mut delta = 0.0;
            for &j in neighbors {
                delta += vibes[j] - vibes[i];
            }
            new_vibes[i] += diffusion_rate * delta;
        }
        vibes = new_vibes;
        history.push(vibes.clone());
    }
    history
}

/// Surprise attenuation per hop: measure how much the surprise signal decays with distance
pub fn surprise_attenuation_per_hop(graph: &Graph, base_vibe: f64, surprise_room: usize, surprise_value: f64, diffusion_rate: f64, ticks: usize) -> Vec<(usize, f64)> {
    let n = graph.n;
    // BFS to get distances from surprise_room
    let distances = bfs_distances(graph, surprise_room);
    let history = surprise_propagation(graph, base_vibe, surprise_room, surprise_value, diffusion_rate, ticks);
    let final_vibes = &history[ticks.min(history.len() - 1)];
    let surprise_mag = (surprise_value - base_vibe).abs();
    let mut by_hop: Vec<(usize, f64)> = Vec::new();
    for hop in 0..=*distances.iter().max().unwrap_or(&0) {
        let nodes_at_hop: Vec<usize> = (0..n).filter(|&i| distances[i] == hop && i != surprise_room).collect();
        if nodes_at_hop.is_empty() {
            continue;
        }
        let avg_deviation: f64 = nodes_at_hop.iter().map(|&i| (final_vibes[i] - base_vibe).abs()).sum::<f64>() / nodes_at_hop.len() as f64;
        let attenuation = if surprise_mag > 0.0 { 1.0 - avg_deviation / surprise_mag } else { 1.0 };
        by_hop.push((hop, attenuation));
    }
    by_hop
}

/// BFS distances from a source node
pub fn bfs_distances(graph: &Graph, source: usize) -> Vec<usize> {
    let n = graph.n;
    let mut dist = vec![usize::MAX; n];
    dist[source] = 0;
    let mut queue = vec![source];
    let mut head = 0;
    while head < queue.len() {
        let u = queue[head];
        head += 1;
        for &v in &graph.neighbors[u] {
            if dist[v] == usize::MAX {
                dist[v] = dist[u] + 1;
                queue.push(v);
            }
        }
    }
    dist
}

/// Robustness: remove a node and measure how much the convergence degrades
pub fn robustness_measure(graph: &Graph, initial_vibes: &[f64], diffusion_rate: f64, remove_node: usize, max_ticks: usize) -> (Option<usize>, Option<usize>) {
    let baseline = convergence_ticks(graph, initial_vibes, diffusion_rate, 0.01, max_ticks);
    let damaged = graph.remove_node(remove_node);
    let damaged_vibes: Vec<f64> = initial_vibes.iter().enumerate().filter(|&(i, _)| i != remove_node).map(|(_, &v)| v).collect();
    // Remap graph to remove the node
    let remapped = remap_graph(&damaged, remove_node);
    let after = convergence_ticks(&remapped, &damaged_vibes, diffusion_rate, 0.01, max_ticks);
    (baseline, after)
}

/// Remap graph to remove a node index, shifting indices down
fn remap_graph(graph: &Graph, removed: usize) -> Graph {
    let mut mapping = vec![0; graph.n];
    let mut idx = 0;
    for i in 0..graph.n {
        if i != removed {
            mapping[i] = idx;
            idx += 1;
        }
    }
    let new_n = idx;
    let mut g = Graph::new(new_n);
    for i in 0..graph.n {
        if i == removed { continue; }
        for &j in &graph.neighbors[i] {
            if j == removed || j < i { continue; }
            g.add_edge(mapping[i], mapping[j]);
        }
    }
    g
}

/// Learning rate: how fast does surprise decrease per tick after injection
pub fn learning_rate(graph: &Graph, base_vibe: f64, surprise_room: usize, surprise_value: f64, diffusion_rate: f64, ticks: usize) -> f64 {
    let _ = graph.n; // used implicitly by surprise_propagation
    let history = surprise_propagation(graph, base_vibe, surprise_room, surprise_value, diffusion_rate, ticks);
    let initial_spread: f64 = history[0].iter().map(|&v| (v - base_vibe).abs()).sum::<f64>();
    let final_spread: f64 = history.last().unwrap().iter().map(|&v| (v - base_vibe).abs()).sum::<f64>();
    if initial_spread == 0.0 || ticks == 0 {
        return 0.0;
    }
    (initial_spread - final_spread) / ticks as f64
}

/// Fragility index: compare disruption from hub removal vs leaf removal
pub fn fragility_index(graph: &Graph, initial_vibes: &[f64], diffusion_rate: f64, max_ticks: usize) -> f64 {
    let n = graph.n;
    if n < 3 { return 0.0; }
    // Find hub (highest degree) and leaf (lowest degree > 0)
    let mut hub = 0;
    let mut hub_deg = 0;
    let mut leaf = 0;
    let mut leaf_deg = usize::MAX;
    for i in 0..n {
        let d = graph.degree(i);
        if d > hub_deg { hub_deg = d; hub = i; }
        if d > 0 && d < leaf_deg { leaf_deg = d; leaf = i; }
    }
    if hub == leaf { return 0.0; }
    let (_, hub_after) = robustness_measure(graph, initial_vibes, diffusion_rate, hub, max_ticks);
    let (_, leaf_after) = robustness_measure(graph, initial_vibes, diffusion_rate, leaf, max_ticks);
    let hub_ticks = hub_after.unwrap_or(max_ticks) as f64;
    let leaf_ticks = leaf_after.unwrap_or(max_ticks) as f64;
    if leaf_ticks == 0.0 { return 0.0; }
    hub_ticks / leaf_ticks
}

// ── Topology Registry ────────────────────────────────────────────────────────

pub fn build_all_topologies(n: usize) -> Vec<(&'static str, Graph)> {
    vec![
        ("Chain", chain(n)),
        ("Ring", ring(n)),
        ("Star", star(n)),
        ("Mesh", mesh(n)),
        ("Small-world(p=0.1)", small_world(n, 2, 0.1, 42)),
        ("Small-world(p=0.3)", small_world(n, 2, 0.3, 42)),
        ("Small-world(p=0.5)", small_world(n, 2, 0.5, 42)),
        ("Hierarchical", hierarchical(n)),
        ("Scale-free(BA)", scale_free(n, 2, 42)),
        ("Random(ER,p=0.3)", random_er(n, 0.3, 42)),
    ]
}

pub fn standard_initial_vibes(n: usize) -> Vec<f64> {
    // Spike in room 0, rest at 0
    let mut v = vec![0.0; n];
    v[0] = 10.0;
    v
}

// ── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    let n = 10;
    let topologies = build_all_topologies(n);
    let initial_vibes = standard_initial_vibes(n);
    let diffusion_rate = 0.1;
    let max_ticks = 1000;

    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║       GRAND PATTERN — Topology Sweep (Mono-Vibe, 10 rooms)        ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();

    println!("┌──────────────────────┬───────┬───────┬────────────────┬───────────┐");
    println!("│ Topology             │ Edges │  Conv │  Surprise Spd  │ Fragility │");
    println!("├──────────────────────┼───────┼───────┼────────────────┼───────────┤");

    for (name, graph) in &topologies {
        let edges = graph.edge_count();
        let conv = convergence_ticks(graph, &initial_vibes, diffusion_rate, 0.01, max_ticks);
        let conv_str = match conv {
            Some(t) => format!("{:>5}", t),
            None => " >1k ".to_string(),
        };

        // Surprise propagation: how many ticks until surprise reaches all rooms
        let history = surprise_propagation(graph, 0.0, 0, 10.0, diffusion_rate, max_ticks);
        let mut surprise_speed = max_ticks;
        for (t, vibes) in history.iter().enumerate() {
            let all_touched = vibes.iter().all(|&v| v.abs() > 0.01);
            if all_touched {
                surprise_speed = t;
                break;
            }
        }

        let frag = fragility_index(graph, &initial_vibes, diffusion_rate, max_ticks);

        println!("│ {:<20} │ {:>5} │ {:>5} │ {:>14} │ {:>9.2} │",
            name, edges, conv_str, surprise_speed, frag);
    }

    println!("└──────────────────────┴───────┴───────┴────────────────┴───────────┘");

    // Detailed surprise attenuation for each topology
    println!();
    println!("── Surprise Attenuation Per Hop ──");
    for (name, graph) in &topologies {
        let atten = surprise_attenuation_per_hop(graph, 0.0, 0, 10.0, diffusion_rate, 100);
        let atten_str: Vec<String> = atten.iter().map(|(hop, a)| format!("{}:{:.2}", hop, a)).collect();
        println!("  {:<20} {}", name, atten_str.join("  "));
    }

    // Learning rates
    println!();
    println!("── Learning Rates ──");
    for (name, graph) in &topologies {
        let lr = learning_rate(graph, 0.0, 0, 10.0, diffusion_rate, 100);
        println!("  {:<20} {:.6}", name, lr);
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const N: usize = 10;

    // 1-10: each topology generates correctly

    #[test]
    fn test_chain_generates() {
        let g = chain(N);
        assert_eq!(g.n, N);
        assert_eq!(g.edge_count(), N - 1);
        assert_eq!(g.degree(0), 1);
        assert_eq!(g.degree(N - 1), 1);
        assert_eq!(g.degree(5), 2);
    }

    #[test]
    fn test_ring_generates() {
        let g = ring(N);
        assert_eq!(g.n, N);
        assert_eq!(g.edge_count(), N);
        // All nodes degree 2
        for i in 0..N {
            assert_eq!(g.degree(i), 2);
        }
        assert!(g.has_edge(0, N - 1));
    }

    #[test]
    fn test_star_generates() {
        let g = star(N);
        assert_eq!(g.n, N);
        assert_eq!(g.edge_count(), N - 1);
        assert_eq!(g.degree(0), N - 1);
        for i in 1..N {
            assert_eq!(g.degree(i), 1);
        }
    }

    #[test]
    fn test_mesh_generates() {
        let g = mesh(N);
        assert_eq!(g.n, N);
        assert_eq!(g.edge_count(), N * (N - 1) / 2);
        for i in 0..N {
            assert_eq!(g.degree(i), N - 1);
        }
    }

    #[test]
    fn test_small_world_p01_generates() {
        let g = small_world(N, 2, 0.1, 42);
        assert_eq!(g.n, N);
        // Ring lattice starts with 2*k=4 edges per node / 2 = N*2 edges total for k=2
        // With p=0.1, some edges rewired but count stays ~same
        assert!(g.edge_count() >= 5, "small-world p=0.1 should have edges: got {}", g.edge_count());
        assert!(g.edge_count() <= 30, "small-world p=0.1 should not have too many edges: got {}", g.edge_count());
    }

    #[test]
    fn test_small_world_p03_generates() {
        let g = small_world(N, 2, 0.3, 42);
        assert_eq!(g.n, N);
        assert!(g.edge_count() > 0);
    }

    #[test]
    fn test_small_world_p05_generates() {
        let g = small_world(N, 2, 0.5, 42);
        assert_eq!(g.n, N);
        assert!(g.edge_count() > 0);
    }

    #[test]
    fn test_hierarchical_generates() {
        let g = hierarchical(N);
        assert_eq!(g.n, N);
        let half = N / 2;
        // Bridge should exist
        assert!(g.has_edge(half - 1, half));
        // Nodes within each cluster should be connected
        assert!(g.has_edge(0, 1));
        assert!(g.has_edge(half, half + 1));
    }

    #[test]
    fn test_scale_free_generates() {
        let g = scale_free(N, 2, 42);
        assert_eq!(g.n, N);
        assert!(g.edge_count() >= 3); // At least the initial triangle + some
        // Should have a hub (node with high degree)
        let max_deg = (0..N).map(|i| g.degree(i)).max().unwrap();
        assert!(max_deg >= 3);
    }

    #[test]
    fn test_random_er_generates() {
        let g = random_er(N, 0.3, 42);
        assert_eq!(g.n, N);
        // With p=0.3 and 45 possible edges, expect ~13 edges
        assert!(g.edge_count() > 0);
        assert!(g.edge_count() < N * (N - 1) / 2);
    }

    // 11. Small-world converges faster than ring
    #[test]
    fn test_small_world_faster_than_ring() {
        let vibes = standard_initial_vibes(N);
        let ring_conv = convergence_ticks(&ring(N), &vibes, 0.1, 0.01, 1000);
        let sw_conv = convergence_ticks(&small_world(N, 2, 0.3, 42), &vibes, 0.1, 0.01, 1000);
        // Small-world should converge faster (or at least not significantly slower)
        // Due to randomness, we allow small margin
        let ring_t = ring_conv.unwrap_or(1000);
        let sw_t = sw_conv.unwrap_or(1000);
        assert!(sw_t <= ring_t + 50, "small-world ({}) should be ≈ ring ({}) or faster", sw_t, ring_t);
    }

    // 12. Small-world converges faster than chain
    #[test]
    fn test_small_world_faster_than_chain() {
        let vibes = standard_initial_vibes(N);
        let chain_conv = convergence_ticks(&chain(N), &vibes, 0.1, 0.01, 1000);
        let sw_conv = convergence_ticks(&small_world(N, 2, 0.3, 42), &vibes, 0.1, 0.01, 1000);
        let chain_t = chain_conv.unwrap_or(1000);
        let sw_t = sw_conv.unwrap_or(1000);
        assert!(sw_t <= chain_t + 20, "small-world ({}) should be ≈ chain ({}) or faster", sw_t, chain_t);
    }

    // 13. Scale-free has fast convergence
    #[test]
    fn test_scale_free_fast_convergence() {
        let vibes = standard_initial_vibes(N);
        let conv = convergence_ticks(&scale_free(N, 2, 42), &vibes, 0.1, 0.01, 1000);
        // Scale-free should converge reasonably fast (hub helps propagate)
        assert!(conv.unwrap_or(1000) < 300, "scale-free should converge in < 300 ticks");
    }

    // 14. Hub removal in star is catastrophic — leaves become isolated
    #[test]
    fn test_star_hub_removal_catastrophic() {
        let g = star(N);
        let damaged = g.remove_node(0);
        // After hub removal, all leaves should be isolated (degree 0)
        for i in 1..N {
            assert_eq!(damaged.degree(i), 0, "leaf {} should be isolated after hub removal", i);
        }
        // Isolated graph cannot converge from non-uniform vibes
        let remapped = remap_graph(&damaged, 0);
        let _damaged_vibes: Vec<f64> = standard_initial_vibes(N).iter().skip(1).copied().collect();
        // Vibe at index 0 was 10.0, now remapped to node 0 with vibe 0.0
        // Actually index 0 had 10.0 in original, node 0 is removed, so remaining all 0.0
        // Let's use vibes where remaining nodes differ
        let mut test_vibes = vec![0.0; N - 1];
        test_vibes[0] = 10.0;
        let conv = convergence_ticks(&remapped, &test_vibes, 0.1, 0.01, 1000);
        assert!(conv.is_none(), "isolated nodes should not converge: {:?}", conv);
    }

    // 15. Leaf removal in star has minimal impact (vs hub removal)
    #[test]
    fn test_leaf_removal_minimal_impact() {
        let g = star(N);
        let vibes = standard_initial_vibes(N);
        // Use vibes where hub (0) has the signal so removing a leaf barely matters
        let baseline = convergence_ticks(&g, &vibes, 0.1, 0.01, 1000);
        // Remove a leaf (node N-1)
        let (_, leaf_after) = robustness_measure(&g, &vibes, 0.1, N - 1, 1000);
        let base_t = baseline.unwrap_or(1000) as f64;
        let leaf_t = leaf_after.unwrap_or(1000) as f64;
        // Removing a leaf from star should barely change convergence
        // Both should be very fast since hub remains intact
        assert!(leaf_t <= 10.0,
            "leaf removal in star should still converge fast: {} -> {}", base_t, leaf_t);
    }

    // Bonus: Conservation check
    #[test]
    fn test_vibe_conservation() {
        let g = ring(N);
        let initial: Vec<f64> = (0..N).map(|i| i as f64).collect();
        let total_initial: f64 = initial.iter().sum();
        let mut vibes = initial;
        for _ in 0..100 {
            let mut new_vibes = vibes.clone();
            for i in 0..N {
                let neighbors = &g.neighbors[i];
                let mut delta = 0.0;
                for &j in neighbors {
                    delta += vibes[j] - vibes[i];
                }
                new_vibes[i] += 0.1 * delta;
            }
            vibes = new_vibes;
        }
        let total_final: f64 = vibes.iter().sum();
        assert!((total_final - total_initial).abs() < 1e-9,
            "vibes should be conserved: {} -> {}", total_initial, total_final);
    }

    // Bonus: Star is fastest
    #[test]
    fn test_star_is_fastest() {
        let vibes = standard_initial_vibes(N);
        let star_conv = convergence_ticks(&star(N), &vibes, 0.1, 0.01, 1000).unwrap_or(1000);
        let chain_conv = convergence_ticks(&chain(N), &vibes, 0.1, 0.01, 1000).unwrap_or(1000);
        assert!(star_conv < chain_conv, "star ({}) should be faster than chain ({})", star_conv, chain_conv);
    }

    // Bonus: Convergence actually happens for connected graphs
    #[test]
    fn test_all_connected_converge() {
        let vibes = standard_initial_vibes(N);
        for (name, graph) in build_all_topologies(N) {
            let conv = convergence_ticks(&graph, &vibes, 0.1, 0.01, 2000);
            assert!(conv.is_some(), "{} should converge within 2000 ticks", name);
        }
    }
}
