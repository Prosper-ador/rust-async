/*use std::time::Duration;
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
}*/
use std::time::Duration;
use mini_rt::{spawn, sleep};

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
    let mut rt = mini_rt::MiniRuntime::new();
    rt.block_on(async {
        let _ = spawn(async {
            println!("Runtime started...");
        });

        let h1 = spawn(task_one());
        let h2 = spawn(task_two());

        join_all!(h1, h2); // now they run concurrently
    });
}

mod mini_rt;
