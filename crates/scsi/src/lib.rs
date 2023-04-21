macro_rules! fuzzer_visible_mod {
    ($module:tt) => {
        pub mod $module;
    };
}
fuzzer_visible_mod!(backend);
fuzzer_visible_mod!(scsi);
fuzzer_visible_mod!(vhu_scsi);
fuzzer_visible_mod!(virtio);
