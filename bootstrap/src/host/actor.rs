use std::collections::{BTreeMap, VecDeque};

static mut NEXT_ID: u64 = 1;
fn next_actor_id() -> u64 { unsafe { let id = NEXT_ID; NEXT_ID += 1; id } }
fn reset_ids() { unsafe { NEXT_ID = 1; } }

pub type HandlerFn = Box<dyn FnMut(&mut ActorContext, Vec<u8>)>;

pub struct ActorContext {
    pub id: u64,
    inbox: VecDeque<(u64, Vec<u8>)>,
}

impl ActorContext {
    pub fn new(id: u64) -> Self { Self { id, inbox: VecDeque::new() } }
    pub fn inbox_len(&self) -> usize { self.inbox.len() }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActorError {
    NotFound { id: u64 },
    AlreadyExists { id: u64 },
    Stopped { id: u64 },
    InboxFull { id: u64 },
}

impl std::fmt::Display for ActorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActorError::NotFound { id } => write!(f, "actor {id} not found"),
            ActorError::AlreadyExists { id } => write!(f, "actor {id} exists"),
            ActorError::Stopped { id } => write!(f, "actor {id} stopped"),
            ActorError::InboxFull { id } => write!(f, "actor {id} inbox full"),
        }
    }
}

impl std::error::Error for ActorError {}

struct ActorEntry {
    id: u64,
    ctx: ActorContext,
    handler: HandlerFn,
    stopped: bool,
    processed: u64,
}

pub struct ActorSystem {
    actors: BTreeMap<u64, ActorEntry>,
    inbox_capacity: usize,
    total_sent: u64,
    total_processed: u64,
}

impl ActorSystem {
    pub fn new(inbox_capacity: usize) -> Self {
        Self { actors: BTreeMap::new(), inbox_capacity, total_sent: 0, total_processed: 0 }
    }

    pub fn spawn(&mut self, handler: HandlerFn) -> u64 {
        let id = next_actor_id();
        let ctx = ActorContext::new(id);
        self.actors.insert(id, ActorEntry { id, ctx, handler, stopped: false, processed: 0 });
        id
    }

    pub fn send(&mut self, target: u64, sender: u64, msg: Vec<u8>) -> Result<(), ActorError> {
        let entry = self.actors.get_mut(&target).ok_or(ActorError::NotFound { id: target })?;
        if entry.stopped { return Err(ActorError::Stopped { id: target }); }
        if entry.ctx.inbox.len() >= self.inbox_capacity { return Err(ActorError::InboxFull { id: target }); }
        entry.ctx.inbox.push_back((sender, msg));
        self.total_sent += 1;
        Ok(())
    }

    pub fn tick(&mut self, target: u64) -> Result<usize, ActorError> {
        let entry = self.actors.get_mut(&target).ok_or(ActorError::NotFound { id: target })?;
        if entry.stopped { return Err(ActorError::Stopped { id: target }); }
        let batch: Vec<(u64, Vec<u8>)> = entry.ctx.inbox.drain(..).collect();
        let count = batch.len();
        for (sender, msg) in batch {
            let handler = &mut entry.handler;
            let ctx = &mut entry.ctx;
            handler(ctx, msg);
            let _ = sender;
        }
        let entry = self.actors.get_mut(&target).unwrap();
        entry.processed += count as u64;
        self.total_processed += count as u64;
        Ok(count)
    }

    pub fn tick_all(&mut self) -> usize {
        let ids: Vec<u64> = self.actors.keys().copied().collect();
        let mut total = 0;
        for id in ids {
            if let Ok(n) = self.tick(id) { total += n; }
        }
        total
    }

    pub fn stop(&mut self, id: u64) -> Result<(), ActorError> {
        let entry = self.actors.get_mut(&id).ok_or(ActorError::NotFound { id })?;
        entry.stopped = true;
        Ok(())
    }

    pub fn inbox_len(&self, id: u64) -> Option<usize> {
        self.actors.get(&id).map(|e| e.ctx.inbox.len())
    }

    pub fn is_stopped(&self, id: u64) -> Option<bool> {
        self.actors.get(&id).map(|e| e.stopped)
    }

