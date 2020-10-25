use actix::prelude::*;
use actix::Context;
use notify::{
    raw_watcher, watcher, DebouncedEvent, RawEvent, RecommendedWatcher, RecursiveMode,
    Result as FsResult, Watcher,
};
use std::path::PathBuf;

use bs3_files::served::ServedFile;
use futures_util::StreamExt;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::collections::HashMap;
use std::env::current_dir;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

pub struct FsWatcher {
    items: HashMap<usize, Recipient<ServedFile>>,
    listeners: HashMap<usize, Recipient<FsNotify>>,
    evt_count: usize,
    rng: ThreadRng,
    arbiters: HashMap<PathBuf, Arbiter>,
}

impl Default for FsWatcher {
    fn default() -> Self {
        Self {
            items: HashMap::new(),
            listeners: HashMap::new(),
            evt_count: 0,
            rng: rand::thread_rng(),
            arbiters: HashMap::new(),
        }
    }
}

impl Actor for FsWatcher {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct RegisterFs {
    pub addr: Recipient<FsNotify>,
}
impl Handler<RegisterFs> for FsWatcher {
    type Result = ();

    fn handle(&mut self, msg: RegisterFs, _ctx: &mut Context<Self>) -> Self::Result {
        let RegisterFs { addr } = msg;
        let id = self.rng.gen::<usize>();
        self.listeners.insert(id, addr);
        log::debug!(
            "+++ self.listeners adding for FsWatcher {:#?}",
            self.listeners
        );
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct FsNotify {
    pub item: ServedFile,
}

/// Handler for `ListRooms` message.
impl Handler<FsNotify> for FsWatcher {
    type Result = ();

    fn handle(&mut self, msg: FsNotify, ctx: &mut Context<Self>) -> Self::Result {
        for (k, v) in self.listeners.iter() {
            v.do_send(msg.clone());
        }
    }
}

impl Handler<ServedFile> for FsWatcher {
    type Result = ();

    fn handle(&mut self, msg: ServedFile, ctx: &mut Context<Self>) -> Self::Result {
        // log::debug!("ServedFile = {:#?}", msg);
        let add = ctx.address();
        if let Some(..) = self.arbiters.get(&msg.path) {
            log::debug!("bailing since we already have a listener");
        } else {
            log::debug!("+++ creating new Arbiter");
            let a = actix_rt::Arbiter::new();
            let pb_clone = msg.path.clone();
            a.send(Box::pin(async move {
                let self_add = add.clone();
                try_run(msg.clone(), self_add)
            }));
            self.arbiters.insert(pb_clone, a);
        }
    }
}

fn try_run(pattern: ServedFile, addr: Addr<FsWatcher>) {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_millis(300)).expect("create watcher failed");
    log::debug!("+~+ watching {}", pattern.path.display());
    let _ = watcher.watch(&pattern.path, RecursiveMode::NonRecursive);
    loop {
        let next: Option<PathBuf> = match rx.recv() {
            Ok(event) => {
                match event {
                    DebouncedEvent::Write(pb) => {
                        log::debug!("+ Write {}", pb.display());
                        Some(pb)
                    }
                    DebouncedEvent::Create(pb) => {
                        log::debug!("+ Create {}", pb.display());
                        Some(pb)
                    }
                    DebouncedEvent::Remove(pb) => {
                        log::debug!("+ Remove {}", pb.display());
                        Some(pb)
                    }
                    DebouncedEvent::Rename(src, dest) => {
                        log::debug!("+ Rename {} -> {}", src.display(), dest.display());
                        // Some(pb)
                        None
                    }
                    _evt => {
                        log::debug!("- {:?}", _evt);
                        None
                    }
                }
            }
            Err(e) => {
                log::debug!("watch error: {:?}", e);
                None
            }
        };
        log::debug!("next event");
        if let Some(pb) = next {
            addr.do_send(FsNotify {
                item: pattern.clone(),
            });
        }
    }
}
