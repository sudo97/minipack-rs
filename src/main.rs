mod swc_solution;
use anyhow::{Context, Result};
use std::{collections::VecDeque, fmt::Display};
use swc_solution::{create_asset, Asset};

fn run_for_path(path: &str) -> Result<()> {
    let res = create_graph(path)?;
    let bundle = bundle(res)?;
    println!("{}", bundle);
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

    let mut visited: Vec<Asset> = vec![main_asset.clone()];

    let mut queue = VecDeque::new();
    queue.push_back(main_asset.clone());

    let mut result: Vec<Asset> = vec![];

    while let Some(mut asset) = queue.pop_front() {
        let dirname = std::path::Path::new(&asset.filename)
            .parent()
            .ok_or(GraphError)
            .context(format!("Failed to find a dirname of {}", asset.filename))?;

        for dep in &asset.dependencies {
            let dep_path = dirname.join(dep).canonicalize()?;

            let absolute_path = dep_path
                .to_str()
                .ok_or(GraphError)
                .context(format!("Failed to find a dirname of {}", asset.filename))?;

            if let Some(child) = visited.iter().find(|a| a.filename == absolute_path) {
                asset.mapping.insert(dep.clone(), child.id);
            } else {
                let child = create_asset(absolute_path, id)?;

                asset.mapping.insert(dep.clone(), child.id);

                id += 1;

                queue.push_back(child.clone());
                visited.push(child);
            }
        }
        result.push(asset);
    }

    Ok(result)
}

fn bundle(graph: Vec<Asset>) -> Result<String> {
    let mut result = String::new();

    for asset in graph {
        let code = format!(
            "{}:  [
            function (require, module, exports) {{
                {}
            }},
            {:?}
        ],
        ",
            asset.id, asset.code, asset.mapping
        );
        result.push_str(&code);
    }

    Ok(format!(
        "(function(modules) {{
            function require(id) {{
                const [fn, mapping] = modules[id];
                
                function localRequire(name) {{
                    return require(mapping[name]);
                }}
                
                const module = {{ exports : {{}} }};
                
                fn(localRequire, module, module.exports);
                
                return module.exports;
            }}
            
            require(0);
        }})({{ {} }})",
        result
    ))
}
fn main() -> Result<()> {
    let mut args = std::env::args();
    let path = args
        .nth(1)
        .ok_or(GraphError)
        .context("Please provide a path argument")?;
    run_for_path(&path)?;
    Ok(())
}
