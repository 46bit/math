# `math`

A mathematical programming language that [@46bit](https://46b.it) made as a hobby. Here's an example program that divides two inputs:

```
inputs a, b;
x = a / b;
outputs x;
```

Here's an example program that computes [Fibonacci numbers](https://en.wikipedia.org/wiki/Fibonacci_number):

```
inputs n;
fib(n) = match n {
  0 => 0,
  1 => 1,
  _ => fib(n - 1) + fib(n - 2),
};
m = fib(n);
outputs m;
```

## Usage

Build the programs by running `make build`. You'll need a Rust nightly build (it's been tested with `rustc 1.24.0-nightly (4a7c072fa 2017-12-25)`, amongst others.)

### Interpreter

* Calculate `5 ÷ 2`: `cat examples/div.math | target/debug/mathi 5 2`
* Calculate the 3rd [Fibonacci number](https://en.wikipedia.org/wiki/Fibonacci_number): `cat examples/fib.math | target/debug/mathi 3`

### Compiler

You'll need LLVM 5.0.0.

* Calculate `5 ÷ 2`:
  1. `target/debug/mathc examples/div.math div.out`
  2. `./div.out 5 2`
* Calculate the 3rd [Fibonacci number](https://en.wikipedia.org/wiki/Fibonacci_number):
  1. `target/debug/mathc examples/fib.math fib.out`
  2. `./fib.out 3`
