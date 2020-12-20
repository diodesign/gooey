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

use core::fmt::Write;

/* this will initialize and manage our supervisor-level environment */
extern crate supervisor;
use supervisor::stdio::Stdout;

#[no_mangle]
pub extern "C" fn main()
{
    let mut stdout = Stdout::new();
    write!(&mut stdout, "hello world! I'm a system service");
}
