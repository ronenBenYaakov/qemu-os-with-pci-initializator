use x86_64::{structures::paging::{FrameAllocator, OffsetPageTable}, PhysAddr};

use crate::{ehci::{reset_ehci_controller, PciDevice}, memory::{map_mmio_region, BootInfoFrameAllocator}, println};

pub fn init_pci(ehci: PciDevice, mut frame_allocator: BootInfoFrameAllocator, mut mapper: OffsetPageTable<'static>) {
    println!("Found EHCI Controller:");
        println!(" Bus: {}, Slot: {}, Func: {}", ehci.bus, ehci.slot, ehci.func);
        println!(" Vendor ID: {:#06x}, Device ID: {:#06x}", ehci.vendor_id, ehci.device_id);
        println!(" BAR0: {:#010x}", ehci.bar0);

        let phys_addr = PhysAddr::new(ehci.bar0 as u64);
        match map_mmio_region(phys_addr, &mut mapper, &mut frame_allocator) {
            Ok(virt_addr) => {
                println!("Mapped EHCI MMIO at virtual address: {:p}", virt_addr.as_ptr::<u8>());
                
                // SAFETY: virt_addr maps to phys_addr MMIO region
                unsafe {
                    // Read the first 32-bit register (offset 0) of EHCI MMIO to test
                    let reg_ptr = virt_addr.as_ptr::<u32>();
                    let reg_val = reg_ptr.read_volatile();
                    println!("EHCI MMIO first register value: {:#010x}", reg_val);

                    reset_ehci_controller(virt_addr.as_mut_ptr());

                }
            }
            Err(e) => println!("Failed to map EHCI MMIO: {}", e),
        }
}