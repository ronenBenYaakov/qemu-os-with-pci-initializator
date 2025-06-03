cargo bootimage
qemu-system-x86_64 -m 512M -drive format=raw,file=C:\Users\ronen\Downloads\blog_os\target\x86_64-blog_os\debug\bootimage-blog_os.bin -device usb-ehci,id=ehci -serial stdio
