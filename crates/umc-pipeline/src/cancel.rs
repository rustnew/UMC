use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

/// Cooperative cancellation token shared across pipeline threads.
#[derive(Clone, Default, Debug)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self { cancelled: Arc::new(AtomicBool::new(false)) }
    }

    /// Signal cancellation — all threads will stop at their next checkpoint.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    /// Returns true if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cancel_flag() {
        let tok = CancellationToken::new();
        assert!(!tok.is_cancelled());
        tok.cancel();
        assert!(tok.is_cancelled());
    }

    #[test]
    fn test_clone_shares_flag() {
        let tok1 = CancellationToken::new();
        let tok2 = tok1.clone();
        tok1.cancel();
        assert!(tok2.is_cancelled());
    }
}
