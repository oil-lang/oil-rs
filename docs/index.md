# Welcome to Oil !

Oil is a [GUI](https://en.wikipedia.org/wiki/GUI) library for Game developers.
It is written in Rust. To learn more about Rust, please visit the [official web site](http://rust-lang.org).

Oil abstracts the user interface related code from your core game application.
In that respect, it works similarly to [QML](http://en.wikipedia.org/wiki/QML).
So how it is different ?

Instead of using a scripting language, Oil provides you:

 * A markup language which use two-ways property bindings to attach values from your models to each view's
   elements. Those bindings are explicit and use a [mustache](https://mustache.github.io/)-like syntax.

 * A style language similar to CSS with additional states that cover both animation needs
   and classic interaction changes.

 * A dependency description language to load your assets: image, fonts, shaders and so on.

With that in mind, we hope Oil will help you have a clear separation between your game logic,
and the user interface.

<img class="welcome-logo" src="img/logo.svg" />

> Note:
>
> If you think that some of the above statements are currently wrong, you're probably right.
> What are you waiting for to contribute to the project then ? :)

## Getting started

To use oil in your code, you must add it to your Cargo.toml file in
the dependency section:

```toml
[dependencies]
oil = "*"
```

Oil currently relies on [glium](http://tomaka.github.io/glium/) and
[glutin](http://tomaka.github.io/glutin/glutin/index.html) to do the rendering.

While this will be more flexible in the future, both these library are pretty good
to start with and have a decent wrapper around OpenGL.

The only thing needed to know about glutin/glium to use the Oil is how to create
a window, which as simple as doing writing such as:

```rust
extern crate glutin;
extern crate glium;

use glium::DisplayBuild;

// ...

let display = glutin::WindowBuilder::new()
    .build_glium()
    .unwrap();
```

## Writing your first interface

> TODO
