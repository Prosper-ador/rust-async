# Async Runtime
**This project implements a minimal asynchronous runtime in Rust, inspired by how async runtimes like tokio or async-std work. It provides basic functionality to execute asynchronous tasks, spawn new tasks, and manage concurrency.**

## Features
Custom Runtime: A simple runtime to execute asynchronous tasks.
Task Spawning: Spawn tasks using the spawn function.
Concurrency: Run multiple tasks concurrently using the join_all! macro.
Sleep Functionality: Simulate delays with the sleep function.
Yielding: Yield control back to the runtime with yield_now.
Macros:
mini_rt!: Simplify runtime initialization.
join_all!: Run multiple tasks concurrently.