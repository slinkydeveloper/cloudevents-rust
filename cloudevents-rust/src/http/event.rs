use crate::Event;

pub enum HttpEvent {
    Binary(Event),
    Structured(Event),
    Batch(Vec<Event>)
}
