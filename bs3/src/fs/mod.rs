use actix::prelude::*;
use actix::Context;
use notify::{
    raw_watcher, watcher, DebouncedEvent, FsEventWatcher, RawEvent, RecommendedWatcher,
    RecursiveMode, Result as FsResult, Watcher,
};
use std::path::PathBuf;

use bs3_files::served::ServedFile;
use crossbeam_channel;
use crossbeam_channel::internal::select;
use crossbeam_channel::unbounded;
use futures_util::StreamExt;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::collections::HashMap;
use std::env::current_dir;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct FsWatcher {
    items: HashMap<PathBuf, ServedFile>,
    listeners: HashMap<usize, Recipient<FsNotify>>,
    evt_count: usize,
    rng: ThreadRng,
    watcher: Option<FsEventWatcher>,
}

impl Default for FsWatcher {
    fn default() -> Self {
        Self {
            items: HashMap::new(),
            listeners: HashMap::new(),
            evt_count: 0,
            rng: rand::thread_rng(),
            watcher: None,
        }
    }
}

impl Actor for FsWatcher {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let a = actix_rt::Arbiter::new();
        let b = actix_rt::Arbiter::new();
        let (tx, rx) = channel();
        let watcher = watcher(tx, Duration::from_millis(300)).expect("create watcher failed");
        let (s, r) = unbounded::<DebouncedEvent>();

        self.watcher = Some(watcher);

        let self_address = ctx.address();

        a.exec_fn(move || {
            loop {
                match rx.recv() {
                    Ok(evt) => match s.send(evt) {
                        Ok(..) => println!("sent!"),
                        Err(e) => println!("send error = {:#?}", e),
                    },
                    Err(e) => {
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
pub struct FsNotify {
    pub item: ServedFile,
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
struct FsNotifyAll {
    pb: PathBuf,
}

/// Handler for `ListRooms` message.
impl Handler<FsNotifyAll> for FsWatcher {
    type Result = ();

    fn handle(&mut self, msg: FsNotifyAll, ctx: &mut Context<Self>) -> Self::Result {
        log::debug!("FsNotifyAll {:?}", msg);
        log::debug!(
            "FsNotifyAll self.listeners count: [{}]",
            self.listeners.len()
        );
        for (k, v) in self.listeners.iter() {
            if let Some(served) = self.items.get(&msg.pb) {
                log::debug!("found `served` {:?}", served);
                v.do_send(FsNotify {
                    item: served.clone(),
                });
            }
        }
    }
}

impl Handler<ServedFile> for FsWatcher {
    type Result = ();

    fn handle(&mut self, msg: ServedFile, ctx: &mut Context<Self>) -> Self::Result {
        // log::debug!("ServedFile = {:#?}", msg);
        self.items.insert(msg.path.clone(), msg.clone());
        if let Some(watcher) = self.watcher.as_mut() {
            log::debug!("+++ adding item to watch {}", msg.path.display());
            watcher
                .watch(&msg.path, RecursiveMode::NonRecursive)
                .expect("watcher.watch");
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
            log::debug!("next event = {:?}", pb);
            addr.do_send(FsNotifyAll { pb: pb.clone() });
        }
    }
}
