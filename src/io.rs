use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

/// Adjacency structure using integer node IDs. Nodes are assigned IDs in
/// first-seen insertion order from the edge list.
pub struct Graph {
    pub labels: Vec<String>,
    /// adj[u] = sorted list of neighbour IDs (simple graph, no self-loops)
    pub adj: Vec<Vec<u32>>,
}

impl Graph {
    pub fn n(&self) -> usize {
        self.labels.len()
    }
}

/// Parse an undirected edge list. Rules matching networkx `read_edgelist`:
/// - Text from the first `#` onward is a comment (stripped before tokenising),
///   so a `#` anywhere on a line ends the data, not just a leading one.
/// - Blank lines, and lines that are entirely comment, are ignored.
/// - Each data line must have at least two whitespace-separated tokens; the
///   first two are the endpoints.
/// - Self-loops register the node but add no edge: networkx excludes a
///   self-loop from both triangle counting and the clustering degree
///   denominator, yet still keeps the node (degree 0, clustering 0).
/// - Duplicate edges collapse to a simple graph.
///
/// Nodes are assigned integer IDs in the order they first appear.
pub fn read_edgelist(path: Option<&Path>) -> Result<Graph> {
    let reader: Box<dyn BufRead> = match path {
        None => Box::new(BufReader::new(std::io::stdin())),
        Some(p) if p == Path::new("-") => Box::new(BufReader::new(std::io::stdin())),
        Some(p) => Box::new(BufReader::new(File::open(p).map_err(|e| {
            RsomicsError::Io(std::io::Error::new(
                e.kind(),
                format!("{}: {e}", p.display()),
            ))
        })?)),
    };

    let mut labels: Vec<String> = Vec::new();
    let mut index: HashMap<String, u32> = HashMap::new();
    // Collect raw edge pairs before building adjacency (need full node count).
    let mut raw_edges: Vec<(u32, u32)> = Vec::new();

    for (lineno, line) in reader.lines().enumerate() {
        let lineno = lineno + 1;
        let line = line.map_err(RsomicsError::Io)?;
        // networkx parse_edgelist strips a `#` comment anywhere in the line
        // before tokenising: `1 2#note` is edge (1,2), and `0 #1` is a lone
        // token, not an edge to a node named `#1`.
        let t = line.split('#').next().unwrap_or("").trim();
        if t.is_empty() {
            continue;
        }
        let mut tokens = t.split_ascii_whitespace();
        let u_str = tokens.next().unwrap(); // non-empty, guaranteed by trim check
        let v_str = tokens.next().ok_or_else(|| {
            RsomicsError::InvalidInput(format!("line {lineno}: expected two node labels, got one"))
        })?;

        let u = intern(&mut labels, &mut index, u_str);
        let v = intern(&mut labels, &mut index, v_str);
        if u == v {
            continue; // self-loop: node stays registered, no edge added
        }
        raw_edges.push((u, v));
    }

    let n = labels.len();
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (u, v) in raw_edges {
        adj[u as usize].push(v);
        adj[v as usize].push(u);
    }
    // Deduplicate (simple graph) and sort for fast set intersection.
    for nbrs in &mut adj {
        nbrs.sort_unstable();
        nbrs.dedup();
    }

    Ok(Graph { labels, adj })
}

fn intern(labels: &mut Vec<String>, index: &mut HashMap<String, u32>, s: &str) -> u32 {
    if let Some(&id) = index.get(s) {
        return id;
    }
    let id = labels.len() as u32;
    labels.push(s.to_owned());
    index.insert(s.to_owned(), id);
    id
}
