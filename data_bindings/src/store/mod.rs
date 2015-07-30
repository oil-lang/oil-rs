
pub use self::cast::Cast;
pub use self::cast::AssignFromCast;
pub use self::value::StoreValue;
pub use self::value::StoreValueStatic;

// This is where Store default implementation lives
mod impls;

// Those two modules must be implemented
// in sync.
mod value;
mod cast;
