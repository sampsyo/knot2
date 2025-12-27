use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher, event::ModifyKind};
use std::path::Path;
use tokio::sync::broadcast;

pub struct Watch {
    pub watcher: RecommendedWatcher,
    pub channel: broadcast::Sender<Event>,
}

// TODO this looks silly, but we have an enum here for possible future
// extensibility (only reloading one page instead of all of them)
#[derive(Debug, Clone)]
pub enum Event {
    Reload,
}

impl Watch {
    pub fn new(path: &Path) -> Self {
        let (tx, _) = broadcast::channel(16);
        let channel = tx.clone();

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| {
                match res {
                    Ok(event) => {
                        eprintln!("event: {:?}", event);
                        if let EventKind::Modify(ModifyKind::Data(_)) = event.kind {
                            for path in event.paths.iter() {
                                // TODO check if it's ignored
                                dbg!(path);
                            }
                        }
                        // TODO debounce
                        match tx.send(Event::Reload) {
                            Ok(_) => (),
                            Err(e) => eprintln!("channel send error: {e}"),
                        }
                    }
                    Err(error) => eprintln!("error: {}", error),
                };
            },
            Config::default(),
        )
        .unwrap();

        watcher.watch(path, RecursiveMode::Recursive).unwrap();

        Self { watcher, channel }
    }
}
