/* diosix user-interface system service main loop
 *
 * (c) Chris Williams, 2020.
 *
 * See LICENSE for usage and copying.
 */

/* we're running in supervisor mode, not userland */
#![no_std]
#![no_main]
#![allow(unused_must_use)]

#[macro_use]
extern crate lazy_static;
extern crate spin;

lazy_static!
{
    static ref STDOUT_LOCK: spin::Mutex<bool> = spin::Mutex::new(false);
}

/* this will initialize and manage our supervisor-level environment */
extern crate supervisor;
use supervisor::stdio::Stdout;
use core::fmt::Write;

#[no_mangle]
pub extern "C" fn main()
{
    let mut stdout = Stdout::new();

    {
        let mut lock = STDOUT_LOCK.lock();
        *lock = true;
        write!(&mut stdout, "hello world! I'm a system service\n");
    }
}
