use crate::core::Context;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher, event::ModifyKind};
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
    pub fn new(ctx: Context) -> Self {
        let (tx, _) = broadcast::channel(16);
        let foo = tx.clone();
        let src_dir = ctx.src_dir.clone();

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| {
                match res {
                    Ok(event) => {
                        eprintln!("event: {:?}", event);
                        match event.kind {
                            EventKind::Modify(ModifyKind::Data(_)) => {
                                for path in event.paths.iter() {
                                    dbg!(path);
                                    dbg!(ctx.resource_for_file(&path));
                                }
                            }
                            _ => (),
                        }
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

        watcher.watch(&src_dir, RecursiveMode::Recursive).unwrap();

        Self { watcher, tx: foo }
    }
}

#[tokio::main]
pub async fn blarg(ctx: Context) {
    let watch = Watch::new(ctx);

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
