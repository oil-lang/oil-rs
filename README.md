# oil-rs [![Build Status](https://travis-ci.org/oil-lang/oil-rs.svg?branch=master)](https://travis-ci.org/oil-lang/oil-rs)

Oil is a graphical user interface library for Rust with video games in mind.
It is designed around three languages to describe your user interface:

 * A markup language
 * A style language
 * A dependency description language

Now you've got it right? It definitely looks pretty similar to HTML and CSS.
Of course there's a non goal of redoing a web browser here. That's not the point.

This library's goals are completely different from a web browser engine such as [servo](https://github.com/servo/servo).
The key idea behind familiarity is the ease of learning while bringing *(trying)* the good part
from web development for game development with Rust.

Okay, now a few more things to keep in mind before getting started:

* The library is young and still in its early development stage. Don't expect speed yet.
* A video game in development is currently using Uil, leading the design decisions for Uil.
  It essentially means some feature might be set as lower/higher priority because of the main project.
* Contributions are welcomed !

## [Getting-started](http://oil-lang.github.io/#getting-started)

```toml
[dependencies]
oil = "*"
```

For a concrete example, you should have a look at the examples in the `examples/` folder.

## Roadmap

This library does not allow to do many things right now. In the future, you'll have:

  * fonts support
  * User events such as mouse/key
  * Data-bindings
  * Animations
