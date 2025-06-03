use crate::{print, println};
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{
    stream::{Stream, StreamExt},
    task::AtomicWaker,
};
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if queue.push(scancode).is_err() {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE.try_get().expect("scancode queue not initialized");

        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}

pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    use alloc::vec::Vec;

    let mut buffer = Vec::with_capacity(1024);
    print!("> "); // Initial prompt

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(c) => {
                        match c {
                            '\n' => {
                                println!();
                                if let Ok(input_str) = core::str::from_utf8(&buffer) {
                                    println!("You typed: {}", input_str);
                                } else {
                                    println!("[Invalid UTF-8 input]");
                                }
                                buffer.clear();
                                print!("> ");
                            }
                            '\x08' => { // Backspace
                                if !buffer.is_empty() {
                                    buffer.pop();
                                    // Move cursor back, erase char, move cursor back
                                    print!("\u{8} \u{8}");
                                }
                            }
                            c if !c.is_control() => {
                                if buffer.len() < 1024 {
                                    buffer.push(c as u8);
                                    print!("{}", c);
                                }
                            }
                            _ => {
                                // Ignore other control chars
                            }
                        }
                    }
                    DecodedKey::RawKey(_) => {
                        // Don't print raw keys at all
                    }
                }
            }
        }
    }
}
