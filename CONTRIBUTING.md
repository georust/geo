# Contributing to `geo`
[contributing-to-geo]: #contributing-to-geo

Thank you for your interest in contributing to `geo`! There
are many ways to contribute, and we appreciate all of them.
This document is a bit long, so here's links to the major
sections:

* [Feature Requests](#feature-requests)
* [Bug Reports](#bug-reports)
* [Pull Requests](#pull-requests)
* [Writing Tests and Documentation](#writing-documentation)

As a reminder, all contributors are expected to follow our [Code of Conduct][coc].

[coc]: https://github.com/georust/geo/blob/master/CODE_OF_CONDUCT.md

## Feature Requests
[feature-requests]: #feature-requests

`geo` aims to provide a broad suite of functionality. If
you're in any doubt as to whether a feature can be included,
simply open an issue and ask. It's a good idea to check open
issues and pull requests first, in order to check whether a
requested feature is already in progress.

## Bug Reports
[bug-reports]: #bug-reports

While bugs are unfortunate, they're a reality in software.
We can't fix what we don't know about, so please report
liberally. If you're not sure whether something is a bug or
not, feel free to file a bug anyway.

If you have the chance, before reporting a bug, please [search existing
issues](https://github.com/georust/geo/search?q=&type=Issues&utf8=✓),
as it's possible that someone else has already reported your error. This doesn't
always work, and sometimes it's hard to know what to search for, so consider this
extra credit. We won't mind if you accidentally file a duplicate report.

Similarly, to help others who encountered the bug find your
issue, consider filing an issue with with a descriptive
title, which contains information that might be unique to
it. This can be the language or compiler feature used, the
conditions that trigger the bug, or part of the error
message if there is any.

Opening an issue is as easy as following [this
link](https://github.com/georust/geo/issues/new) and filling out the fields.
Here's a template that you can use to file a bug, though it's not necessary to
use it exactly:

    <short summary of the bug>

    I tried this code:

    <code sample that causes the bug>

    I expected to see this happen: <explanation>

    Instead, this happened: <explanation>

    ## Meta

    `rustc --version --verbose`:

    Backtrace:

All three components are important: what you did, what you expected, what
happened instead. Please include the output of `rustc --version --verbose`,
which includes important information about what platform you're on, what
version of Rust you're using, etc.

Sometimes, a backtrace is helpful, and so including that is nice. To get
a backtrace, set the `RUST_BACKTRACE` environment variable to a value
other than `0`. The easiest way
to do this is to invoke `rustc` like this:

```bash
$ RUST_BACKTRACE=1 rustc ...
```

## Pull Requests
[pull-requests]: #pull-requests

Pull requests are the primary mechanism we use to change
Geo. GitHub itself has some [great
documentation][about-pull-requests] on using the Pull
Request feature. We use the "fork and pull" model [described
here][development-models], where contributors push changes
to their personal fork and create pull requests to bring
those changes into the source repository.

[about-pull-requests]: https://help.github.com/articles/about-pull-requests/
[development-models]: https://help.github.com/articles/about-collaborative-development-models/

Please make pull requests against the `master` branch.

All pull requests are reviewed by another person.

After someone has reviewed your pull request, they will
leave an annotation on the pull request with an `r+`. It
will look something like this:

    bors: r+

This tells @bors, our lovable integration bot, that your
pull request has been approved. The PR then enters the merge
queue, where @bors will run all the tests on every platform
we support. If it all works out, @bors will merge your code
into `master` and close the pull request.

## Writing Tests and Documentation
[writing-documentation]: #writing-documentation

Documentation improvements are very welcome. Standard API
documentation is generated from the source code itself. If
you're adding a new feature, you **must** document its use,
and write tests, preferably covering 100% of the added
functionality. [Several
geometries](geo/src/algorithm/test_fixtures) are provided as
test fixtures. If you need help with the format or content
of docs, or help writing some tests, don't hesitate to ask.

## Publishing to crates.io

The repo contains three crates: `geo-types`, `geo`,
`geo-postgis`, and the latter two depends on the former. A
typical release involves publishing one of these, but major
releases may involve all three.  If publishing more than one,

### Publishing a sub-crate

1. Ensure `CHANGES.md` lists the PRs, or features added in
   the version to be released.q

1. Update `Cargo.toml` to reflect the new version. Note
   that, a breaking change should be released with a change
   in the major version number.

1. If there is a dependency on the other sub-crates (eg.
   `geo` depends on `geo-types`), ensure the major version
   matches the latest release, and the minor the minimum
   version needed to compile.

1. Commit the changes and run `cargo publish` in the
   appropriate directory.

After releasing the crates, push the changes, and tag as
`{crate-name}-{version}`.
