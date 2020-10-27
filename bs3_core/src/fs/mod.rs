use actix::prelude::*;
use actix::Context;
use notify::{watcher, DebouncedEvent, FsEventWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;

use bs3_files::served::ServedFile;
use crossbeam_channel;

use crossbeam_channel::unbounded;

use rand::rngs::ThreadRng;
use rand::Rng;
use std::collections::{HashMap, HashSet};

use std::sync::mpsc::channel;

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
                    Ok(evt) => match s.send(evt) {
                        Err(e) => println!("send error = {:#?}", e),
                        _ => { /* noop */ }
                    },
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

#[derive(Message, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[rtype(result = "()")]
pub struct FsNotify {
    pub item: ServedFile,
}

impl FsNotify {
    pub fn new(item: impl Into<ServedFile>) -> Self {
        Self { item: item.into() }
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

impl Handler<ServedFile> for FsWatcher {
    type Result = ();

    fn handle(&mut self, msg: ServedFile, _ctx: &mut Context<Self>) -> Self::Result {
        // log::debug!("ServedFile = {:#?}", msg);
        if self.watched.contains(&msg.path) {
            log::trace!("!! skipping, already watching: {}", msg.path.display());
            return ();
        }
        self.items.insert(msg.path.clone(), msg.clone());
        if let Some(watcher) = self.watcher.as_mut() {
            log::debug!("+++ adding item to watch {}", msg.path.display());
            let result = watcher.watch(&msg.path, RecursiveMode::NonRecursive);
            match result {
                Ok(..) => {
                    self.watched.insert(msg.path);
                }
                Err(e) => {
                    log::error!("Could not watch the path {}", msg.path.display());
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
