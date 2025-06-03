
//define I/O ports
const PCI_CONFIG_ADDR: u16 = 0xcf8;
const PCI_CONFIG_DATA: u16 = 0xcfc;

#[inline(always)]
pub unsafe fn outl(port: u16, val: u32) {
    core::arch::asm!("out dx, eax", in("dx") port, in("eax") val);
}

#[inline(always)]
pub unsafe fn inl(port: u16) -> u32 {
    let value: u32;
    core::arch::asm!("in eax, dx", in("dx") port, out("eax") value);
    value
}

fn pci_config_address(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let lbus = bus as u32;
    let lslot = slot as u32;
    let lfunc = func as u32;
    let loffset = (offset & 0xFC) as u32;

    0x8000_0000 | (lbus << 16) | (lslot << 11) | (lfunc << 8) | loffset
}

pub fn pci_read_config_dword(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let address = pci_config_address(bus, slot, func, offset);
    unsafe {
        outl(PCI_CONFIG_ADDR, address);
        inl(PCI_CONFIG_DATA)
    }
}

pub fn pci_read_config_word(bus: u8, slot: u8, func: u8, offset: u8) -> u16 {
    let dword = pci_read_config_dword(bus, slot, func, offset & 0xFC);
    let shift = ((offset & 2) * 8) as u32;
    ((dword >> shift) & 0xFFFF) as u16
}

pub fn pci_read_config_byte(bus: u8, slot: u8, func: u8, offset: u8) -> u8 {
    let dword = pci_read_config_dword(bus, slot, func, offset & 0xFC);
    let shift = ((offset & 3) * 8) as u32;
    ((dword >> shift) & 0xFF) as u8
}

const EHCI_CLASS_CODE: u8 = 0x0C;  // Serial Bus Controller
const EHCI_SUBCLASS_CODE: u8 = 0x03; // USB Controller
const EHCI_PROG_IF: u8 = 0x20;  // EHCI specific

/// Represents a basic PCI device header
pub struct PciDevice {
    pub bus: u8,
    pub slot: u8,
    pub func: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub bar0: u32,
}

pub fn find_ehci_controller() -> Option<PciDevice> {
    for bus in 0..=255 {
        for slot in 0..32 {
            for func in 0..8 {
                let vendor = pci_read_config_word(bus, slot, func, 0x00);
                if vendor == 0xFFFF {
                    continue;
                }

                let class = pci_read_config_byte(bus, slot, func, 0x0B);
                let subclass = pci_read_config_byte(bus, slot, func, 0x0A);
                let prog_if = pci_read_config_byte(bus, slot, func, 0x09);

                if class == EHCI_CLASS_CODE && subclass == EHCI_SUBCLASS_CODE && prog_if == EHCI_PROG_IF {
                    let device = PciDevice {
                        bus,
                        slot,
                        func,
                        vendor_id: vendor,
                        device_id: pci_read_config_word(bus, slot, func, 0x02),
                        class_code: class,
                        subclass,
                        prog_if,
                        bar0: pci_read_config_dword(bus, slot, func, 0x10),
                    };
                    return Some(device);
                }

                // If not multi-function device, skip remaining funcs
                if func == 0 {
                    let header_type = pci_read_config_byte(bus, slot, func, 0x0E);
                    if header_type & 0x80 == 0 {
                        break;
                    }
                }
            }
        }
    }
    None
}

use alloc::boxed::Box;
use x86_64::{
    structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, PhysFrame}, PhysAddr, VirtAddr
};

use crate::println;

const EHCI_MMIO_VADDR: u64 = 0xdead_beef_0000;

fn map_mmio_region(
    phys_addr: PhysAddr,
    mapper: &mut impl Mapper<x86_64::structures::paging::Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<x86_64::structures::paging::Size4KiB>,
) -> Result<VirtAddr, &'static str> {
    let start_page = Page::containing_address(VirtAddr::new(EHCI_MMIO_VADDR));
    let phys_frame = PhysFrame::containing_address(phys_addr);
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE;

    unsafe {
        mapper
            .map_to(start_page, phys_frame, flags, frame_allocator)
            .map_err(|_| "map_to failed")?
            .flush();
    }

    Ok(start_page.start_address())
}

/// EHCI USB Command Register offset from BAR0
const USB_CMD_OFFSET: usize = 0x20;
const USB_CMD_RESET: u32 = 1 << 1;

pub unsafe fn reset_ehci_controller(mmio_base: *mut u8) {
    let cmd_reg = mmio_base.add(USB_CMD_OFFSET) as *mut u32;

    // Write reset bit
    cmd_reg.write_volatile(USB_CMD_RESET);

    // Wait for reset bit to clear
    loop {
        let val = cmd_reg.read_volatile();
        if (val & USB_CMD_RESET) == 0 {
            break;
        }
    }
    println!("EHCI controller reset complete.");
}

use x86_64::structures::paging::{ Size4KiB, OffsetPageTable};
use crate::memory::{BootInfoFrameAllocator};

const EHCI_USBCMD_OFFSET: usize = 0x20;
const EHCI_USBSTS_OFFSET: usize = 0x24;
const EHCI_PERIODICLISTBASE_OFFSET: usize = 0x18;

