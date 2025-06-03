use x86_64::{instructions::interrupts::enable, structures::paging::{FrameAllocator, OffsetPageTable}, PhysAddr};

use crate::{ehci::{ reset_ehci_controller, PciDevice}, memory::{map_mmio_region, BootInfoFrameAllocator}, println};

pub fn init_pci(
    ehci: PciDevice,
    mut frame_allocator: BootInfoFrameAllocator,
    mut mapper: OffsetPageTable<'static>,
) {
    println!("Found EHCI Controller:");
    println!(" Bus: {}, Slot: {}, Func: {}", ehci.bus, ehci.slot, ehci.func);
    println!(" Vendor ID: {:#06x}, Device ID: {:#06x}", ehci.vendor_id, ehci.device_id);
    println!(" BAR0: {:#010x}", ehci.bar0);

    let phys_addr = PhysAddr::new(ehci.bar0 as u64);
    match map_mmio_region(phys_addr, &mut mapper, &mut frame_allocator) {
        Ok(virt_addr) => {
            println!("Mapped EHCI MMIO at virtual address: {:p}", virt_addr.as_ptr::<u8>());

            unsafe {
                // EHCI MMIO base pointer
                let base_ptr = virt_addr.as_mut_ptr::<u8>();

                // Run EHCI test before reset
                ehci_probe_test(base_ptr);

                reset_ehci_controller(base_ptr);
            }
        }
        Err(e) => println!("\nFailed to map EHCI MMIO: {}", e),
    }
}

unsafe fn ehci_probe_test(mmio_base: *mut u8) {
    use core::ptr::read_volatile;

    const CAPLENGTH_OFFSET: usize = 0x00;
    const HCIVERSION_OFFSET: usize = 0x02;
    const HCSPARAMS_OFFSET: usize = 0x04;
    const HCCPARAMS_OFFSET: usize = 0x08;

    let caplength = read_volatile(mmio_base.add(CAPLENGTH_OFFSET) as *const u8);
    let hciversion = read_volatile(mmio_base.add(HCIVERSION_OFFSET) as *const u16);
    let hcsparams = read_volatile(mmio_base.add(HCSPARAMS_OFFSET) as *const u32);
    let hccparams = read_volatile(mmio_base.add(HCCPARAMS_OFFSET) as *const u32);

    println!("EHCI Probe Test:");
    println!(" CAPLENGTH: {:#04x}", caplength);
    println!(" HCIVERSION: {:#06x}", hciversion);
    println!(" HCSPARAMS: {:#010x}", hcsparams);
    println!(" HCCPARAMS: {:#010x}", hccparams);
}
