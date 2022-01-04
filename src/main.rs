use quote::ToTokens;
use std::fs::File;
use std::io::Read;
use syn::visit::{self, Visit};
use syn::visit_mut::VisitMut;
use syn::ItemFn;

fn find_rust_source_files(
    dir: &std::path::Path,
    files_list: &mut Vec<std::path::PathBuf>,
) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() && !path.ends_with("tests") && !path.ends_with("benches") {
                // ignore directories that contain code that isn't used in builds
                find_rust_source_files(&path, files_list)?;
            } else if let Some("rs") = path.extension().map(|x| x.to_str().unwrap()) {
                // add rust files to list
                // todo find a nice, non-panic-ing way to do this conditional
                // this panics if the extension is not valid unicode
                if !path.ends_with("build.rs") {
                    // ignore build.rs files
                    // todo promote this to the parent if let when it's no longer experimental
                    files_list.push(path)
                }
            } else {
                // we have reached a dead end file or directory
            }
        }
    }
    Ok(())
}

fn get_file(filename: &std::path::Path) -> String {
    let mut file = File::open(&filename).expect("Unable to open file");

    let mut src = String::new();
    file.read_to_string(&mut src).expect("Unable to read file");
    src
}

fn output_file(filename: &std::path::Path, syntax: syn::File) {
    let buf = syntax.into_token_stream().to_string();
    std::fs::write(&filename, buf.as_bytes()).expect("Unable to write file");
}

struct UnsafeVisitor {
    // Whether or not the
    pub contains_unsafe: bool,
}

impl UnsafeVisitor {
    pub fn is_checkable_unsafe_method(&mut self, ) {

    }
}

impl<'ast> Visit<'ast> for UnsafeVisitor {
    /* We want function usages not function definitions
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        println!("Function with name={}", node.sig.ident);

        // Delegate to the default impl to visit any nested functions.
        visit::visit_item_fn(self, node);
    } */

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        println!("Method Call with name={}", node.method);

        visit::visit_expr_method_call(self, node)
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        match &*node.func {
            syn::Expr::Path(p) => {
                println!(
                    "Expr Call with name={}",
                    p.path.segments.last().unwrap().ident
                );
            }
            _ => {}
        }

        visit::visit_expr_call(self, node)
    }

    /*  I'm not sure yet if there is anything to be found in expr_unsafe
    fn visit_expr_unsafe(&mut self, node: &'ast syn::ExprUnsafe) {
        visit::visit_expr_unsafe(self, node)
    } */
}

impl VisitMut for UnsafeVisitor {}

// todo
/* Ways to output back out the edits
- Do in place on the original file
    - Fast?
    - Overwrites user files?
- Create new files in directory
    - Slow?
    - Easier to create edits/versions
    - Large?
- Just output a binary somehow?
    - How?
*/

fn main() -> Result<(), std::io::Error> {
    let source_dir = std::path::Path::new("vendor");
    let output_dir = std::path::Path::new("vendor");

    // Find all rust files in dependencies
    let mut files = Vec::new();
    find_rust_source_files(source_dir, &mut files)?;
    dbg!(&files);

    /*     let filename = get_current_filename();
    let file = get_file(&filename); */

    //find files with unsafe methods we are interested in
    for file in files {
        dbg!(&file);

        let syntax = syn::parse_file(&get_file(&file)).expect("Unable to parse file");

        let mut visitor = UnsafeVisitor {
            contains_unsafe: false,
        };

        visitor.visit_file(&syntax);

        let root_file = file.strip_prefix(source_dir).unwrap();
        let output_filename = output_dir.join(&root_file);
        if !output_filename.exists() {
            std::fs::create_dir_all(&output_filename.parent().unwrap()).unwrap()
        }
        //output_file(&output_filename, syntax);
    }
    Ok(())
}
