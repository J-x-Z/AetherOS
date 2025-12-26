use crate::backend::{Backend, ExitReason};
use std::sync::Arc;
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

pub type ProcessId = u64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

pub struct Process {
    pub id: ProcessId,
    pub backend: Arc<dyn Backend>,
    pub state: ProcessState,
}

pub struct Scheduler {
    processes: VecDeque<Process>,
    next_pid: ProcessId,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            processes: VecDeque::new(),
            next_pid: 1,
        }
    }

    pub fn spawn(&mut self, backend: Arc<dyn Backend>) -> ProcessId {
        let pid = self.next_pid;
        self.next_pid += 1;
        self.processes.push_back(Process {
            id: pid,
            backend,
            state: ProcessState::Ready,
        });
        println!("[Aether::Scheduler] Spawned Process {}", pid);
        pid
    }

    pub fn run(&mut self) {
        println!("[Aether::Scheduler] Starting Round-Robin Scheduler...");
        
        loop {
            if let Some(mut process) = self.processes.pop_front() {
                match process.state {
                    ProcessState::Ready | ProcessState::Running => {
                        process.state = ProcessState::Running;
                        
                        // Execute one step of the guest
                        let exit_reason = process.backend.step();
                        
                        match exit_reason {
                            ExitReason::Yield => {
                                process.state = ProcessState::Ready;
                                self.processes.push_back(process);
                            }
                            ExitReason::Halt => {
                                println!("[Aether::Scheduler] Process {} Terminated", process.id);
                                process.state = ProcessState::Terminated;
                            }
                            ExitReason::Io(_) | ExitReason::Mmio(_) | ExitReason::Unknown => {
                                // Treat as implicit yield (IO handled by step() side-effects for now)
                                process.state = ProcessState::Ready;
                                self.processes.push_back(process);
                            }
                        }
                    }
                    ProcessState::Terminated => {
                        // Drop process
                    }
                    ProcessState::Blocked => {
                        // TODO: Check unblock conditions
                        self.processes.push_back(process);
                    }
                }
            } else {
                // No processes ready, sleep briefly to avoid burning host CPU
                // In real OS this would be WFI/HLT
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}
