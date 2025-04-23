use std::time::Duration;
use mini_rt::{MiniRuntime, spawn, sleep};

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

fn main() {
    let mut rt = MiniRuntime::new();
    rt.block_on(async {
        let _ = spawn(async {
            println!("Runtime started...");
        }).await;

        task_one().await;
        task_two().await;
    })
}
mod mini_rt;