use alloc::sync::Arc;
use spin::RwLock;

pub struct Promise<T: Copy> {
  value: Arc<RwLock<Option<T>>>,
}

impl<T: Copy> Promise<T> {
  pub fn new() -> Promise<T> {
    Promise {
      value: Arc::new(RwLock::new(None)),
    }
  }

  pub fn resolve(&self, value: T) {
    let mut lock = self.value.write();
    *lock = Some(value);
  }

  pub fn get_value(&self) -> PromiseValue<T> {
    PromiseValue::new(&self.value)
  }
}

pub struct PromiseValue<T: Copy>(Arc<RwLock<Option<T>>>);

impl<T: Copy> PromiseValue<T> {
  pub fn new(value: &Arc<RwLock<Option<T>>>) -> PromiseValue<T> {
    PromiseValue(Arc::clone(value))
  }

  pub fn is_resolved(&self) -> bool {
    match self.0.try_read() {
      Some(value) => match *value {
        Some(_) => true,
        None => false,
      },
      None => false
    }
  }

  pub fn get_result(&self) -> Option<T> {
    match self.0.try_read() {
      Some(value) => *value,
      None => None,
    }
  }
}
