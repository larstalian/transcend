use ringbuf::{
    rb::SharedRb,
    storage::Array,
    traits::Split,
    wrap::{CachingCons, CachingProd},
    Arc,
};

pub const SESSION_BUF_SIZE: usize = 128 * 1024;

type SessionRb = SharedRb<Array<u8, SESSION_BUF_SIZE>>;
pub type SessionProd = CachingProd<Arc<SessionRb>>;
pub type SessionCons = CachingCons<Arc<SessionRb>>;

pub struct SessionChannel {
    pub prod: SessionProd,
    pub cons: SessionCons,
}

impl SessionChannel {
    pub fn new() -> Self {
        // One heap alloc here for the Arc+ring, nothing on the hot path.
        let rb = Arc::new(SessionRb::default());
        let (prod, cons) = rb.split();
        Self { prod, cons }
    }
}
