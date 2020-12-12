/* diosix user-interface system service main loop
 *
 * (c) Chris Williams, 2020.
 *
 * See LICENSE for usage and copying.
 */

 /* we're running in supervisor mode, not userland */
#![no_std]
#![no_main]

/* provide a framework for unit testing */
#![feature(custom_test_frameworks)]
#![test_runner(crate::run_tests)]
#![reexport_test_harness_main = "gooeytests"] /* entry point for tests */

/* this will initialize and manage our supervisor-level environment */
#[macro_use]
extern crate supervisor;

#[no_mangle]
pub extern "C" fn main()
{
    println!("hello world! I'm a system service");
}

#[cfg(test)]
fn run_tests(unit_tests: &[&dyn Fn()])
{
    /* run each test one by one */
    for test in unit_tests
    {
        test();
    }

    /* exit cleanly once tests are complete */
    // platform::test::end(Ok(0));
}

#[test_case]
fn test_assertion()
{
    assert_eq!(42, 42);
}