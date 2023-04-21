use std::io::{stdin, stdout, Write};

use virtio_bindings::{virtio_ring::VRING_DESC_F_WRITE, virtio_scsi::virtio_scsi_cmd_req};
use virtio_queue::{mock::MockSplitQueue, Descriptor};
use vm_memory::{
    Address, ByteValued, Bytes, GuestAddress, GuestAddressSpace, GuestMemoryAtomic, GuestMemoryMmap,
};

#[derive(Debug, Default, Clone, Copy)]
#[repr(transparent)]
pub(crate) struct VirtioScsiCmdReq(pub virtio_scsi_cmd_req);
/// SAFETY: struct is a transparent wrapper around the request
/// which can be read from a byte array
unsafe impl ByteValued for VirtioScsiCmdReq {}

fn create_lun_specifier(target: u8, lun: u16) -> [u8; 8] {
    let lun = lun.to_le_bytes();

    [
        0x1,
        target,
        lun[0] | 0b0100_0000,
        lun[1],
        0x0,
        0x0,
        0x0,
        0x0,
    ]
}

fn main() {
    let mem = GuestMemoryAtomic::new(
        GuestMemoryMmap::<()>::from_ranges(&[(GuestAddress(0), 0x300)]).unwrap(),
    );
    // The `build_desc_chain` function will populate the `NEXT` related flags and field.
    let v = vec![
        Descriptor::new(0x100, 0x100, 0, 0), // request
        Descriptor::new(0x200, 0x100, VRING_DESC_F_WRITE as u16, 0), // response
    ];

    let req = VirtioScsiCmdReq(virtio_scsi_cmd_req {
        lun: create_lun_specifier(0, 0),
        tag: 0,
        task_attr: 0,
        prio: 0,
        crn: 0,
        cdb: [0; 32],
    });

    mem.memory()
        .write_obj(req, GuestAddress(0x100))
        .expect("writing to succeed");

    let mem_handle = mem.memory();

    let queue = MockSplitQueue::new(&*mem_handle, 8);

    queue.build_desc_chain(&v).unwrap();

    // Put the descriptor index 0 in the first available ring position.
    mem.memory()
        .write_obj(0u16, queue.avail_addr().unchecked_add(4))
        .unwrap();

    // Set `avail_idx` to 1.
    mem.memory()
        .write_obj(1u16, queue.avail_addr().unchecked_add(2))
        .unwrap();

    let mut buf = [0; 0x300];
    mem.memory().read_slice(&mut buf, GuestAddress(0)).unwrap();
    dbg!(
        queue.desc_table_addr(),
        queue.avail_addr(),
        queue.used_addr()
    );
    // dbg!(buf);
    // stdout().write_all(&mut buf).unwrap();
    // stdout().flush().unwrap();
}
