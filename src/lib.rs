/*!
    Library to get the last 20 Hansard Bound Volumes of the UK Parliament

    Usage:

    ```no_run
        extern crate hansard;

        use hansard::retrieve;

        fn main(){
            // call retrieve to start the download of the bound volumes
            retrieve::retrieve();
        }
    ```

*/

#![deny(missing_docs)]

extern crate atom_syndication;
extern crate hyper;
extern crate zip;
#[macro_use]
extern crate log;
extern crate tokio_core;
extern crate futures;

/// Module for retrieving the Hansard bound volumes
pub mod retrieve;
