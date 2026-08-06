# ausgleich

Every value of every fundamental constant comes from a least-squares adjustment a small group performs every four years, described in publications with its inputs and correlation coefficients in PDF tables and no code anywhere. The 2010 adjustment had 160 inputs, 83 adjusted constants and 77 degrees of freedom, which runs in seconds, yet nobody can recompute it. It contains judgement calls: the 2018 adjustment needed an expansion factor of 3.9 on the sixteen measurements of the gravitational constant, 1.6 on 62 Rydberg and charge-radius inputs and 1.7 on two proton mass inputs, an expansion factor being the term for enlarging error bars by hand until contradicting measurements agree. With the code in the open, three questions become askable that cannot be asked today: what comes out without expansion factors, with robust estimators, and with a hierarchical Bayesian model of the between-laboratory scatter.

Planning happens on the issue tracker first. Every decision that shapes
the architecture is written down there with its reasons before the code
that depends on it exists.

See [NOTICE.md](NOTICE.md) for the intended-use notice.
