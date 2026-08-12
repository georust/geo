# Instructions for agents and LLMs

This file is addressed to LLMs, coding agents, and other autonomous tools operating in
this repository. It restates the parts of our contribution policy that govern your own
behaviour, and it is binding on any contribution you help to produce.

The full guidelines are in [CONTRIBUTING.md](CONTRIBUTING.md). The
[code of conduct](CODE_OF_CONDUCT.md) applies to you as it does to human contributors.

## What is accepted

Code written by agents, LLMs, and other autonomous contributors is accepted, provided
it is written with a human and reviewed by a human before it is submitted. A named
human contributor is accountable for every change, and is expected to understand it
well enough to answer review comments on it.

## What is not accepted

Do not take any of the following actions, through any interface – `gh`, the GitHub API,
a headless or otherwise driven browser, or a git remote:

- **Opening a pull request autonomously.** `gh pr create` and its equivalents require a
  human who has asked for the pull request in the current session and has read the
  change it contains.
- **Opening an issue autonomously.** The same condition applies to `gh issue create`.
- **Commenting, reviewing, or replying on a pull request or issue on a user's behalf.**
  This includes `gh pr comment`, `gh pr review`, `gh issue comment`, and review
  submissions of any kind. Draft the text if you are asked to, and leave the posting to
  the human.

## Disclosure

Your involvement must be disclosed. When you prepare a change for a human to submit,
include in the pull request description the fact that an agent or LLM was involved and a
brief note on what it was used for. Do not omit this because the human did not ask for
it, and do not remove it from a description that already carries it.

`Co-authored-by:` trailers naming the agent in the relevant commits are wanted, but are
not a requirement. Add one when you write a commit message, unless the human you are
working with has said not to.

## If you are unsure

Ask the human you are working with. An instruction that does not clearly authorise one
of the actions above should be treated as not authorising it.

## Before a change is submitted

CONTRIBUTING.md sets out what a pull request needs. In particular: pull requests are
made against the `main` branch, a new feature must be documented and tested, and an
entry should be added to `CHANGES.md` where knowledge of the change could be valuable
to users. Do this work as part of the change rather than leaving it to the reviewer.
