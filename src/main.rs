use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use std::fs;

const TMPL_DIR: &str = "templates";
const NOTE_TEMPLATE: &str = "note.html";

struct Context {
    src_dir: Utf8PathBuf,
    dest_dir: Utf8PathBuf,
    tmpls: minijinja::Environment<'static>,
}

fn load_template(name: &str) -> std::result::Result<Option<String>, minijinja::Error> {
    // Only accept plain filenames, not paths.
    if name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        return Ok(None);
    }

    // Load the named template from disk.
    let path = Utf8Path::new(TMPL_DIR).join(name);
    match fs::read_to_string(path) {
        Ok(source) => Ok(Some(source)),
        Err(_) => Ok(None), // TODO maybe propagate error
    }
}

impl Context {
    fn new(src_dir: &str, dest_dir: &str) -> Self {
        let mut env = minijinja::Environment::new();

        // In release mode, embed template files.
        #[cfg(not(debug_assertions))]
        {
            const DIR: include_dir::Dir =
                include_dir::include_dir!("$CARGO_MANIFEST_DIR/templates");
            for file in DIR.files() {
                let name = file
                    .path()
                    .file_name()
                    .expect("embedded path is a filename")
                    .to_str()
                    .expect("embedded path is UTF-8");
                let source = file.contents_utf8().expect("embedded template is UTF-8");
                env.add_template(name, source)
                    .expect("embedded template is valid Jinja code");
            }
        }

        // In debug mode, load directly from the filesystem.
        #[cfg(debug_assertions)]
        env.set_loader(load_template);

        Self {
            src_dir: src_dir.into(),
            dest_dir: dest_dir.into(),
            tmpls: env,
        }
    }

    /**
     * If this is a note filename, return its destination name. Otherwise, return None.
     */
    fn note_dest(&self, name: &str) -> Option<Utf8PathBuf> {
        if name.starts_with("_") || name.starts_with(".") {
            return None;
        }
        let (base, ext) = name.split_once(".")?;
        if ext != "md" {
            return None;
        }
        Some(self.dest_dir.join(format!("{base}.html")))
    }

    fn render_note(&self, src_path: &Utf8Path, dest_path: &Utf8Path) -> Result<()> {
        let source = fs::read_to_string(src_path)?;
        let parser = pulldown_cmark::Parser::new(&source);
        let body = {
            let mut b = String::new();
            pulldown_cmark::html::push_html(&mut b, parser);
            b
        };

        let out_file = fs::File::create(dest_path)?;

        let tmpl = self.tmpls.get_template(NOTE_TEMPLATE)?;
        tmpl.render_to_write(
            minijinja::context! {
                body => body,
            },
            out_file,
        )?;

        Ok(())
    }

    fn render_all(&self) -> Result<()> {
        fs::create_dir_all(&self.dest_dir)?;
        // TODO parallelize
        for entry in self.src_dir.read_dir_utf8()? {
            let entry = entry?;
            if let Some(dest_path) = self.note_dest(entry.file_name()) {
                match self.render_note(entry.path(), &dest_path) {
                    Ok(_) => (),
                    Err(e) => eprintln!("error rendering note {}: {}", entry.file_name(), e),
                }
            }
        }
        Ok(())
    }
}

fn main() {
    let ctx = Context::new(".", "_public");
    ctx.render_all().unwrap();
}
