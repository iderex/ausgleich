# Security Policy

## What this repository is, before anything else

`ausgleich` is an open rebuild of the CODATA least-squares adjustment of the
fundamental constants. It is a Rust workspace of five crates that reads local
files, does linear algebra on them, and writes local files. There is no server,
no daemon, no socket, no account, no credential and no deployment.

The tree says one thing the description does not, and a reporter should know it
before spending an evening here. The adjustment does not run yet.
`crates/ausgleich-cli/src/main.rs` prints one line to standard error and exits
2, because the command surface has not been decided.
`crates/ausgleich-equations/src/lib.rs` and `crates/ausgleich-report/src/lib.rs`
are documentation with no code under them. No file in `crates/*/src` calls
`std::fs`, opens a path, reads an environment variable, or contains the word
`unsafe`. There are no releases, and `publish = false` is set across the
workspace, so nothing here sits on a registry for anybody to depend on.

What exists is four libraries whose readers take a document as a string and
either return a record or refuse it by name. That is the security surface of
this repository today.

## Where to report

Use GitHub's private vulnerability reporting:

<https://github.com/iderex/ausgleich/security/advisories/new>

That door opens today. Measured rather than assumed:

```console
$ gh api repos/iderex/ausgleich/private-vulnerability-reporting
{"enabled":true}
```

Useful in a report: the commit you tested and the shortest input that shows the
problem. A fixture and the output it produced is worth more than a description
of both.

If what you found is better argued in the open, the tracker has three forms:
`defect.yml` for a crash or a refusal that should not have refused,
`data-correction.yml` for a digitised value, and `equation-correction.yml` for
an observational equation.

## What I do not promise

I do not promise a time to first answer. A deadline this project cannot keep is
worse than no deadline at all: a reporter told to expect a reply by some point,
who does not get one, spends the wait wondering whether the report arrived
rather than whether it is being worked on. Saying up front that the answer comes
when it comes is the position I can actually hold.

## What would actually be a vulnerability here

**A crafted record file.** The readers in `crates/ausgleich-data/src/`
(`datum.rs`, `coefficient.rs`, `constant.rs`, `provenance.rs`, `units.rs`) parse
TOML this project did not write. An input set is meant to be contributed to, one
datum per file with a provenance block naming the publication it was read out
of, so a record file is genuinely third-party input. A panic, an unbounded
allocation, or anything worse reached from one is the report I want.

**A crafted seam document.** `crates/ausgleich-solve/src/seam.rs` is a
deliberate interchange format: the problem file going in, the result file coming
back, so that a sampler or a reimplementation in another language can join
there. Those bytes can arrive from anywhere, and they feed the factorisation in
`whiten.rs` and the solve in `solve.rs`.

**Silent numerical corruption.** This is the class I care most about, and the
one a generic policy would never name. What comes out of this program is a value
of a physical constant that somebody will cite. An input that makes it print a
wrong number without refusing is worse than one that makes it crash, because the
crash is visible and the wrong number is not. The readers already refuse `nan`
and `inf`, a negative uncertainty, an uncertainty form that is neither absolute
nor relative, a covariance that disagrees with itself across the diagonal, and
one identifier carried by two files. A route past any of those, or any other way
to get a wrong number out of a run that looks ordinary, belongs in the advisory
channel even though none of it is memory safety.

**A network call.** The README promises that nothing leaves your machine, and
the check named `Enforce greppable invariants` holds it with `git grep` patterns
over `crates/*/src` and the member manifests. Those patterns read names. A
request through a dependency two levels down, or through a name the pattern does
not know, would walk straight past them. That is a finding against a promise
this repository makes in writing, and it goes to the advisory channel rather
than to the tracker.

**The build.** `Cargo.lock` pins seventeen packages, twelve from outside, all of
them underneath `toml`. Sixteen workflows run on changes. A way to get code into
a build through the lockfile or through a workflow reaches every number this
project would ever publish.

## What is not a vulnerability here

A constant that disagrees with the published value. That is a data or equation
defect, it has its own form on the tracker, and in a sense it is the point of
the project.

A refusal. The readers refuse a great deal on purpose. A published table that
cannot be assembled into a covariance that factors, a design matrix that leaves
a direction the data does not determine, a file not named for the identifier it
carries: those are findings about the input, reported rather than repaired. A
refusal that fired wrongly is a defect for the tracker. That a refusal fired at
all is the product working.

The binary doing nothing. `ausgleich` exits 2 and says there is no command
surface yet, so that a run which did nothing cannot read as a run that worked.

Resource exhaustion from a file you chose. Nothing here is a service: you start
the program yourself, on files you named, on your own machine. A large or deeply
nested document making it slow is the same category as feeding a large file to
any local tool. Memory unsafety would change that, and so would a small input
with an output out of all proportion to it.

A vulnerability in `toml` or below it. Report those upstream where they get
fixed for everybody, and tell me too if this workspace uses the affected path so
I can move the pin. Dependabot, `cargo deny` and dependency review run here.

Missing hardening with nothing to hold on to. There is no authentication, no
session, no rate limit, no sandbox and no secret store, because nothing here
would need one. A report listing their absence describes a template rather than
this program.

## What is supported

`main`, and only `main`. There is no release, no tag, no published crate and no
support window I could honestly draw. Name the commit you tested: the tip moves,
and the code you read may be a week old by the time I reach your report.
