use actix::prelude::*;
use actix::Context;
use notify::{
    raw_watcher, watcher, DebouncedEvent, RawEvent, RecommendedWatcher, RecursiveMode,
    Result as FsResult, Watcher,
};
use std::path::PathBuf;

use bs3_files::served::ServedFile;
use std::collections::HashMap;
use std::time::Duration;
use std::env::current_dir;
use std::sync::mpsc::channel;

pub struct FsWatcher {
    items: HashMap<usize, Recipient<ServedFile>>,
    evt_count: usize,
}

impl Default for FsWatcher {
    fn default() -> Self {
        Self { items: HashMap::new(), evt_count: 0 }
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

        let fn2 = futures::future::lazy(move |_| {
            // Create a channel to receive the events.
            let (tx, rx) = channel();
            //
            // // Create a watcher object, delivering debounced events.
            // // The notification back-end is selected based on the platform.
            let mut watcher = watcher(tx, Duration::from_millis(300))?;
            //
            // // Add a path to be watched. All files and directories at that path and
            // // below will be monitored for changes.
            log::debug!("watching {}", msg.pattern.display());
            watcher.watch(&msg.pattern, RecursiveMode::Recursive)?;
            loop {
                match rx.recv() {
                    Ok(event) => {
                        match event {
                            DebouncedEvent::Write(pb) => log::debug!("+ Write {}", pb.display()),
                            DebouncedEvent::Create(pb) => log::debug!("+ Create {}", pb.display()),
                            DebouncedEvent::Remove(pb) => log::debug!("+ Remove {}", pb.display()),
                            DebouncedEvent::Rename(src, dest) => log::debug!("+ Rename {} -> {}", src.display(), dest.display()),
                            _evt => log::debug!("- {:?}", _evt)
                        };
                        // log::debug!("- {:?}", event);
                        self.evt_count += 1;
                    },
                    Err(e) => log::debug!("watch error: {:?}", e),
                }
            }
            Ok(())
        });

        let h = ctx.spawn(fn2);


        Ok(())
    }
}

impl Handler<ServedFile> for FsWatcher {
    type Result = ();

    fn handle(&mut self, msg: ServedFile, ctx: &mut Context<Self>) -> Self::Result {
        log::debug!("ServedFile = {:#?}", msg);
        let add = ctx.address();
        add.do_send(AddWatcher { pattern: msg.path })
    }
}
