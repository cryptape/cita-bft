// CITA
// Copyright 2016-2017 Cryptape Technologies LLC.

// This program is free software: you can redistribute it
// and/or modify it under the terms of the GNU General Public
// License as published by the Free Software Foundation,
// either version 3 of the License, or (at your option) any
// later version.

// This program is distributed in the hope that it will be
// useful, but WITHOUT ANY WARRANTY; without even the implied
// warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
// PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use core::cita_bft::Step;
use std::sync::mpsc::{Receiver, Sender};
// use std::thread;
use std::time::{Duration, Instant};
use threadpool::ThreadPool;

extern crate min_max_heap;

const THREAD_POOL_NUM: usize = 10;

#[derive(Debug, Clone)]
pub struct TimeoutInfo {
    pub timeval: Instant,
    pub height: usize,
    pub round: usize,
    pub step: Step,
}
//unsafe impl ::std::marker::Sync for TimeoutInfo {}

pub struct WaitTimer {
    timer_seter: Receiver<TimeoutInfo>,
    timer_notify: Sender<TimeoutInfo>,
    thpool: ThreadPool,
}

//unsafe impl ::std::marker::Sync for WaitTimer {}

impl WaitTimer {
    pub fn new(ts: Sender<TimeoutInfo>, rs: Receiver<TimeoutInfo>) -> WaitTimer {
        let pool = ThreadPool::new(THREAD_POOL_NUM);
        WaitTimer {
            timer_notify: ts,
            timer_seter: rs,
            thpool: pool,
        }
    }

    pub fn start(&self) {
        let innersetter = &self.timer_seter;
        let mut timer_heap = min_max_heap::MinMaxHeap::new();

        loop {
            let timeout = if !timer_heap.is_empty() {
                timer_heap.peek_min().cloned().unwrap() - Instant::now()
            } else {
                Duration::from_millis(0)
            };

            let set_time = innersetter.recv_timeout(timeout);

            if set_time.is_ok() {
                timer_heap.push(set_time.unwrap().timeval);
            }

            if !timer_heap.is_empty() {
                let now = Instant::now();
                let notify = self.timer_notify.clone();
                while now >= timer_heap.peek_min().cloned().unwrap() {
                    notify.send(innersetter.recv().unwrap()).unwrap();
                    timer_heap.pop_min();
                }
            }
        }
    }
}
