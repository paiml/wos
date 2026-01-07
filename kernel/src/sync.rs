//! Synchronization Primitives
//!
//! POSIX-like synchronization primitives (Mutex, Semaphore, Condition Variables)
//! for coordinating concurrent processes in WOS.

use crate::state::ProcessId;
use im::{HashMap, Vector};
use serde::{Deserialize, Serialize};

/// Mutex identifier
pub type MutexId = u32;

/// Semaphore identifier
pub type SemaphoreId = u32;

/// Mutex state
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutexState {
    /// Mutex is unlocked
    Unlocked,
    /// Mutex is locked by a process
    Locked(ProcessId),
}

/// Mutex for mutual exclusion
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mutex {
    /// Unique mutex identifier
    pub id: MutexId,
    /// Current state (locked or unlocked)
    pub state: MutexState,
    /// Queue of processes waiting to acquire this mutex
    pub wait_queue: Vector<ProcessId>,
}

impl Mutex {
    /// Create a new mutex
    pub fn new(id: MutexId) -> Self {
        Self {
            id,
            state: MutexState::Unlocked,
            wait_queue: Vector::new(),
        }
    }

    /// Check if mutex is locked
    pub fn is_locked(&self) -> bool {
        matches!(self.state, MutexState::Locked(_))
    }

    /// Get the process that owns this mutex (if locked)
    pub fn owner(&self) -> Option<ProcessId> {
        match self.state {
            MutexState::Locked(pid) => Some(pid),
            MutexState::Unlocked => None,
        }
    }

    /// Try to lock the mutex
    ///
    /// Returns Ok(()) if lock acquired, Err if already locked
    pub fn try_lock(&mut self, pid: ProcessId) -> Result<(), &'static str> {
        match self.state {
            MutexState::Unlocked => {
                self.state = MutexState::Locked(pid);
                Ok(())
            }
            MutexState::Locked(_) => Err("Mutex already locked"),
        }
    }

    /// Lock the mutex (blocking)
    ///
    /// If already locked, add process to wait queue
    pub fn lock(&mut self, pid: ProcessId) -> MutexLockResult {
        match self.state {
            MutexState::Unlocked => {
                self.state = MutexState::Locked(pid);
                MutexLockResult::Acquired
            }
            MutexState::Locked(owner) if owner == pid => {
                // Deadlock detection: process trying to lock mutex it already owns
                MutexLockResult::Deadlock
            }
            MutexState::Locked(_) => {
                // Add to wait queue
                if !self.wait_queue.contains(&pid) {
                    self.wait_queue.push_back(pid);
                }
                MutexLockResult::Blocked
            }
        }
    }

    /// Unlock the mutex
    ///
    /// Returns the next process to wake up from wait queue
    pub fn unlock(&mut self, pid: ProcessId) -> Result<Option<ProcessId>, &'static str> {
        match self.state {
            MutexState::Locked(owner) if owner == pid => {
                // Unlock and wake up next waiting process
                if let Some(next_pid) = self.wait_queue.pop_front() {
                    self.state = MutexState::Locked(next_pid);
                    Ok(Some(next_pid))
                } else {
                    self.state = MutexState::Unlocked;
                    Ok(None)
                }
            }
            MutexState::Locked(_) => Err("Cannot unlock mutex owned by another process"),
            MutexState::Unlocked => Err("Mutex is not locked"),
        }
    }
}

/// Result of mutex lock operation
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutexLockResult {
    /// Lock successfully acquired
    Acquired,
    /// Process blocked and added to wait queue
    Blocked,
    /// Deadlock detected (process trying to lock mutex it already owns)
    Deadlock,
}

/// Semaphore for counting resources
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Semaphore {
    /// Unique semaphore identifier
    pub id: SemaphoreId,
    /// Current count (number of available resources)
    pub count: i32,
    /// Maximum count (for bounded semaphores)
    pub max_count: Option<i32>,
    /// Queue of processes waiting on this semaphore
    pub wait_queue: Vector<ProcessId>,
}

impl Semaphore {
    /// Create a new semaphore with initial count
    pub fn new(id: SemaphoreId, initial_count: i32) -> Self {
        Self {
            id,
            count: initial_count,
            max_count: None,
            wait_queue: Vector::new(),
        }
    }

