# Naive Bundler

This is a simple bundler for bundling Javascript files. It doesn't understand you might have files in node_modules, you have to write the full path to a file you want to include (with .js as well, yes). It uses swc for simplistic transpilation, but I bet it could do more.

## Usage

```bash
cargo run <input_path> <output_path>
```

## Things I wish I knew better

1. Why do I have to use GLOBALS for swc, and what these GLOBALS are anyway
2. How do I reduce the number of .clone() in my code? Is it Rc?

## Special thanks

1. [Minipack](https://github.com/ronami/minipack/tree/master)
1. [This blogpost](https://kakoc.blog/blog/myox-js-bundler/)

## What didn't work out for me

Cargo.toml includes few dependencies that are left as comments. I tried doing it with rslint's parser, but it's ast was too low-level, and I don't think it is capable of transpiling. I also tried OXC, which I think is a great project, but it was hard for me to fight the borrow checker with it's allocator, so I was unable to figure out how to transpiled with it. SWC seemed to fit in.

One issue I found is that when I do this:

```js
console.log(importedFunction());
```

it produces something like this:

```js
console.log(0, importedFunction());
```

while the swc playground produces this:

```js
console.log((0, importedFunction()));
```

## What's the difference to Minipack?

First and foremost, it uses Rust btw. Also I didn't fully understand it's graph walking algorythm, so I just implemented my own width-first search.