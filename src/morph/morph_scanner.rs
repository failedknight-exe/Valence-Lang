// src/morph/morph_scanner.rs
// 🕵️ MORPH DRAWER 1: THE DETECTOR
// Scans Valence AST for 'bond' and 'use' statements to find all files needed for morphing!

use crate::parser::Node;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidecarAsset {
    pub lang: String,
    pub file_path: PathBuf,
}

pub struct MorphScanner;

impl MorphScanner {
    /// Scans an entire AST recursively and returns a list of all required sidecar files!
    pub fn scan(ast: &[Node]) -> Vec<SidecarAsset> {
        let mut assets = Vec::new();
        Self::scan_nodes(ast, &mut assets);
        assets
    }

    fn scan_nodes(nodes: &[Node], assets: &mut Vec<SidecarAsset>) {
        for node in nodes {
            match node {
                Node::Bond { target, .. } => {
                    let (lang, path) = Self::parse_target(target);
                    let asset = SidecarAsset {
                        lang,
                        file_path: path,
                    };
                    if !assets.contains(&asset) {
                        assets.push(asset);
                    }
                }
                Node::UseModule(path_str) => {
                    let asset = SidecarAsset {
                        lang: "cor".to_string(),
                        file_path: PathBuf::from(path_str),
                    };
                    if !assets.contains(&asset) {
                        assets.push(asset);
                    }
                }
                Node::Block(inner_nodes) => Self::scan_nodes(inner_nodes, assets),
                Node::Circle { body, .. } => Self::scan_nodes(body, assets),
                Node::Check { body, or_checks, else_body, .. } => {
                    Self::scan_nodes(body, assets);
                    for (_, or_body) in or_checks {
                        Self::scan_nodes(or_body, assets);
                    }
                    if let Some(else_nodes) = else_body {
                        Self::scan_nodes(else_nodes, assets);
                    }
                }
                Node::FuncDecl { body, .. } => Self::scan_nodes(body, assets),
                Node::Guard { body, .. } => Self::scan_nodes(body, assets),
                Node::Attempt { body, rescue_body, always_body, .. } => {
                    Self::scan_nodes(body, assets);
                    if let Some(r_body) = rescue_body {
                        Self::scan_nodes(r_body, assets);
                    }
                    if let Some(a_body) = always_body {
                        Self::scan_nodes(a_body, assets);
                    }
                }
                Node::AsyncBlock { body } => Self::scan_nodes(body, assets),
                Node::Protect { body, .. } => Self::scan_nodes(body, assets),
                Node::Weave { body, .. } => Self::scan_nodes(body, assets),
                Node::MeshCall { args, .. } => Self::scan_nodes(args, assets),
                _ => {}
            }
        }
    }

    fn parse_target(target: &str) -> (String, PathBuf) {
        let (prefix_lang, clean_target) = if let Some((lang, path)) = target.split_once(':') {
            if !lang.contains('/') && !lang.contains('\\') && lang.len() <= 8 {
                (Some(lang.to_lowercase()), path)
            } else {
                (None, target)
            }
        } else {
            (None, target)
        };

        let path = PathBuf::from(clean_target);

        let lang = if let Some(l) = prefix_lang {
            l
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "py" | "pyw" => "py".to_string(),
                "java" => "java".to_string(),
                "cs" => "cs".to_string(),
                "c" => "c".to_string(),
                "cpp" | "cc" | "cxx" => "cpp".to_string(),
                "js" | "mjs" => "js".to_string(),
                "rs" => "rs".to_string(),
                other => other.to_string(),
            }
        } else {
            "bin".to_string()
        };

        // Resolve actual disk location (cwd or src/)
        let actual_path = if path.exists() {
            path
        } else {
            let in_src = Path::new("src").join(&path);
            if in_src.exists() {
                in_src
            } else if let Some(filename) = path.file_name() {
                let bare = Path::new(filename);
                if bare.exists() {
                    bare.to_path_buf()
                } else {
                    Path::new("src").join(filename)
                }
            } else {
                path
            }
        };

        (lang, actual_path)
    }
}