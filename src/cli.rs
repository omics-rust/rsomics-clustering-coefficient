use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use serde::Serialize;

use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta, run};

use rsomics_clustering_coefficient::{cluster, io};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

/// Which clustering metric to compute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Metric {
    /// Per-node triangle count (default).
    Triangles,
    /// Per-node local clustering coefficient.
    Local,
    /// Average clustering coefficient (scalar).
    Average,
    /// Global transitivity (scalar).
    Transitivity,
}

/// Triangle-based clustering metrics for undirected graphs.
///
/// Reads an edge list (one `u v` pair per line, whitespace-separated) from a
/// file argument or stdin (`-`). Comment lines starting with `#` and blank
/// lines are ignored. Self-loops are silently dropped. Duplicate edges collapse
/// to a simple graph.
///
/// Output order matches networkx node-insertion order (the order each node
/// label first appears in the edge list).
#[derive(Parser, Debug)]
#[command(name = "rsomics-clustering-coefficient", version, about, long_about = None)]
pub struct Cli {
    /// Metric to compute.
    #[arg(long = "metric", value_enum, default_value = "triangles")]
    pub metric: Metric,

    /// Edge list file (`-` or omitted reads stdin).
    #[arg(value_name = "EDGELIST")]
    pub edgelist: Option<PathBuf>,

    #[command(flatten)]
    pub common: CommonFlags,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Out {
    PerNode {
        metric: &'static str,
        nodes: Vec<String>,
        values: Vec<serde_json::Value>,
    },
    Scalar {
        metric: &'static str,
        value: f64,
    },
}

impl Cli {
    pub fn run(self) -> ExitCode {
        let common = self.common.clone();
        run(&common, META, || self.execute(&common))
    }

    fn execute(self, common: &CommonFlags) -> Result<Out> {
        let path = self.edgelist.as_deref();
        let g = io::read_edgelist(path)?;

        match self.metric {
            Metric::Triangles => {
                let t = cluster::triangles(&g);
                if !common.json {
                    let stdout = std::io::stdout().lock();
                    let mut w = BufWriter::new(stdout);
                    for (i, &count) in t.iter().enumerate() {
                        writeln!(w, "{}\t{count}", g.labels[i]).map_err(RsomicsError::Io)?;
                    }
                    w.flush().map_err(RsomicsError::Io)?;
                }
                Ok(Out::PerNode {
                    metric: "triangles",
                    nodes: g.labels,
                    values: t.iter().map(|&v| serde_json::Value::from(v)).collect(),
                })
            }
            Metric::Local => {
                let c = cluster::local_clustering(&g);
                if !common.json {
                    let stdout = std::io::stdout().lock();
                    let mut w = BufWriter::new(stdout);
                    for (i, &coef) in c.iter().enumerate() {
                        writeln!(w, "{}\t{coef}", g.labels[i]).map_err(RsomicsError::Io)?;
                    }
                    w.flush().map_err(RsomicsError::Io)?;
                }
                Ok(Out::PerNode {
                    metric: "local",
                    nodes: g.labels,
                    values: c.iter().map(|&v| serde_json::Value::from(v)).collect(),
                })
            }
            Metric::Average => {
                let avg = cluster::average_clustering(&g);
                if !common.json {
                    println!("{avg}");
                }
                Ok(Out::Scalar {
                    metric: "average",
                    value: avg,
                })
            }
            Metric::Transitivity => {
                let t = cluster::transitivity(&g);
                if !common.json {
                    println!("{t}");
                }
                Ok(Out::Scalar {
                    metric: "transitivity",
                    value: t,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        super::Cli::command().debug_assert();
    }
}
