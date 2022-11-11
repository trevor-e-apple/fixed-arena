/// Errors that may be returned from an attempt to allocate from an arena
#[derive(Debug, PartialEq)]
pub enum AllocError {
    AtCapacity,
}
