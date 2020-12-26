use actix::prelude::*;
use actix::Context;
use notify::{watcher, DebouncedEvent, FsEventWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;

use crossbeam_channel::unbounded;

use rand::rngs::ThreadRng;
use rand::Rng;
use std::collections::{HashMap, HashSet};

use std::sync::mpsc::channel;

use crate::ws::client::{FsNotify, ServedFile};
use std::time::Duration;

pub struct FsWatcher {
    items: HashMap<PathBuf, ServedFile>,
    listeners: HashMap<usize, Recipient<FsNotify>>,
    rng: ThreadRng,
    watcher: Option<FsEventWatcher>,
    watched: HashSet<PathBuf>,
}

impl Default for FsWatcher {
    fn default() -> Self {
        Self {
            items: HashMap::new(),
            listeners: HashMap::new(),
            rng: rand::thread_rng(),
            watcher: None,
            watched: HashSet::new(),
        }
    }
}

impl Actor for FsWatcher {
    type Context = Context<Self>;

    ///
    /// When this actor starts we start a couple of threads (via the Arbiter)
    /// to handle listening to file-system events in a none-blocking way
    ///
    fn started(&mut self, ctx: &mut Self::Context) {
        let a = actix_rt::Arbiter::new();
        let b = actix_rt::Arbiter::new();
        let (tx, rx) = channel();
        let watcher = watcher(tx, Duration::from_millis(300)).expect("create watcher failed");
        let (s, r) = unbounded::<DebouncedEvent>();

        // save the watcher, so that we can add more patterns later (eg: when files are served)
        self.watcher = Some(watcher);

        let self_address = ctx.address();

        a.exec_fn(move || {
            loop {
                match rx.recv() {
                    Ok(evt) => {
                        if let Err(e) = s.send(evt) {
                            println!("send error = {:#?}", e);
                        }
                    }
                    Err(_e) => {
                        // no-op, we cannot recover/handle this
                    }
                };
            }
        });

        b.exec_fn(move || {
            let self_add = self_address.clone();
            receive_fs_messages(self_add, r);
        });
    }
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
struct FsNotifyAll {
    pb: PathBuf,
}

/// Handler for `ListRooms` message.
impl Handler<FsNotifyAll> for FsWatcher {
    type Result = ();

    fn handle(&mut self, msg: FsNotifyAll, _ctx: &mut Context<Self>) -> Self::Result {
        log::debug!("{:?}", msg);
        log::trace!(
            "FsNotifyAll self.listeners count: [{}]",
            self.listeners.len()
        );
        for (_k, v) in self.listeners.iter() {
            if let Some(served) = self.items.get(&msg.pb) {
                log::trace!("found `served` {:?}", served);
                if let Err(_e) = v.do_send(FsNotify::new(served.clone())) {
                    log::error!("failed to send FsNotify to a listener");
                }
            }
        }
    }
}

///
/// Convert the foreign type into one that resides in this crate
///
impl From<bs3_files::served::ServedFile> for ServedFile {
    fn from(served: bs3_files::served::ServedFile) -> Self {
        Self {
            web_path: served.web_path.clone(),
            path: served.path.clone(),
            referer: served.referer,
        }
    }
}

impl Handler<bs3_files::served::ServedFile> for FsWatcher {
    type Result = ();

    fn handle(
        &mut self,
        msg: bs3_files::served::ServedFile,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        // log::debug!("ServedFile = {:#?}", msg);
        if self.watched.contains(&msg.path) {
            log::trace!("!! skipping, already watching: {}", msg.path.display());
            return;
        }
        let clone: ServedFile = msg.into();
        self.items.insert(clone.path.clone(), clone.clone());
        if let Some(watcher) = self.watcher.as_mut() {
            log::debug!("+++ adding item to watch {}", clone.path.display());
            let result = watcher.watch(&clone.path, RecursiveMode::NonRecursive);
            match result {
                Ok(..) => {
                    self.watched.insert(clone.path);
                }
                Err(e) => {
                    log::error!("Could not watch the path {}", clone.path.display());
                    log::error!(" ^^ {}", e);
                }
            }
        }
    }
}

fn receive_fs_messages(addr: Addr<FsWatcher>, rx: crossbeam_channel::Receiver<DebouncedEvent>) {
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
                        // log::debug!("- {:?}", _evt);
                        None
                    }
                }
            }
            Err(_e) => None,
        };
        if let Some(pb) = next {
            log::trace!("path in question: = {:?}", pb);
            addr.do_send(FsNotifyAll { pb: pb.clone() });
        }
    }
}
