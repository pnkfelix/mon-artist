```rust
use Scene;
```

All rendering engines expose an instance of the object-safe `Render` trait.

In addition, there is also a structured rendering trait that may provide
a structured format specific to that renderer.
```rust
pub trait Render {
    fn render(&self, scene: &Scene) -> String;
}

pub trait RenderS {
    type Out;
    fn render_s(&self, scene: &Scene) -> Self::Out;
}
```

The `render::ascii` module handles rendering (back) to ASCII art.

It is meant to be 1. simple code, 2. a way to test the correctness
of the parsing (via round-trip comparison), and 3. a way to generate
debug output.

```rust
pub mod ascii {

}
```

The bulk of the rendering targets SVG, held in the `render::svg` module.

```rust
pub mod svg;
```
