use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;
use std::collections::VecDeque;

// --------------------------------------------------------------------
// 1. Event trait
// --------------------------------------------------------------------
pub trait Event: Any + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
}
impl<T: Any + Send + Sync + 'static> Event for T {
    fn as_any(&self) -> &dyn Any { self }
}

// --------------------------------------------------------------------
// 2. Handler ID
// --------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandlerId(pub usize);

// --------------------------------------------------------------------
// 3. Internal trait for stored handlers
// --------------------------------------------------------------------
trait ErasedHandler: Send + Sync {
    fn handle(&self, ev: &dyn Event);
    fn id(&self) -> usize;
    fn box_clone(&self) -> Box<dyn ErasedHandler>;
}
impl Clone for Box<dyn ErasedHandler> {
    fn clone(&self) -> Self { self.box_clone() }
}

// Concrete struct that wraps the user’s closure
struct HandlerImpl<F> {
    id: usize,
    func: Arc<F>,
}
impl<F> ErasedHandler for HandlerImpl<F>
where
    F: Fn(&dyn Event) + Send + Sync + 'static
{
    fn handle(&self, ev: &dyn Event) {
        (self.func)(ev);
    }
    fn id(&self) -> usize {
        self.id
    }
    fn box_clone(&self) -> Box<dyn ErasedHandler> {
        Box::new(Self { id: self.id, func: self.func.clone() })
    }
}

// --------------------------------------------------------------------
// 4. The global EventBus
// --------------------------------------------------------------------
pub struct EventBus {
    handlers: HashMap<TypeId, Vec<Box<dyn ErasedHandler>>>,
    queue: VecDeque<Box<dyn Event>>,
    next_id: usize,
}
impl EventBus {
    fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            queue: VecDeque::new(),
            next_id: 0,
        }
    }
    fn dispatch<E: Event + 'static>(&mut self, ev: E) {
        self.queue.push_back(Box::new(ev));
    }
    fn process(&mut self) {
        let mut current = std::mem::take(&mut self.queue);
        while let Some(ev) = current.pop_front() {
            let tid = ev.as_any().type_id();
            if let Some(list) = self.handlers.get(&tid) {
                for h in list.iter() {
                    h.handle(&*ev);
                }
            }
        }
    }
    fn subscribe<E: Event + 'static, F>(&mut self, closure: F) -> HandlerId
    where
        F: Fn(&E) + Send + Sync + 'static
    {
        let id = HandlerId(self.next_id);
        self.next_id += 1;

        let tid = TypeId::of::<E>();
        let erased = HandlerImpl {
            id: id.0,
            func: Arc::new(move |ev: &dyn Event| {
                if let Some(real) = ev.as_any().downcast_ref::<E>() {
                    closure(real);
                }
            }),
        };

        self.handlers.entry(tid).or_default()
            .push(Box::new(erased));
        id
    }
    fn unsubscribe<E: Event + 'static>(&mut self, handler_id: HandlerId) {
        let tid = TypeId::of::<E>();
        if let Some(list) = self.handlers.get_mut(&tid) {
            list.retain(|h| h.id() != handler_id.0);
        }
    }
}

// A global OnceLock for the bus
static GLOBAL_BUS: OnceLock<Arc<Mutex<EventBus>>> = OnceLock::new();

fn global_bus() -> Arc<Mutex<EventBus>> {
    GLOBAL_BUS.get_or_init(|| {
        Arc::new(Mutex::new(EventBus::new()))
    }).clone()
}

// --------------------------------------------------------------------
// 5. Public API
// --------------------------------------------------------------------
pub fn dispatch<E: Event + 'static>(ev: E) {
    if let Ok(mut bus) = global_bus().lock() {
        bus.dispatch(ev);
    }
}
pub fn process_events() {
    if let Ok(mut bus) = global_bus().lock() {
        bus.process();
    }
}
pub fn subscribe<E: Event + 'static, F>(f: F) -> HandlerId
where
    F: Fn(&E) + Send + Sync + 'static
{
    if let Ok(mut bus) = global_bus().lock() {
        bus.subscribe(f)
    } else {
        HandlerId(0)
    }
}
pub fn unsubscribe<E: Event + 'static>(handler_id: HandlerId) {
    if let Ok(mut bus) = global_bus().lock() {
        bus.unsubscribe::<E>(handler_id);
    }
}
