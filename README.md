# Ascia
Ascia is a lightweight hobby framework designed for creating 3D computer graphics applications that operate within a terminal environment.

## Examples
To run the examples, use the following command:

`cargo run --release [feature flags] --example [example_name] [window_width] [window_height]`

For example:

`cargo run --features wgpu,termios-controller --release --example example1 140 40`

`cargo run --release --example example2 140 40`

`cargo run --features termios-controller --release --example cube 140 40`

`cargo run --features wgpu,termios-controller --release --example teapot 140 40`

`cargo run --features wgpu,termios-controller --release --example particle_test_board 140 40`

## Gallery

<img src="gif/example1.gif" width="400">
<img src="gif/example2.gif" width="400">
<img src="gif/teapot.gif" width="400">