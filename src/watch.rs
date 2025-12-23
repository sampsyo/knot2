use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tokio::sync::broadcast;

pub struct Watch {
    watcher: RecommendedWatcher,
    tx: broadcast::Sender<Event>,
}

#[derive(Debug, Clone)]
pub enum Event {
    One(String),
    All,
}

impl Watch {
    pub fn new(path: &Path) -> Self {
        let (tx, _) = broadcast::channel(16);
        let foo = tx.clone();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                match res {
                    Ok(event) => {
                        eprintln!("event: {:?}", event);
                        match tx.send(Event::All) {
                            Ok(_) => (),
                            Err(e) => eprintln!("channel send error: {e}"),
                        }
                    }
                    Err(error) => eprintln!("error: {}", error),
                };
                ()
            },
            Config::default(),
        )
        .unwrap();

        watcher.watch(path, RecursiveMode::Recursive).unwrap();

        Self { watcher, tx: foo }
    }
}

#[tokio::main]
pub async fn blarg(path: &Path) {
    let watch = Watch::new(path);

    let mut rx = watch.tx.subscribe();
    tokio::spawn(async move {
        loop {
            let event = rx.recv().await.unwrap();
            dbg!(event);
        }
    })
    .await
    .unwrap();
}
