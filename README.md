# Visual Physics

Copyright (C) 2026 Jacob Farnsworth

Visual Physics is a physics simulation program designed as an educational tool. Visual Physics is mainly built to run simulations of orbital systems, such as the Earth-Moon system.

Simulations are defined in sim definition files (JSON-based format). In a sim definition, any number of bodies can be specified, with predefined mass, absolute position, velocity, as well as graphical data such as name and color. Additionally, the sim definition can specify settings for the physics simulation, such as time scale, or custom gravitational constant.

Visual Physics can smoothly simulate systems with an impressive degree of accuracy:

* The simulator uses velocity Verlet integration, which is a reasonably accurate symplectic second-order integrator. Symplectic means that the algorithm conserves the total energy of the system, meaning that the simulator tends to produce orbits which are relatively stable, even with time steps which are quite fast.
* The program is multithreaded; physics and rendering take place in separate threads. As such, the physics simulation is not constrained to run one tick per frame, but can tick many very small time steps per frame, which greatly improves the accuracy of the simulation.

## Using Visual Physics

Keyboard shortcuts:

* Left-click on a body to select it.
* Right-click while dragging the mouse to pan the camera.
* Scroll to zoom in and out.
* F to focus. In this mode, the camera will follow the selected body.
* O to open the settings window.

## Contributing

Visual Physics is written in Rust. If you wish to contribute to Visual Physics, you need to install the Rust compiler.

[Install Rust here](https://rust-lang.org/tools/install/)

## License

Visual Physics is licensed under the terms of the GNU GPLv2. See the file LICENSE for the full license text.
