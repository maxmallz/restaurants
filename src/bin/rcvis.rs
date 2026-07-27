//! rcvis: visualize the module/package tree of Rust source files as PlantUML or Graphviz.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use syn::{Item, ItemMod};

#[derive(Parser)]
#[command(name = "rcvis", about = "Visualize the module tree of Rust source files")]
struct Args {
    /// Rust source files to visualize (each becomes a root package)
    #[arg(required = true)]
    sources: Vec<PathBuf>,

    /// Output format
    #[arg(long, value_enum)]
    format: Format,

    /// Output file
    #[arg(long)]
    out: PathBuf,
}

#[derive(Copy, Clone, ValueEnum)]
enum Format {
    Plantuml,
    Graphviz,
    /// ASCII tree, in the style of `cargo modules structure` / `cargo tree`.
    Tree,
}

struct ModuleNode {
    name: String,
    children: Vec<ModuleNode>,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let mut roots = Vec::new();
    for source in &args.sources {
        match load_module_tree(source) {
            Ok(root) => roots.push(root),
            Err(err) => {
                eprintln!("rcvis: failed to parse {}: {err}", source.display());
                return ExitCode::FAILURE;
            }
        }
    }

    let rendered = match args.format {
        Format::Plantuml => render_plantuml(&roots),
        Format::Graphviz => render_graphviz(&roots),
        Format::Tree => render_tree(&roots),
    };

    if let Err(err) = fs::write(&args.out, rendered) {
        eprintln!("rcvis: failed to write {}: {err}", args.out.display());
        return ExitCode::FAILURE;
    }

    println!("rcvis: wrote {}", args.out.display());
    ExitCode::SUCCESS
}

fn load_module_tree(path: &Path) -> Result<ModuleNode, String> {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("crate")
        .to_string();
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let file = syn::parse_file(&content).map_err(|e| e.to_string())?;
    let children = collect_modules(&file.items, dir);
    Ok(ModuleNode { name, children })
}

fn collect_modules(items: &[Item], dir: &Path) -> Vec<ModuleNode> {
    items
        .iter()
        .filter_map(|item| match item {
            Item::Mod(item_mod) => Some(module_from_item(item_mod, dir)),
            _ => None,
        })
        .collect()
}

/// Resolves `mod foo;` declarations against `foo.rs` or `foo/mod.rs` next to the parent file.
fn module_from_item(item_mod: &ItemMod, dir: &Path) -> ModuleNode {
    let name = item_mod.ident.to_string();

    let children = if let Some((_, items)) = &item_mod.content {
        collect_modules(items, dir)
    } else {
        let flat = dir.join(format!("{name}.rs"));
        let nested = dir.join(&name).join("mod.rs");

        let resolved = if flat.exists() {
            Some((flat, dir.to_path_buf()))
        } else if nested.exists() {
            Some((nested, dir.join(&name)))
        } else {
            None
        };

        resolved
            .and_then(|(file_path, sub_dir)| {
                let content = fs::read_to_string(&file_path).ok()?;
                let file = syn::parse_file(&content).ok()?;
                Some(collect_modules(&file.items, &sub_dir))
            })
            .unwrap_or_default()
    };

    ModuleNode { name, children }
}

fn render_plantuml(roots: &[ModuleNode]) -> String {
    let mut out = String::from("@startuml\n");
    for root in roots {
        render_plantuml_node(root, 0, &mut out);
    }
    out.push_str("@enduml\n");
    out
}

fn render_plantuml_node(node: &ModuleNode, depth: usize, out: &mut String) {
    let indent = "  ".repeat(depth);
    out.push_str(&format!("{indent}package \"{}\" {{\n", node.name));
    for child in &node.children {
        render_plantuml_node(child, depth + 1, out);
    }
    out.push_str(&format!("{indent}}}\n"));
}

fn render_graphviz(roots: &[ModuleNode]) -> String {
    let mut out = String::from("digraph packages {\n  node [shape=box];\n");
    let mut counter = 0usize;
    for root in roots {
        render_graphviz_node(root, None, &mut out, &mut counter);
    }
    out.push_str("}\n");
    out
}

fn render_graphviz_node(
    node: &ModuleNode,
    parent_id: Option<&str>,
    out: &mut String,
    counter: &mut usize,
) -> String {
    *counter += 1;
    let id = format!("n{counter}");
    out.push_str(&format!("  {id} [label=\"{}\"];\n", node.name));
    if let Some(parent_id) = parent_id {
        out.push_str(&format!("  {parent_id} -> {id};\n"));
    }
    for child in &node.children {
        render_graphviz_node(child, Some(&id), out, counter);
    }
    id
}

fn render_tree(roots: &[ModuleNode]) -> String {
    let mut out = String::new();
    for root in roots {
        out.push_str(&format!("crate {}\n", root.name));
        render_tree_children(&root.children, "", &mut out);
    }
    out
}

fn render_tree_children(children: &[ModuleNode], prefix: &str, out: &mut String) {
    let count = children.len();
    for (i, child) in children.iter().enumerate() {
        let is_last = i + 1 == count;
        let branch = if is_last { "└── " } else { "├── " };
        out.push_str(&format!("{prefix}{branch}mod {}\n", child.name));

        let child_prefix = format!("{prefix}{}", if is_last { "    " } else { "│   " });
        render_tree_children(&child.children, &child_prefix, out);
    }
}
