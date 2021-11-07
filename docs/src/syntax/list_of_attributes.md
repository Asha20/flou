## List of attributes

### Node attributes

These are the attributes that can be defined on a node:

- `text` — The text to render inside the node.
- `class` — One or more CSS classes that will get appended to this node's SVG representation; read more [here](../styling_flowchart.md).
- `shape` — Determines the node's shape. Can be one of the following:
  - `rect` — Rectangle (default).
  - `square` — Square.
  - `ellipse` — Ellipse.
  - `circle` — Circle.
  - `diamond` — Diamond.
  - `angled_square` — Square at a 45° angle.
- `connect` — Defines one or more connections this node has to other nodes. Consists of two parts:
  - Connection sides. Has the format `x:y` meaning "connect the **x** side of the source node to the **y** side of the destination node. `x` and `y` can be one of the following:
    - `n` — North.
    - `s` — South.
    - `w` — West.
    - `e` — East.
  - Destination. Can be one of the following:
    - `#dest` — Connect to the node with the label `dest`.
    - `@n` — Connect to the node directly **north** of source node. (similar for other cardinal directions).
    - `@` — Connect source node to itself.

### Connection attributes

These are the attributes that can be defined on a connection:

- `text` — The text that appears next to the connection's beginning.
- `class` — One or more CSS classes that will get appended to this connection's SVG representation; read more [here](../styling_flowchart.md).
- `arrowheads` — Determines which arrowheads the connection will have. Can be one of the following:
  - `none` — No arrowheads.
  - `start` — Arrowhead on the source node only.
  - `end` — Arrowhead on the destination node only (default).
  - `both` — Arrowheads on both the source and destination nodes.