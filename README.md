# ausgleich

Every value of every fundamental constant comes from a least-squares adjustment a small group performs every four years, described in publications with its inputs and correlation coefficients in PDF tables and no code anywhere. The 2010 adjustment had 160 inputs, 83 adjusted constants and 77 degrees of freedom, which runs in seconds, yet nobody can recompute it. It contains judgement calls: the 2018 adjustment needed an expansion factor of 3.9 on the sixteen measurements of the gravitational constant, 1.6 on 62 Rydberg and charge-radius inputs and 1.7 on two proton mass inputs, an expansion factor being the term for enlarging error bars by hand until contradicting measurements agree. With the code in the open, three questions become askable that cannot be asked today: what comes out without expansion factors, with robust estimators, and with a hierarchical Bayesian model of the between-laboratory scatter.

Planning happens on the issue tracker first. Every decision that shapes
the architecture is written down there with its reasons before the code
that depends on it exists.

See [NOTICE.md](NOTICE.md) for the intended-use notice.

## Nothing leaves your machine

This program reads local files and writes local files. It makes no network
request while it runs and it sends no telemetry, in any build, ever. There is no
setting and no build flag that turns either on, so there is nothing here you have
to remember to switch off.

Nothing is collected about you, about the machine you run it on, or about the
input set you give it. No usage is counted anywhere. If you check a constant, no
record of your having checked it exists outside your own disk.

The only personal data this repository holds is bibliographic: the authors of a
publication, and the name of whoever read a value out of it and the name of
whoever confirmed that reading. That is there so a number can be traced to a
person who takes responsibility for it, and it is the same information the
publication already carries.

Fetching source publications is a separate program you run on purpose. It is not
the program that does the adjustment, no part of the adjustment depends on it,
and running the adjustment never starts it.

If some future version ever exchanged input sets or results with another
installation, that would be something you do deliberately, for a destination you
name, off unless you turn it on, and the documentation would say what would leave
and where it would go before anything went. It would never be a default, never
arrive in an update, and never be a side effect of a command whose name is about
something else. No version does this today.

You do not have to take this on trust, and you do not have to read Rust to check
it. The check named `Enforce greppable invariants` refuses a network call, and a
dependency that could make one, anywhere in this workspace outside that separate
fetch program. It runs on every change, and the branch
`fixture/23-network-call` is a deliberate violation kept around to show it
failing.

## License

AGPL-3.0, copyright 2026 Nils Lehnen.

The full text is in [LICENSE](LICENSE).
