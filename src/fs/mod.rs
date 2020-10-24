mod fs_wrap;

use actix::prelude::*;
use std::path::PathBuf;
use notify::{RecommendedWatcher, Watcher, Result as FsResult, RecursiveMode, watcher, DebouncedEvent, RawEvent, raw_watcher};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::env::current_dir;

pub struct FsWatcher {
    evt_count: usize
}

impl Default for FsWatcher {
    fn default() -> Self {
        Self { evt_count: 0 }
    }
}

impl Actor for FsWatcher {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

#[derive(Message, Debug)]
#[rtype(result = "FsResult<()>")]
pub struct AddWatcher {
    pub pattern: PathBuf,
}

/// Handler for `ListRooms` message.
impl Handler<AddWatcher> for FsWatcher {
    type Result = FsResult<()>;

    fn handle(&mut self, msg: AddWatcher, ctx: &mut Context<Self>) -> Self::Result {
        // println!("AddWatcher = {}", msg.pattern.display());
        // log::trace!("AddWatcher = {}", msg.pattern.display());
        // let mut watcher: RecommendedWatcher = Watcher::new_immediate(|res| {
        //     match res {
        //         Ok(event) => log::debug!("event: {:?}", event),
        //         Err(e) => log::debug!("watch error: {:?}", e),
        //     }
        // })?;
        //
        // watcher.watch(PathBuf::from("/Users/shakyshane/sites/bs3/fixtures/app.js"), RecursiveMode::Recursive)?;
        //
        // log::debug!("watching!");

        // Create a channel to receive the events.
        // let (tx, rx) = channel();
        //
        // // Create a watcher object, delivering debounced events.
        // // The notification back-end is selected based on the platform.
        // let mut watcher = watcher(tx, Duration::from_millis(300))?;
        //
        // // Add a path to be watched. All files and directories at that path and
        // // below will be monitored for changes.
        // let cwd = PathBuf::from(current_dir()?);
        // watcher.watch(&cwd, RecursiveMode::Recursive)?;
        //
        // loop {
        //     match rx.recv() {
        //         Ok(event) => {
        //             match event {
        //                 DebouncedEvent::Write(pb) => log::debug!("+ Write {}", pb.display()),
        //                 DebouncedEvent::Create(pb) => log::debug!("+ Create {}", pb.display()),
        //                 DebouncedEvent::Remove(pb) => log::debug!("+ Remove {}", pb.display()),
        //                 DebouncedEvent::Rename(src, dest) => log::debug!("+ Rename {} -> {}", src.display(), dest.display()),
        //                 _evt => log::debug!("- {:?}", _evt)
        //             };
        //             // log::debug!("- {:?}", event);
        //             self.evt_count += 1;
        //         },
        //         Err(e) => log::debug!("watch error: {:?}", e),
        //     }
        // }

        Ok(())
    }
}
