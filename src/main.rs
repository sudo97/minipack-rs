mod swc_solution;
use anyhow::{Context, Result};
use std::{collections::VecDeque, fmt::Display};
use swc_solution::{create_asset, Asset};

fn run_for_path(path: &str) -> Result<()> {
    let res = create_graph(path)?;
    for asset in res {
        println!("{:}", asset);
        println!();
    }
    Ok(())
}

#[derive(Debug)]
struct GraphError;

impl Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GraphError")
    }
}

impl std::error::Error for GraphError {}

fn create_graph(path: &str) -> Result<Vec<Asset>> {
    let mut id = 0;
    let main_asset = create_asset(path, id)?;

    id += 1;

    let mut result = Vec::new();

    let mut queue = VecDeque::new();
    queue.push_back(main_asset.clone());

    while let Some(mut asset) = queue.pop_front() {
        let dirname = std::path::Path::new(&asset.filename)
            .parent()
            .ok_or(GraphError)
            .context(format!("Failed to find a dirname of {}", asset.filename))?;

        for dep in &asset.dependencies {
            let dep_path = dirname.join(dep);

            let absolute_path = dep_path
                .to_str()
                .ok_or(GraphError)
                .context(format!("Failed to find a dirname of {}", asset.filename))?;

            let child = create_asset(absolute_path, id)?;

            asset.mapping.insert(dep.clone(), child.id);

            id += 1;

            queue.push_back(child.clone());
        }
        result.push(asset);
    }

    Ok(result)
}

fn main() {
    let mut args = std::env::args();
    match args.nth(1) {
        Some(arg) => run_for_path(&arg).unwrap(),
        None => println!("Please provide a path argument"),
    }
}
