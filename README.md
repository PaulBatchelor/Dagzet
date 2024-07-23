# Dagzet

DAGZet (Directed-Acyclic-Graph Zettelkasten) is a simple
text-based data format for producing knowledge graphs.
A graph described in Dagzet gets compiled down into
SQLite code, which can then be parsed by sqlite to
be turned into a queryable database.

## Installation
Compile and install locally with:

```
cargo install --path .
```

## Basic Usage
Here is an example of a simple graph

```
zz declare the namespace
ns hello

zz create a new node "world"
zz the full namespace is "hello/world"

nn world
ln this is a line.
ln many lines can be appended.

zz create a new node "another"
nn another
ln Another node has been created

zz make "another" point to "world" (another -> world)"
zz this makes "another" a child of "world"
co another world
```

This can then be converted to a SQLite database:

```
$ dagzet hello.dz | sqlite a.db
$ echo "SELECT name FROM dz_nodes" | sqlite3 a.db"
hello/another
hello/world
```
