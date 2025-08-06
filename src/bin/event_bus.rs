use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

pub struct EventBus<T: Clone + Send> {
    chamber : tokio::sync::Mutex<HashMap<u32, VecDeque<T>>>,
    count: AtomicU32,
    dead_ids: std::sync::Mutex<Vec<u32>>
}

impl <T: Clone + Send> EventBus<T> {
    pub fn new() -> Self {
        Self{
            chamber: tokio::sync::Mutex::new(HashMap::new()),
            count: AtomicU32::new(0),
            dead_ids: std::sync::Mutex::new(Vec::new())
        }
    }

    pub async fn subscribe(&self) -> EventHandle<T> {
        let mut chamber = self.chamber.lock().await;
        let id = self.count.fetch_add(1, Ordering::SeqCst);
        chamber.insert(id, VecDeque::new());

        EventHandle {
            id,
            event_bus: Arc::new(self)
        }
    }

    pub fn unsubscribe(&self, id: u32) {
        self.dead_ids.lock().unwrap().push(id)
    }
    pub async fn poll(&self, id: u32) -> Option<T> {
        let mut chamber = self.chamber.lock().await;
        let queue = chamber.get_mut(&id).unwrap();
        queue.pop_front()
    }

    pub async fn send(&self, event: T) {
        let mut chamber = self.chamber.lock().await;
        for (_, value) in chamber.iter_mut() {
            value.push_back(event.clone())
        }
    }
}

pub struct EventHandle<'a, T: Clone + Send> {
    pub id: u32,
    event_bus: Arc<&'a EventBus<T>>,
}
impl <'a, T: Clone + Send> EventHandle<'a, T> {
    pub async fn poll(&self) -> Option<T> {
        self.event_bus.poll(self.id).await
    }
}

impl <'a, T: Clone + Send> Drop for EventHandle<'a, T> {
    fn drop(&mut self) {
        self.event_bus.unsubscribe(self.id)
    }
}

async fn consume_event(event_bus: Arc<EventBus<f32>>) {
    let handle = event_bus.subscribe().await;
    loop {
        let event = handle.poll().await;
        match event {
            Some(event) => {
                println!("id: {}, value: {}", handle.id, event);
                if event == 3.0 {
                    break;
                }
            }
            None => {}
        }
    }
}

async fn garbage_collector(event_bus: Arc<EventBus<f32>>) {
    loop {
        let mut chamber = event_bus.chamber.lock().await;
        let dead_ids = event_bus.dead_ids.lock().unwrap().clone();
        event_bus.dead_ids.lock().unwrap().clear();
        for id in dead_ids.iter() {
            println!("garbage_collector.remove id: {}", id);
            chamber.remove(id);
        }
        drop(chamber);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

#[tokio::main]
async fn main() {
    let event_bus = Arc::new(EventBus::<f32>::new());
    let bus1 = event_bus.clone();
    let bus2 = event_bus.clone();
    let gb_bus_ref = event_bus.clone();

    let _gb = tokio::task::spawn(async {
        garbage_collector(gb_bus_ref).await
    });
    let one = tokio::task::spawn(async {
        consume_event(bus1).await
    });
    let two = tokio::task::spawn(async {
        consume_event(bus2).await
    });

    std::thread::sleep(std::time::Duration::from_secs(1));
    event_bus.send(1.0).await;
    event_bus.send(2.0).await;
    event_bus.send(3.0).await;

    let _ = one.await;
    let _ = two.await;
    println!("chamber: {:?}", event_bus.chamber.lock().await);
    println!("dead_ids: {:?}", event_bus.dead_ids.lock());
    std::thread::sleep(std::time::Duration::from_secs(3));
    // drop後
    println!("chamber: {:?}", event_bus.chamber.lock().await);
    println!("dead_ids: {:?}", event_bus.dead_ids.lock());

}