    pub fn actor_count(&self) -> usize { self.actors.len() }
    pub fn active_count(&self) -> usize { self.actors.values().filter(|a| !a.stopped).count() }
    pub fn total_sent(&self) -> u64 { self.total_sent }
    pub fn total_processed(&self) -> u64 { self.total_processed }
    pub fn processed_by(&self, id: u64) -> Option<u64> { self.actors.get(&id).map(|a| a.processed) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn setup() { reset_ids(); }

    #[test]
    fn spawn_actor() {
        setup();
        let mut sys = ActorSystem::new(10);
        let id = sys.spawn(Box::new(|_ctx, _msg| {}));
        assert_eq!(sys.actor_count(), 1);
        assert_eq!(id, 1);
    }

    #[test]
    fn send_and_tick() {
        setup();
        let counter = Arc::new(AtomicUsize::new(0));
        let c2 = counter.clone();
        let mut sys = ActorSystem::new(10);
        let id = sys.spawn(Box::new(move |_ctx, _msg| { c2.fetch_add(1, Ordering::SeqCst); }));
        sys.send(id, 0, vec![1]).unwrap();
        sys.tick(id).unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn not_found() {
        setup();
        let mut sys = ActorSystem::new(10);
        let err = sys.send(99, 0, vec![]).unwrap_err();
        assert!(matches!(err, ActorError::NotFound { .. }));
    }

    #[test]
    fn inbox_full() {
        setup();
        let mut sys = ActorSystem::new(2);
        let id = sys.spawn(Box::new(|_ctx, _msg| {}));
        sys.send(id, 0, vec![1]).unwrap();
        sys.send(id, 0, vec![2]).unwrap();
        let err = sys.send(id, 0, vec![3]).unwrap_err();
        assert!(matches!(err, ActorError::InboxFull { .. }));
    }

    #[test]
    fn stop_actor() {
        setup();
        let mut sys = ActorSystem::new(10);
        let id = sys.spawn(Box::new(|_ctx, _msg| {}));
        sys.stop(id).unwrap();
        let err = sys.send(id, 0, vec![]).unwrap_err();
        assert!(matches!(err, ActorError::Stopped { .. }));
        assert_eq!(sys.active_count(), 0);
    }

    #[test]
    fn tick_all() {
        setup();
        let counter = Arc::new(AtomicUsize::new(0));
        let c2 = counter.clone();
        let mut sys = ActorSystem::new(10);
        let a1 = sys.spawn(Box::new(move |_ctx, _msg| { c2.fetch_add(1, Ordering::SeqCst); }));
        let a2 = sys.spawn(Box::new(|_ctx, _msg| {}));
        sys.send(a1, 0, vec![]).unwrap();
        sys.send(a2, 0, vec![]).unwrap();
        let total = sys.tick_all();
        assert_eq!(total, 2);
    }

    #[test]
    fn batch_messages() {
        setup();
        let counter = Arc::new(AtomicUsize::new(0));
        let c2 = counter.clone();
        let mut sys = ActorSystem::new(10);
        let id = sys.spawn(Box::new(move |_ctx, _msg| { c2.fetch_add(1, Ordering::SeqCst); }));
        sys.send(id, 0, vec![1]).unwrap();
        sys.send(id, 0, vec![2]).unwrap();
        sys.send(id, 0, vec![3]).unwrap();
        let n = sys.tick(id).unwrap();
        assert_eq!(n, 3);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn inbox_len() {
        setup();
        let mut sys = ActorSystem::new(10);
        let id = sys.spawn(Box::new(|_ctx, _msg| {}));
        sys.send(id, 0, vec![]).unwrap();
        assert_eq!(sys.inbox_len(id), Some(1));
    }

    #[test]
    fn stats() {
        setup();
        let mut sys = ActorSystem::new(10);
        let id = sys.spawn(Box::new(|_ctx, _msg| {}));
        sys.send(id, 0, vec![]).unwrap();
        sys.tick(id).unwrap();
        assert_eq!(sys.total_sent(), 1);
        assert_eq!(sys.total_processed(), 1);
        assert_eq!(sys.processed_by(id), Some(1));
    }

    #[test]
    fn multiple_actors() {
        setup();
        let mut sys = ActorSystem::new(10);
        let a1 = sys.spawn(Box::new(|_ctx, _msg| {}));
        let a2 = sys.spawn(Box::new(|_ctx, _msg| {}));
        sys.send(a1, a2, vec![]).unwrap();
        assert_eq!(sys.actor_count(), 2);
    }

    #[test]
    fn stop_not_found() {
        setup();
        let mut sys = ActorSystem::new(10);
        let err = sys.stop(99).unwrap_err();
        assert!(matches!(err, ActorError::NotFound { .. }));
    }

    #[test]
    fn error_display() {
        assert!(ActorError::NotFound { id: 5 }.to_string().contains("5"));
    }
}
