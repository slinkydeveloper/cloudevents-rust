use crate::Event;

pub enum HttpEvent {
    Binary(Option<Event>),
    Structured(Option<Event>),
    Batch(Vec<Event>)
}
