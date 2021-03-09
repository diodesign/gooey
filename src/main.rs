/* diosix user-interface system service main loop
 *
 * (c) Chris Williams, 2020-2021.
 *
 * See LICENSE for usage and copying.
 */

/* we're running in supervisor mode, not userland */
#![no_std]
#![no_main]
#![allow(unused_must_use)]

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::collections::btree_map::BTreeMap;
use alloc::collections::btree_map::Entry::{Occupied, Vacant};

/* this will initialize and manage our supervisor-level environment.
   it also provides access to the underlying environment and macros
   such as println!() for outputting formatted text to the user */
#[macro_use]
extern crate supervisor;
use supervisor::sbi;

#[macro_use]
extern crate lazy_static;
extern crate spin;
use spin::Mutex;

/* keep track of the output of capsules and the hypervisor */
lazy_static!
{
    static ref INIT_DONE: Mutex<bool> = Mutex::new(false);
    static ref CAPSULE_STDOUT: Mutex<BTreeMap<usize, String>> = Mutex::new(BTreeMap::new());
    static ref HV_STDOUT: Mutex<String> = Mutex::new(String::new());
}

/* colors and escape codes taken from https://en.wikipedia.org/wiki/ANSI_escape_code */
const COLOR_RED: usize = 31; /* Red is used for the hypervisor output */
const COLORS: &'static [usize] = &[ 32, 33, 34, 35, 36, 37 ]; /* Green, Yellow, Blue, Magenta, Cyan, White */
const ANSI_ESC_SEQ: &'static str = "\x1b[";

fn gather_system_output()
{
    /* check for any output bytes waiting for us from other capsules */
    loop
    {
        match sbi::capsule_getc()
        {
            Ok(cap_char) => if cap_char.get_char() as i8 != -1
            {
                match CAPSULE_STDOUT.lock().entry(cap_char.get_capsule_id())
                {
                    Occupied(mut s) => s.get_mut().push(cap_char.get_char()),
                    Vacant(v) =>
                    {
                        let mut s = String::new();
                        s.push(cap_char.get_char());
                        v.insert(s);
                    }
                }
            }
            else
            {
                break;  
            },
            Err(e) =>
            {
                println!("Unexpected error {:?} while gathering capsule output", e);
                sbi::exit(1);
            }
        }
    }

    /* check for output bytes waiting for us from the hypervisor */
    loop
    {
        match sbi::hypervisor_getc()
        {
            Ok(c) => if c as i8 != -1
            {
                HV_STDOUT.lock().push(c);
            }
            else
            {
                break;
            },
            Err(e) =>
            {
                println!("Unexpected error {:?} while gathering hypervisor output", e);
                sbi::exit(1);
            }
        }
    }
}

/* primitives for controlling the ANSI terminal */
fn clear_screen()
{
    print!("{}1;1H", ANSI_ESC_SEQ); /* return to cursor pos 1,1 */
    print!("{}0J", ANSI_ESC_SEQ); /* clear screen from cursor */
}

fn clear_attributes()
{
    print!("{}0m", ANSI_ESC_SEQ);
}

fn set_fg_color(color: usize)
{
    print!("{}1;{}m", ANSI_ESC_SEQ, color.to_string());
}

/* ui update loop */
fn draw_ui()
{
    /* start afresh */
    clear_attributes();

    /* output text by the hypervisor */
    let mut hypervisor_output = HV_STDOUT.lock();
    if hypervisor_output.len() > 0
    {
        set_fg_color(COLOR_RED);
        for c in hypervisor_output.drain(..)
        {
            print!("{}", c.to_string());
        }
    }

    /* and also the capsules */
    for (capsule_id, buffer) in CAPSULE_STDOUT.lock().iter_mut()
    {
        if buffer.len() > 0
        {
            set_fg_color(COLORS[capsule_id % COLORS.len()]);
            for character in buffer.drain(..)
            {
                print!("{}", character.to_string());
            }
        }
    }
}

fn get_user_input()
{
    if let Ok(character) = sbi::getc()
    {
        sbi::capsule_putc(character, 1);
    }
}

/* application start point
   => tid = scheduler thread ID, starting from 0 and counting up */
#[no_mangle]
pub extern "C" fn main(tid: usize)
{
    /* thread ID 0 will do all the initialization and then unlock other threads
       when init is complete */
    if tid == 0
    {
        if sbi::register_service(sbi::DiosixServiceType::ConsoleInterface).is_ok() == true
        {
            clear_screen();
            println!("System console user interface registered");
            *(INIT_DONE.lock()) = true;
        }
        else
        {
            println!("Failed to register system console user interface");
            sbi::exit(1);
        }
    }
    else
    {
        /* wait until initialization is done */
        while *(INIT_DONE.lock()) != true {}
    }

    loop
    {
        /* all threads can check capsules and the underlying hypervisor for output */
        gather_system_output();

        /* only one thread (tid 0) will handle the user interface */
        if tid == 0
        {
            get_user_input();
            draw_ui();
        }
    }
}
