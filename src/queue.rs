use {
    ringbuf::{
        traits::Split,
        wrap::{CachingCons, CachingProd},
        StaticRb,
    },
    std::sync::Arc,
};

pub const OUT_BUF_SIZE: usize = 128 * 1024;
pub const IN_BUF_SIZE: usize = 8 * 1024;

// Outbound: PTY -> session
type OutRb = StaticRb<u8, OUT_BUF_SIZE>;
pub type OutProd = CachingProd<Arc<OutRb>>;
pub type OutCons = CachingCons<Arc<OutRb>>;

// Inbound: session -> PTY
type InRb = StaticRb<u8, IN_BUF_SIZE>;
pub type InProd = CachingProd<Arc<InRb>>;
pub type InCons = CachingCons<Arc<InRb>>;

pub struct IoChannels {
    pub out_prod: OutProd, // PTY writes here
    pub out_cons: OutCons, // Session reads here
    pub in_prod: InProd,   // Session writes here
    pub in_cons: InCons,   // PTY reads here
}

impl IoChannels {
    pub fn new() -> Self {
        // PTY → TUI
        let out_rb = Arc::new(OutRb::default());
        let (out_prod, out_cons) = out_rb.split();

        // TUI → PTY
        let in_rb = Arc::new(InRb::default());
        let (in_prod, in_cons) = in_rb.split();

        Self {
            out_prod,
            out_cons,
            in_prod,
            in_cons,
        }
    }
}
