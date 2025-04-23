use std::time::Duration;
use mini_rt::{spawn, sleep, yield_now, MiniRuntime};

async fn task_one() {
    println!("task one: start");
    sleep(Duration::from_secs(1)).await;
    println!("task one: done");
}

async fn task_two() {
    println!("task two: start");
    sleep(Duration::from_secs(2)).await;
    println!("task two: done");
}

async fn long_task() {
    for i in 1..=3 {
        println!("long task: step {}", i);
        yield_now().await;
    }
}

fn main() {
    let mut rt = MiniRuntime::new();
    rt.block_on(async {
        let t1 = spawn(async {
            println!("Runtime started...");
        });

        let h1 = spawn(task_one());
        let h2 = spawn(task_two());
        let h3 = spawn(long_task());

        join_all!(t1, h1, h2, h3);
    });
}

mod mini_rt;
