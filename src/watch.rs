use notify::{
    Config, EventHandler, EventKind, RecommendedWatcher, RecursiveMode, Watcher, event::ModifyKind,
};
use std::path::{Component, Path, PathBuf};
use tokio::sync::broadcast;

// TODO this looks silly, but we have an enum here for possible future
// extensibility (only reloading one page instead of all of them)
#[derive(Debug, Clone)]
pub enum Event {
    Reload,
}

pub struct Handler {
    pub base: PathBuf,
    pub channel: broadcast::Sender<Event>,
}

impl EventHandler for Handler {
    fn handle_event(&mut self, res: notify::Result<notify::Event>) {
        if let Ok(event) = res
            && let EventKind::Modify(ModifyKind::Data(_)) = event.kind
            && !event.paths.iter().any(|p| ignore_path(&self.base, p))
        {
            // TODO debounce
            // We ignore errors when sending events: it's OK to
            // silently drop messages when there are no subscribers.
            let _ = self.channel.send(Event::Reload);
        }
    }
}

pub fn watch(path: &Path) -> (RecommendedWatcher, broadcast::Sender<Event>) {
    let (tx, _) = broadcast::channel(16);

    let handler = Handler {
        base: std::path::absolute(path).expect("need absolute base path"),
        channel: tx.clone(),
    };
    let mut watcher = RecommendedWatcher::new(handler, Config::default()).unwrap();

    watcher.watch(path, RecursiveMode::Recursive).unwrap();

    (watcher, tx)
}

/// Check whether we should ignore a given path inside of a base directory.
///
/// It's ignored if any component below `base` is an ignored filename. Also,
/// anything outside `base` is also ignored. Both arguments must be provided as
/// absolute paths.
fn ignore_path(base: &Path, path: &Path) -> bool {
    let frag = match path.strip_prefix(base) {
        Ok(p) => p,
        Err(_) => return true,
    };
    for comp in frag.components() {
        if let Component::Normal(name) = comp
            && crate::core::ignore_filename(name)
        {
            return true;
        }
    }
    false
}
