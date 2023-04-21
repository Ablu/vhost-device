#![no_main]

use libfuzzer_sys::fuzz_target;
use std::{
    convert::TryInto,
    io::{Cursor, Read},
};
use vhost_device_scsi::{
    scsi::emulation::{
        block_device::{BlockDevice, BlockDeviceBackend},
        target::EmulatedTarget,
    },
    vhu_scsi::VhostUserScsiBackend,
};
use vhost_user_backend::{VhostUserBackendMut, VringRwLock, VringT};
use vm_memory::{Bytes, GuestAddress, GuestAddressSpace, GuestMemoryAtomic, GuestMemoryMmap};

#[derive(Debug, arbitrary::Arbitrary)]
struct FuzzingInput {
    mem: [u8; 0x300],
}

struct FuzzingBackend {
    buf: [u8; 0x1000],
}

impl FuzzingBackend {
    fn new() -> Self {
        Self { buf: [0; 0x1000] }
    }
}

impl BlockDeviceBackend for FuzzingBackend {
    fn read_exact_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<()> {
        let mut c = Cursor::new(&self.buf);
        c.set_position(offset);
        c.read_exact(buf)
    }

    fn size_in_blocks(&self) -> std::io::Result<u64> {
        let buf_len: u64 = self
            .buf
            .len()
            .try_into()
            .expect("buf len should fit into u64");
        Ok((buf_len / u64::from(self.block_size()))
            .try_into()
            .expect("fake size_in_blocks should fit u64"))
    }

    fn block_size(&self) -> u32 {
        512
    }

    fn sync(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fuzz_target!(|input: FuzzingInput| {
    let mem = GuestMemoryAtomic::new(
        GuestMemoryMmap::<()>::from_ranges(&[(GuestAddress(0), input.mem.len())]).unwrap(),
    );
    mem.memory()
        .write_slice(&input.mem, GuestAddress(0))
        .unwrap();

    let vring = VringRwLock::new(mem.clone(), 8).unwrap();
    vring.set_queue_info(0, 0x80, 0x8c).unwrap();
    vring.set_queue_ready(true);

    let mut backend = VhostUserScsiBackend::new();
    let mut fake_target = Box::new(EmulatedTarget::new());
    let fake_lun = Box::new(BlockDevice::new(FuzzingBackend::new()));
    fake_target.add_lun(fake_lun);
    backend.add_target(fake_target);

    backend.update_memory(mem.clone()).unwrap();
    let _ = backend.process_request_queue(&vring);
});
