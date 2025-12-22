use notify_debouncer_mini::{DebounceEventResult, new_debouncer, notify::*};
use std::path::Path;
use std::time::Duration;

pub fn blarg(path: &Path) {
    let mut debouncer =
        new_debouncer(
            Duration::from_millis(100),
            |res: DebounceEventResult| match res {
                Ok(events) => events
                    .iter()
                    .for_each(|e| println!("Event {:?} for {:?}", e.kind, e.path)),
                Err(e) => println!("Error {:?}", e),
            },
        )
        .unwrap();

    debouncer
        .watcher()
        .watch(path, RecursiveMode::Recursive)
        .unwrap();

    loop {}
}
