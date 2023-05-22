extern crate idmanager;

use idmanager::IdManager;
use idmanager::ReusePolicy::ReuseSlow;

pub fn main() {
    let manager = IdManager::<u8>::new_limited_range(ReuseSlow, 10, 210);

    manager.mark_interval_as_used(201, 210);
    manager.mark_value_as_used(11);

    assert_eq!(manager.dump(), "[10], [12,200]");

    {
        let id1 = manager.allocate_id();

        let expected_id1: u8 = 10;

        assert_eq!(id1.value(), &expected_id1);

        assert_eq!(manager.dump(), "[12,200]");

        {
            let mut id2 = manager.allocate_id();

            let expected_id2: u8 = 12;

            assert_eq!(id2.value(), &expected_id2);

            assert_eq!(manager.dump(), "[13,200]");

            id2.release();

            {
                let id3 = manager.allocate_id();

                let expected_id: u8 = 13;

                assert_eq!(id3.value(), &expected_id);

                assert_eq!(manager.dump(), "[14,200]");
            }
        }

        assert_eq!(manager.dump(), "[13,200]");
    }

    assert_eq!(manager.dump(), "[10], [13,200]");
}
