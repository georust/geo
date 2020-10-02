Usage instructions for authors to contribute material here.

## Local Installation

This site is built by Github Pages using [jekyll]. See
[local testing] for instructions on testing it locally.
Typically, this involves installing `ruby`, then the right
version of `jekyll` (3.9.0 at the time of writing), and
executing:

``` shell
bundle exec jekyll serve
```

## Organization

This site is meant to be used only for "extra"
documentation, that cannot be published via vanilla markdown
files in the repo, or via the rust docs. These may be RFCs,
or documentation containing considerable diagrams, tables or
math in it.  Outline of the organization here:

```
/
├── rfcs      RFCs
├── docs      Extra docs, and other ref. material
├── support   Additional one-off / temporary material
```

For RFCs, and docs, the source used to generate is best also
checked into the main repo. The advantage here is mainly
custom CSS, mathjax, etc.
