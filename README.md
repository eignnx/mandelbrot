# mandelbrot
A Mandelbrot set renderer written in Rust. Based heavily on chapter 2 of ["Programming Rust"](http://shop.oreilly.com/product/0636920040385.do) by Jim Blandy and Jason Orendorff. (And by heavily, I mean with some slight cosmetic differences.)

## Multithreading
As in the book, I'm using [`crossbeam`](https://crates.io/crates/crossbeam) to render the image in parallel.

Through trial and error, it seems like 10 threads works the fastest on my machine. A 4000x3000 image renders in about 5.8 seconds with one thread and 1.9 seconds with 10. That's strange because my computer has a [core i5 processor](https://everymac.com/systems/apple/macbook_pro/specs/macbook-pro-core-i5-2.5-13-mid-2012-unibody-usb3-specs.html) which has only two cores. Hmmm... I clearly don't entirely understand what a thread is.

## Running on the Tinker Board
I also ran some tests on my Tinker Board. Here is a plot of real execution time vs number of threads. The same image was generated for these tests as was generated in the previous tests.

![Plot of execution time, seconds vs. number of threads](https://github.com/eignnx/mandelbrot/Threads-vs-Execution-Time-Mandelbrot-Tinker-Board-Plot.png)
