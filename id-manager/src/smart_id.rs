use std::sync::Mutex;

use crate::id_manager::IdManager;
use crate::id_type::IdType;

pub struct SmartId<'a, T: IdType> {
    manager: &'a Mutex<IdManager<T>>,
    id: T,
    we_own_id: bool,
}

impl<'a, T: IdType> SmartId<'a, T> {
    pub fn new(manager: &'a Mutex<IdManager<T>>) -> Self {
        let mut locked = manager.lock().unwrap();

        if !locked.can_allocate()
        {
            panic!("No Ids available")
        }

        let id = locked.allocate();

        SmartId { manager, id, we_own_id: true }
    }

    pub fn release(&mut self) -> T {
        let _locked = self.manager.lock().unwrap();

        self.we_own_id = false;

        self.id
    }

    pub fn value(&self) -> &T {
        &self.id
    }
}

impl<'a, T: IdType> Drop for SmartId<'a, T> {
    fn drop(&mut self) {
        let mut locked = self.manager.lock().unwrap();

        if self.we_own_id
        {
            locked.free(self.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::reuse_policy::ReusePolicy::ReuseSlow;

    use super::*;

    #[test]
    fn test_create_one_smart_id() {
        let manager = Mutex::new(IdManager::<u8>::new(ReuseSlow));

        assert_eq!(manager.lock().unwrap().dump(), "[0,255]");

        {
            let id1 = SmartId::<u8>::new(&manager);

            let expected_id: u8 = 0;

            assert_eq!(id1.value(), &expected_id);

            assert_eq!(manager.lock().unwrap().dump(), "[1,255]");
        }

        assert_eq!(manager.lock().unwrap().dump(), "[0,255]");
    }

    #[test]
    fn test_create_multiple_smart_ids() {
        let manager = Mutex::new(IdManager::<u8>::new(ReuseSlow));

        assert_eq!(manager.lock().unwrap().dump(), "[0,255]");

        {
            let id1 = SmartId::<u8>::new(&manager);

            let expected_id1: u8 = 0;

            assert_eq!(id1.value(), &expected_id1);

            assert_eq!(manager.lock().unwrap().dump(), "[1,255]");

            {
                let mut id2 = SmartId::new(&manager);

                let expected_id2: u8 = 1;

                assert_eq!(id2.value(), &expected_id2);

                assert_eq!(manager.lock().unwrap().dump(), "[2,255]");

                id2.release();

                {
                    let id3 = SmartId::new(&manager);

                    let expected_id: u8 = 2;

                    assert_eq!(id3.value(), &expected_id);

                    assert_eq!(manager.lock().unwrap().dump(), "[3,255]");
                }
            }

            assert_eq!(manager.lock().unwrap().dump(), "[2,255]");
        }

        assert_eq!(manager.lock().unwrap().dump(), "[0], [2,255]");
    }

    #[test]
    fn test_release() {
        let manager = Mutex::new(IdManager::<u8>::new(ReuseSlow));

        assert_eq!(manager.lock().unwrap().dump(), "[0,255]");

        {
            let mut id1 = SmartId::<u8>::new(&manager);

            assert_eq!(manager.lock().unwrap().dump(), "[1,255]");

            id1.release();

            assert_eq!(manager.lock().unwrap().dump(), "[1,255]");
        }

        assert_eq!(manager.lock().unwrap().dump(), "[1,255]");
    }
}
