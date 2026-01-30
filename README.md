# GYOSHO (行書)

**Gyosho** is a modern, high-performance graphics research ecosystem built in **Rust**. It aims to solve the fragmentation of GPU shading languages by establishing a "Write Once, Run Anywhere" pipeline rooted in strict type safety, ergonomic design, and native performance.

The project is named after *Gyōsho* (semi-cursive script), a style of Japanese calligraphy that bridges the gap between the rigid strictness of block script (*Kaisho*) and the flowing freedom of cursive (*Sōsho*)—perfectly symbolizing our goal to balance **Rust's reliability** with **Pythonic readability**.

## 🏗️ Architecture

Gyosho is a monorepo workspace containing the three pillars of the **Sumi** ecosystem:

### 1. `libsumi` (The Ink)
* **Role:** The Mathematical Kernel.
* **Description:** A lightweight, `no_std` compatible linear algebra library designed for graphics. It ensures 1:1 mathematical parity between the CPU (Rust) and the GPU (MSL/GLSL/SPIR-V), guaranteeing that a `dot()` product on the processor yields the exact same result as on the shader core.
* **Status:** Rust Port Active (replacing legacy C++ `sumi.h`).

### 2. `sumic` (The Brush)
* **Role:** The Compiler.
* **Description:** A standalone reference compiler for **S2L** (Sumi Shader Language). It parses high-level, ergonomic shader code and transpiles it into optimized native shading languages (Metal MSL, SPIR-V, HLSL).
* **Philosophy:** "Interslavic for GPUs." A unified language that is mutually intelligible by all modern graphics backends.
* **Status:** Porting from Swift to Rust.

### 3. `hanga` (The Print)
* **Role:** The Runtime & Testbed.
* **Description:** A lightweight 2D rendering engine and windowing environment. *Hanga* serves as the immediate runtime for S2L, allowing users to write shaders and see them "printed" to the screen in real-time without setting up complex C++ boilerplates.
* **Status:** In Development (Rust/wgpu).

---

## 🔮 The Sumi Shader Language (S2L)

Gyosho is the home of **S2L**, a new shading language designed with the following tenets:
* **Pythonic Ergonomics:** Clean syntax, optional semicolons, and minimal punctuation.
* **Go-like Simplicity:** Composition over inheritance; structural typing; explicit, readable signatures.
* **Rust Reliability:** Compile-time static analysis for memory and resources. No garbage collection.

## 🚀 Getting Started

### Prerequisites
* [Rust](https://www.rust-lang.org/) (latest stable)
* `cargo`

### Building the Workspace

```bash
# Clone the repository
git clone [https://github.com/crux161/gyosho.git](https://github.com/crux161/gyosho.git)
cd gyosho

# Build all crates (libsumi, sumic, hanga)
cargo build --workspace --release

# Run the test suite
cargo test --workspace
