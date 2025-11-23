use std::fs;
use std::path::Path;

pub struct Assets<E> {
    /// The directory path for this set of assets.
    dir: &'static str,

    /// The names of the available asset files.
    names: &'static [&'static str],

    /// The contents of the files listed in `names`, if available.
    embedded: E,
}

impl<E> Assets<E> {
    pub fn contains(&self, name: &str) -> bool {
        self.names.contains(&name)
    }

    pub fn load(&self, name: &str) -> std::io::Result<Option<String>> {
        if self.contains(name) {
            let path = Path::new(self.dir).join(name);
            fs::read_to_string(path).map(|c| Some(c))
        } else {
            Ok(None)
        }
    }
}

pub type EmbeddedAssets = Assets<&'static [&'static str]>;
pub type FileAssets = Assets<()>;

impl EmbeddedAssets {
    pub const fn new(
        dir: &'static str,
        names: &'static [&'static str],
        contents: &'static [&'static str],
    ) -> Self {
        if contents.len() != names.len() {
            panic!("contents should match filenames");
        }
        Self {
            dir,
            names,
            embedded: contents,
        }
    }

    pub fn get_embedded(&self, name: &str) -> Option<&'static str> {
        assert_eq!(self.embedded.len(), self.names.len());

        // Look up the contents associated with the name.
        match self.names.iter().position(|n| *n == name) {
            Some(idx) => Some(self.embedded[idx]),
            None => None,
        }
    }

    pub fn embedded_files(&self) -> impl Iterator<Item = (&'static str, &'static str)> {
        std::iter::zip(self.names, self.embedded).map(|(n, e)| (*n, *e))
    }
}

impl FileAssets {
    pub const fn new(dir: &'static str, names: &'static [&'static str]) -> Self {
        Self {
            dir,
            names,
            embedded: (),
        }
    }

    pub fn get_embedded(&self, name: &str) -> Option<&'static str> {
        None
    }
}

#[macro_export]
macro_rules! embed_assets {
    ($constname:ident, $dirname:literal, $($filename:literal),*) => {
        const $constname: $crate::assets::EmbeddedAssets = $crate::assets::EmbeddedAssets::new(
            concat!(env!("CARGO_MANIFEST_DIR"), "/", $dirname),
            &[$( $filename, )*],
            &[$(
                include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $dirname, "/", $filename)),
            )*],
        );
    };
}

#[macro_export]
macro_rules! file_assets {
    ($constname:ident, $dirname:literal, $($filename:literal),*) => {
        const $constname: $crate::assets::FileAssets = $crate::assets::FileAssets::new(
            concat!(env!("CARGO_MANIFEST_DIR"), "/", $dirname),
            &[$( $filename, )*],
        );
    };
}

#[macro_export]
macro_rules! assets {
    ($constname:ident, $dirname:literal, $($filename:literal),*) => {
        #[cfg(debug_assertions)]
        file_assets!($constname, $dirname, "note.html");

        #[cfg(not(debug_assertions))]
        embed_assets!($constname, $dirname, "note.html");
    };
}
