# mandelbrot
A Mandelbrot set renderer written in Rust. Based heavily on chapter 1 of "Programming Rust" by Jim Blandy and Jason Orendorff. (And by heavily, I mean with some slight cosmetic differences.)

Multithreading
-------------------
As in the book, I'm using [`crossbeam`](https://crates.io/crates/crossbeam) to render the image in parallel.

Through trial and error, it seems like 10 threads works the fastest on my machine. A 4000x3000 image renders in about 1.9 seconds. That's strange because my computer has a [core i5 processor](https://everymac.com/systems/apple/macbook_pro/specs/macbook-pro-core-i5-2.5-13-mid-2012-unibody-usb3-specs.html) which has only two cores. Hmmm... I clearly don't entirely understand what a thread is.
