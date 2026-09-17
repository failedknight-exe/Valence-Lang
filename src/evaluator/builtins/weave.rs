// src/evaluator/builtins/weave.rs - Parallel SMT Thread Dispatcher

use super::Evaluator;
use crate::evaluator::value::Value;
use crate::parser::Node;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

impl Evaluator {
    /// Executes a block of nodes simultaneously across physical OS threads.
    pub fn eval_weave(&mut self, count_node: Node, body: Vec<Node>) -> Value {
        let max_threads = match self.eval(count_node) {
            Value::Integer(n) if n > 0 => n as usize,
            _ => return Value::Error("weave() expects a positive integer for core count. A zero or a negative count is not parallelism; it's just a dramatic mistake.".into()),
        };

        println!("\x1b[35m[WEAVE] Dispatching {} tasks across {} vCPU cores...\x1b[0m", body.len(), max_threads);
        let start_time = Instant::now();

        let (tx, rx) = mpsc::channel();
        let total_tasks = body.len();

        // Dispatch every statement in the block as a parallel task
        for (task_idx, node) in body.into_iter().enumerate() {
            let tx_clone = tx.clone();
            
            // Fork evaluator state so each thread has its own memory isolate!
            let mut core_evaluator = self.fork();

            thread::spawn(move || {
                let result = core_evaluator.eval(node);
                tx_clone.send((task_idx, result)).unwrap();
            });
        }

        // Drop original transmitter so the receiver unblocks when threads finish
        drop(tx);

        let mut completed_tasks = 0;
        let mut results = vec![Value::Null; total_tasks];

        // Wait for all threads to report back their result!
        while let Ok((task_idx, res)) = rx.recv() {
            results[task_idx] = res;
            completed_tasks += 1;
        }

        let elapsed = start_time.elapsed();
        println!("\x1b[32m  [WEAVE] {} parallel tasks completed in {:.2?}!\x1b[0m", completed_tasks, elapsed);

        // Return array of results matching the original statement order!
        Value::array(results)
    }
}