    /// Create a bounded semaphore with max count
    pub fn new_bounded(id: SemaphoreId, initial_count: i32, max_count: i32) -> Self {
        Self {
            id,
            count: initial_count,
            max_count: Some(max_count),
            wait_queue: Vector::new(),
        }
    }

    /// Wait (decrement) - P operation
    ///
    /// If count > 0, decrement and return Acquired
    /// If count = 0, add process to wait queue and return Blocked
    pub fn wait(&mut self, pid: ProcessId) -> SemaphoreWaitResult {
        if self.count > 0 {
            self.count -= 1;
            SemaphoreWaitResult::Acquired
        } else {
            // Add to wait queue
            if !self.wait_queue.contains(&pid) {
                self.wait_queue.push_back(pid);
            }
            SemaphoreWaitResult::Blocked
        }
    }

    /// Post (increment) - V operation
    ///
    /// Increment count and wake up next waiting process if any
    /// Returns error if bounded semaphore would exceed max_count
    pub fn post(&mut self) -> Result<Option<ProcessId>, &'static str> {
        // Check bounded limit
        if let Some(max) = self.max_count {
            if self.count >= max {
                return Err("Semaphore count would exceed maximum");
            }
        }

        // Wake up next waiting process if any
        if let Some(next_pid) = self.wait_queue.pop_front() {
            // Don't increment count - directly transfer resource to waiting process
            Ok(Some(next_pid))
        } else {
            // No waiting processes - increment count
            self.count += 1;
            Ok(None)
        }
    }

    /// Try wait (non-blocking) - try to decrement without blocking
    pub fn try_wait(&mut self) -> Result<(), &'static str> {
        if self.count > 0 {
            self.count -= 1;
            Ok(())
        } else {
            Err("Semaphore count is zero")
        }
    }
}

/// Result of semaphore wait operation
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemaphoreWaitResult {
    /// Resource acquired (count decremented)
    Acquired,
    /// Process blocked and added to wait queue
    Blocked,
}

/// Synchronization primitive manager
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SyncManager {
    /// All mutexes in the system
    pub mutexes: HashMap<MutexId, Mutex>,
    /// All semaphores in the system
    pub semaphores: HashMap<SemaphoreId, Semaphore>,
    /// Next available mutex ID
    pub next_mutex_id: MutexId,
    /// Next available semaphore ID
    pub next_semaphore_id: SemaphoreId,
}

impl SyncManager {
    /// Create a new synchronization manager
    pub fn new() -> Self {
        Self {
            mutexes: HashMap::new(),
            semaphores: HashMap::new(),
            next_mutex_id: 1,
            next_semaphore_id: 1,
        }
    }

    /// Create a new mutex
    pub fn create_mutex(&mut self) -> MutexId {
        let id = self.next_mutex_id;
        self.next_mutex_id += 1;
        let mutex = Mutex::new(id);
        self.mutexes.insert(id, mutex);
        id
    }

    /// Create a new semaphore with initial count
    pub fn create_semaphore(&mut self, initial_count: i32) -> SemaphoreId {
        let id = self.next_semaphore_id;
        self.next_semaphore_id += 1;
        let sem = Semaphore::new(id, initial_count);
        self.semaphores.insert(id, sem);
        id
    }

    /// Create a bounded semaphore
    pub fn create_bounded_semaphore(&mut self, initial_count: i32, max_count: i32) -> SemaphoreId {
        let id = self.next_semaphore_id;
        self.next_semaphore_id += 1;
        let sem = Semaphore::new_bounded(id, initial_count, max_count);
        self.semaphores.insert(id, sem);
        id
    }

