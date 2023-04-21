// SPDX-License-Identifier: Apache-2.0 or BSD-3-Clause

use vhost_device_scsi::*;

fn main() -> backend::Result<()> {
    backend::scsi_init()
}
