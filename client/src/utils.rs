use std::sync::RwLock;

static UUID: RwLock<Option<String>> = RwLock::new(None);

pub fn set_uuid(new: &str) {
  UUID.write().unwrap().replace(new.to_owned());
}

pub fn get_uuid() -> Option<String> {
  return UUID.read().unwrap().clone();
}
