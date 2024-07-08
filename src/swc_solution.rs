use anyhow::{Context, Result};
use std::collections::HashMap;
use std::io::Read;
use std::rc::Rc;
use swc_common::Globals;
use swc_common::GLOBALS;
use swc_common::{BytePos, Mark};
use swc_ecma_ast::ImportDecl;
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_codegen::Emitter;
use swc_ecma_parser::{EsSyntax, Parser, StringInput, Syntax};
use swc_ecma_transforms::feature::FeatureFlag;
use swc_ecma_transforms_module::common_js::common_js;
use swc_ecma_visit::Fold;

#[derive(Debug)]
struct ParseError;

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseError")
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone)]
pub struct Asset {
    pub id: usize,
    pub filename: String,
    pub dependencies: Vec<String>,
    pub code: String,
    pub mapping: HashMap<String, usize>,
}

impl std::fmt::Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Asset {{")?;
        writeln!(f, "id: {}", self.id)?;
        writeln!(f, "filename: {}", self.filename)?;
        writeln!(f, "dependencies: {:?}", self.dependencies)?;
        writeln!(f, "code: {}", self.code)?;
        writeln!(f, "mapping: {:?}", self.mapping)?;
        writeln!(f, "}}")
    }
}

pub fn create_asset(filepath: &str, id: usize) -> Result<Asset> {
    let content = read_code_to_string(filepath)?;

    let ast = get_ast(content, filepath)?;

    let dependencies = get_dependencies(&ast);

    let code = emit_common_js(ast)?;

    Ok(Asset {
        id,
        filename: {
            let path = std::path::Path::new(filepath);
            path.canonicalize().unwrap().to_str().unwrap().to_string()
        },
        dependencies,
        code,
        mapping: HashMap::new(),
    })
}

fn read_code_to_string(filepath: &str) -> Result<String> {
    let mut content = String::new();
    let mut file = std::fs::File::open(filepath)?;
    file.read_to_string(&mut content)?;

    Ok(content)
}

fn get_ast(text: String, filepath: &str) -> Result<swc_ecma_ast::Module> {
    let mut parser = Parser::new(
        Syntax::Es(EsSyntax::default()),
        StringInput::new(&text, BytePos(0), BytePos(0)),
        None,
    );
    let ast = parser
        .parse_module()
        .map_err(|_| ParseError)
        .context(format!("swc parser failed for {}", filepath))?;

    Ok(ast)
}

fn get_dependencies(ast: &swc_ecma_ast::Module) -> Vec<String> {
    let dependencies = ast
        .body
        .iter()
        .filter_map(|x| match x {
            swc_ecma_ast::ModuleItem::ModuleDecl(swc_ecma_ast::ModuleDecl::Import(
                ImportDecl { src, .. },
            )) => Some(src.value.to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();

    dependencies
}

fn emit_common_js(ast: swc_ecma_ast::Module) -> Result<String> {
    let code = GLOBALS.set(&Globals::new(), || {
        let mut code = common_js(
            Mark::fresh(Mark::root()),
            swc_ecma_transforms_module::util::Config {
                no_interop: true,
                ..Default::default()
            },
            FeatureFlag::default(),
            Some(swc_common::comments::NoopComments),
        );

        let folded_code = code.fold_module(ast);

        let mut writable: Vec<u8> = Vec::new();

        let source_map = Rc::new(swc_common::SourceMap::default());

        match (Emitter {
            cfg: Default::default(),
            comments: Some(&swc_common::comments::NoopComments),
            wr: Box::new(JsWriter::new(source_map.clone(), "\n", &mut writable, None)),
            cm: source_map,
        }
        .emit_module(&folded_code))
        {
            Ok(_) => {
                let code = String::from_utf8(writable).unwrap();

                Ok(code)
            }
            Err(e) => Err(e),
        }
    })?;

    Ok(code)
}
