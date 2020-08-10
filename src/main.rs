#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(tinyos::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate tinyos;
extern crate x86_64;
extern crate bootloader;
extern crate alloc;

use alloc::boxed::Box;
use core::panic::PanicInfo;
//use tinyos::memory::translate_addr;
use tinyos::{memory, allocator, println, task::{Task, executor::Executor, keyboard}};

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    tinyos::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    tinyos::test_panic_handler(info)
}

use bootloader::{BootInfo, entry_point};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    //use tinyos::memory::active_level_4_table;
    //use x86_64::structures::paging::PageTable;
    use x86_64::{structures::paging::Page, VirtAddr};
    println!("The numbers are {} and {}", 42, 1.0/3.0);

    tinyos::init(); // new
    // invoke a breakpoint exception
    //x86_64::instructions::interrupts::int3(); // new

    // trigger a page fault
    /*
    unsafe {
        *(0xdeadbeef as *mut u64) = 42;
    };
    */


     #[cfg(test)]
    test_main();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe {memory::init(phys_mem_offset)};
    /*
    let addresses = [
        0xb8000,
        0x201008,
        0x0100_0020_1a10,
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = unsafe { translate_addr(virt, phys_mem_offset) };
        println!("{:?} -> {:?}", virt, phys);
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }*/

    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);
    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    let _x = Box::new(41);

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses())); // new
    executor.run();

    println!("It did not crash!");
    tinyos::hlt_loop();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
