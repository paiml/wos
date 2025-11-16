//! Signal Handling System
//!
//! POSIX-like signal handling for process communication and control.

use serde::{Deserialize, Serialize};

/// Signal types supported by WOS
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum Signal {
    /// Interrupt from keyboard (Ctrl+C)
    SIGINT = 2,
    /// Termination signal
    SIGTERM = 15,
    /// Kill signal (cannot be caught or ignored)
    SIGKILL = 9,
    /// User-defined signal 1
    SIGUSR1 = 10,
    /// User-defined signal 2
    SIGUSR2 = 12,
    /// Child process terminated or stopped
    SIGCHLD = 17,
    /// Segmentation fault
    SIGSEGV = 11,
    /// Broken pipe: write to pipe with no readers
    SIGPIPE = 13,
}

impl Signal {
    /// Get signal from number
    pub fn from_number(num: u32) -> Option<Self> {
        match num {
            2 => Some(Signal::SIGINT),
            15 => Some(Signal::SIGTERM),
            9 => Some(Signal::SIGKILL),
            10 => Some(Signal::SIGUSR1),
            12 => Some(Signal::SIGUSR2),
            17 => Some(Signal::SIGCHLD),
            11 => Some(Signal::SIGSEGV),
            13 => Some(Signal::SIGPIPE),
            _ => None,
        }
    }

    /// Get signal number
    pub fn number(&self) -> u32 {
        *self as u32
    }

    /// Check if signal can be caught or ignored
    pub fn is_catchable(&self) -> bool {
        !matches!(self, Signal::SIGKILL)
    }

    /// Get default action for signal
    pub fn default_action(&self) -> SignalAction {
        match self {
            Signal::SIGKILL
            | Signal::SIGTERM
            | Signal::SIGINT
            | Signal::SIGSEGV
            | Signal::SIGPIPE => SignalAction::Terminate,
            Signal::SIGCHLD => SignalAction::Ignore,
            Signal::SIGUSR1 | Signal::SIGUSR2 => SignalAction::Terminate,
        }
    }
}

/// Signal action to take when signal is delivered
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalAction {
    /// Terminate the process
    Terminate,
    /// Ignore the signal
    Ignore,
    /// Execute custom handler (handler ID)
    Handler(u32),
}

/// Signal set for pending and blocked signals
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalSet {
    /// Bitmask of signals (bit N set if signal N is present)
    mask: u64,
}

impl SignalSet {
    /// Create empty signal set
    pub fn new() -> Self {
        Self { mask: 0 }
    }

    /// Add signal to set
    pub fn add(&mut self, signal: Signal) {
        self.mask |= 1 << signal.number();
    }

    /// Remove signal from set
    pub fn remove(&mut self, signal: Signal) {
        self.mask &= !(1 << signal.number());
    }

    /// Check if signal is in set
    pub fn contains(&self, signal: Signal) -> bool {
        (self.mask & (1 << signal.number())) != 0
    }

    /// Check if set is empty
    pub fn is_empty(&self) -> bool {
        self.mask == 0
    }

    /// Clear all signals
    pub fn clear(&mut self) {
        self.mask = 0;
    }

    /// Get next pending signal (lowest number)
    pub fn next_signal(&self) -> Option<Signal> {
        for num in [2, 9, 10, 11, 12, 13, 15, 17] {
            if let Some(sig) = Signal::from_number(num) {
                if self.contains(sig) {
                    return Some(sig);
                }
            }
        }
        None
    }
}

impl Default for SignalSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_from_number() {
        assert_eq!(Signal::from_number(2), Some(Signal::SIGINT));
        assert_eq!(Signal::from_number(9), Some(Signal::SIGKILL));
        assert_eq!(Signal::from_number(15), Some(Signal::SIGTERM));
        assert_eq!(Signal::from_number(99), None);
    }

    #[test]
    fn test_signal_number() {
        assert_eq!(Signal::SIGINT.number(), 2);
        assert_eq!(Signal::SIGKILL.number(), 9);
    }

    #[test]
    fn test_signal_catchable() {
        assert!(Signal::SIGINT.is_catchable());
        assert!(Signal::SIGTERM.is_catchable());
        assert!(!Signal::SIGKILL.is_catchable());
    }

    #[test]
    fn test_signal_default_action() {
        assert_eq!(Signal::SIGINT.default_action(), SignalAction::Terminate);
        assert_eq!(Signal::SIGCHLD.default_action(), SignalAction::Ignore);
    }

    #[test]
    fn test_signal_set_basic() {
        let mut set = SignalSet::new();
        assert!(set.is_empty());

        set.add(Signal::SIGINT);
        assert!(set.contains(Signal::SIGINT));
        assert!(!set.contains(Signal::SIGTERM));

        set.add(Signal::SIGTERM);
        assert!(set.contains(Signal::SIGTERM));

        set.remove(Signal::SIGINT);
        assert!(!set.contains(Signal::SIGINT));
        assert!(set.contains(Signal::SIGTERM));
    }

    #[test]
    fn test_signal_set_clear() {
        let mut set = SignalSet::new();
        set.add(Signal::SIGINT);
        set.add(Signal::SIGTERM);
        assert!(!set.is_empty());

        set.clear();
        assert!(set.is_empty());
    }

    #[test]
    fn test_signal_set_next() {
        let mut set = SignalSet::new();
        assert_eq!(set.next_signal(), None);

        set.add(Signal::SIGTERM);
        set.add(Signal::SIGINT);
        // Should return lowest number first (SIGINT=2)
        assert_eq!(set.next_signal(), Some(Signal::SIGINT));
    }
}
