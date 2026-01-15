use crossbeam_deque::{Steal, Worker};

#[test]
fn test_work_steal_order_lifo() {
    let worker: Worker<i32> = Worker::new_lifo();
    worker.push(10);
    worker.push(20);
    worker.push(30);
    assert_eq!(worker.pop(), Some(30));
    let stealer = worker.stealer();
    assert_eq!(stealer.steal(), Steal::Success(10));
    worker.push(40);
    worker.push(50);
    assert_eq!(stealer.steal(), Steal::Success(20));
    assert_eq!(worker.pop(), Some(50));
}

#[test]
fn test_work_steal_order_fifo() {
    let worker: Worker<i32> = Worker::new_fifo();
    worker.push(10);
    worker.push(20);
    worker.push(30);
    assert_eq!(worker.pop(), Some(10));
    let stealer = worker.stealer();
    worker.push(40);
    assert_eq!(stealer.steal(), Steal::Success(20));
    worker.push(50);
    assert_eq!(stealer.steal(), Steal::Success(30));
    assert_eq!(worker.pop(), Some(40));
}
