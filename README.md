# Flou

Flou is a [domain-specific language (DSL)](https://en.wikipedia.org/wiki/Domain-specific_language) for describing flowcharts. It is also a CLI of the same name that renders the previously mentioned flowchart description into an SVG file.

Flou's goal is to offer a textual representation of flowcharts. Here's one example of the kind of flowcharts Flou can create:

![Example](docs/src/syntax/define_block/example1.svg)

Here's the corresponding Flou definition:

```js
grid {
    block("Think about going outside", connect: s:n@s);
    condition("Is it raining?"),                           block#stay("Stay inside");
    condition("Is it cold?");
    condition("Is it night?");
    block("Go outside");
}

define {
    block(class: "pink");
    condition(shape: diamond, class: "yellow", connect: {
        s:n@s("No");
        e:w#stay("Yes");
    });
}
```

## Documentation

Flou's documentation and user guide can be found [here](https://asha20.github.io/flou).

## Installation

You can grab a prebuilt binary for your operating system [here](https://github.com/Asha20/flou/releases). Alternatively, if you have Cargo installed, you can run:

    $ cargo install flou_cli

Which will install the `flou` binary for you to use.

## Reasons to use Flou?

- If you need to generate a flowchart automatically, you can write a program that generates Flou DSL and then use the CLI tool to compile the DSL into an image.
- Textual representation avoids easy-to-miss slight design inconsistencies that might occur when creating a flowchart with a visual design software.
- Flou makes modifying shared flowchart parts straightforward and painless.
- A textual flowchart representation is more suited for version control.

## Reasons NOT to use Flou?

- It's still in beta. This means some features might be unpolished.
- Connections that happen to have overlapping segments can bring visual ambiguity since Flou CLI won't render them side by side and will overlap them instead. However, this issue can be offset by the user since they can pick and choose the connection sides.