    /// Destroy a mutex
    pub fn destroy_mutex(&mut self, id: MutexId) -> Result<(), &'static str> {
        if let Some(mutex) = self.mutexes.get(&id) {
            if mutex.is_locked() {
                return Err("Cannot destroy locked mutex");
            }
            if !mutex.wait_queue.is_empty() {
                return Err("Cannot destroy mutex with waiting processes");
            }
        }
        self.mutexes.remove(&id);
        Ok(())
    }

    /// Destroy a semaphore
    pub fn destroy_semaphore(&mut self, id: SemaphoreId) -> Result<(), &'static str> {
        if let Some(sem) = self.semaphores.get(&id) {
            if !sem.wait_queue.is_empty() {
                return Err("Cannot destroy semaphore with waiting processes");
            }
        }
        self.semaphores.remove(&id);
        Ok(())
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // WOS-014: Mutex Tests
    mod mutex_tests {
        use super::*;

        #[test]
        fn test_mutex_creation() {
            let mutex = Mutex::new(1);
            assert_eq!(mutex.id, 1);
            assert_eq!(mutex.state, MutexState::Unlocked);
            assert!(mutex.wait_queue.is_empty());
            assert!(!mutex.is_locked());
        }

        #[test]
        fn test_mutex_lock_unlock() {
            let mut mutex = Mutex::new(1);

            // Lock the mutex
            let result = mutex.lock(100);
            assert_eq!(result, MutexLockResult::Acquired);
            assert!(mutex.is_locked());
            assert_eq!(mutex.owner(), Some(100));

            // Unlock the mutex
            let woken = mutex.unlock(100).unwrap();
            assert_eq!(woken, None);
            assert!(!mutex.is_locked());
            assert_eq!(mutex.owner(), None);
        }

        #[test]
        fn test_mutex_blocking() {
            let mut mutex = Mutex::new(1);

            // Process 100 locks mutex
            assert_eq!(mutex.lock(100), MutexLockResult::Acquired);

            // Process 200 tries to lock - should block
            assert_eq!(mutex.lock(200), MutexLockResult::Blocked);
            assert_eq!(mutex.wait_queue.len(), 1);
            assert!(mutex.wait_queue.contains(&200));
        }

        #[test]
        fn test_mutex_deadlock_detection() {
            let mut mutex = Mutex::new(1);

            // Process 100 locks mutex
            assert_eq!(mutex.lock(100), MutexLockResult::Acquired);

            // Same process tries to lock again - deadlock
            assert_eq!(mutex.lock(100), MutexLockResult::Deadlock);
        }

        #[test]
        fn test_mutex_wake_on_unlock() {
            let mut mutex = Mutex::new(1);

            // Process 100 locks
            mutex.lock(100);

            // Process 200 and 300 block
            mutex.lock(200);
            mutex.lock(300);

            // Process 100 unlocks - should wake process 200
            let woken = mutex.unlock(100).unwrap();
            assert_eq!(woken, Some(200));
            assert_eq!(mutex.owner(), Some(200));
            assert_eq!(mutex.wait_queue.len(), 1); // Process 300 still waiting
        }

        #[test]
        fn test_mutex_cannot_unlock_by_non_owner() {
            let mut mutex = Mutex::new(1);

            mutex.lock(100);

            // Process 200 tries to unlock - should fail
            let result = mutex.unlock(200);
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err(),
                "Cannot unlock mutex owned by another process"
            );
        }

        #[test]
        fn test_mutex_try_lock() {
            let mut mutex = Mutex::new(1);

            // Try lock succeeds when unlocked
            assert!(mutex.try_lock(100).is_ok());

            // Try lock fails when locked
            assert!(mutex.try_lock(200).is_err());
        }
    }

    // WOS-014: Semaphore Tests
    mod semaphore_tests {
        use super::*;

        #[test]
        fn test_semaphore_creation() {
            let sem = Semaphore::new(1, 3);
            assert_eq!(sem.id, 1);
            assert_eq!(sem.count, 3);
            assert_eq!(sem.max_count, None);
            assert!(sem.wait_queue.is_empty());
        }

        #[test]
        fn test_semaphore_bounded_creation() {
            let sem = Semaphore::new_bounded(1, 2, 5);
            assert_eq!(sem.count, 2);
            assert_eq!(sem.max_count, Some(5));
        }

        #[test]
        fn test_semaphore_wait_post() {
            let mut sem = Semaphore::new(1, 2);

            // First wait - should succeed
            assert_eq!(sem.wait(100), SemaphoreWaitResult::Acquired);
            assert_eq!(sem.count, 1);

            // Second wait - should succeed
            assert_eq!(sem.wait(200), SemaphoreWaitResult::Acquired);
            assert_eq!(sem.count, 0);

            // Third wait - should block
            assert_eq!(sem.wait(300), SemaphoreWaitResult::Blocked);
            assert_eq!(sem.count, 0);
            assert_eq!(sem.wait_queue.len(), 1);

            // Post - should wake process 300
            let woken = sem.post().unwrap();
            assert_eq!(woken, Some(300));
            assert_eq!(sem.count, 0); // Still 0 because resource transferred
        }

        #[test]
        fn test_semaphore_try_wait() {
            let mut sem = Semaphore::new(1, 1);

            // Try wait succeeds
            assert!(sem.try_wait().is_ok());
            assert_eq!(sem.count, 0);

            // Try wait fails when count is 0
            assert!(sem.try_wait().is_err());
        }

        #[test]
        fn test_bounded_semaphore_max_count() {
            let mut sem = Semaphore::new_bounded(1, 5, 5);

            // Post should fail (already at max)
            let result = sem.post();
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "Semaphore count would exceed maximum");
        }

        #[test]
        fn test_semaphore_multiple_waiters() {
            let mut sem = Semaphore::new(1, 0);

            // Three processes wait
            assert_eq!(sem.wait(100), SemaphoreWaitResult::Blocked);
            assert_eq!(sem.wait(200), SemaphoreWaitResult::Blocked);
            assert_eq!(sem.wait(300), SemaphoreWaitResult::Blocked);
            assert_eq!(sem.wait_queue.len(), 3);

            // First post wakes process 100
            assert_eq!(sem.post().unwrap(), Some(100));
            assert_eq!(sem.wait_queue.len(), 2);

            // Second post wakes process 200
            assert_eq!(sem.post().unwrap(), Some(200));
            assert_eq!(sem.wait_queue.len(), 1);
        }
    }

    // WOS-014: SyncManager Tests
    mod sync_manager_tests {
        use super::*;

        #[test]
        fn test_sync_manager_creation() {
            let manager = SyncManager::new();
            assert!(manager.mutexes.is_empty());
            assert!(manager.semaphores.is_empty());
            assert_eq!(manager.next_mutex_id, 1);
            assert_eq!(manager.next_semaphore_id, 1);
        }

        #[test]
        fn test_sync_manager_create_mutex() {
            let mut manager = SyncManager::new();

            let id1 = manager.create_mutex();
            assert_eq!(id1, 1);
            assert!(manager.mutexes.contains_key(&1));

            let id2 = manager.create_mutex();
            assert_eq!(id2, 2);
            assert!(manager.mutexes.contains_key(&2));
        }

        #[test]
        fn test_sync_manager_create_semaphore() {
            let mut manager = SyncManager::new();

            let id1 = manager.create_semaphore(3);
            assert_eq!(id1, 1);
            assert_eq!(manager.semaphores.get(&1).unwrap().count, 3);

            let id2 = manager.create_bounded_semaphore(5, 10);
            assert_eq!(id2, 2);
            assert_eq!(manager.semaphores.get(&2).unwrap().count, 5);
            assert_eq!(manager.semaphores.get(&2).unwrap().max_count, Some(10));
        }

        #[test]
        fn test_sync_manager_destroy_mutex() {
            let mut manager = SyncManager::new();
            let id = manager.create_mutex();

            // Can destroy unlocked mutex
            assert!(manager.destroy_mutex(id).is_ok());
            assert!(!manager.mutexes.contains_key(&id));
        }

        #[test]
        fn test_sync_manager_cannot_destroy_locked_mutex() {
            let mut manager = SyncManager::new();
            let id = manager.create_mutex();

            // Lock the mutex
            manager.mutexes.get_mut(&id).unwrap().lock(100);

            // Cannot destroy locked mutex
            let result = manager.destroy_mutex(id);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "Cannot destroy locked mutex");
        }

        #[test]
        fn test_sync_manager_destroy_semaphore() {
            let mut manager = SyncManager::new();
            let id = manager.create_semaphore(1);

            // Can destroy semaphore with no waiters
            assert!(manager.destroy_semaphore(id).is_ok());
            assert!(!manager.semaphores.contains_key(&id));
        }
    }
}
