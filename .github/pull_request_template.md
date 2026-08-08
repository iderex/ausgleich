<!--
Replace every line below that starts with a prompt in angle brackets. The
hygiene check refuses a body left as this template, and it compares with the
whitespace removed, so deleting a blank line is not filling it in.

Keep the headings. They are what a reader skims for, and the last one is the
one this repository is least willing to lose.
-->

## What changed

<!-- One paragraph. What the change does, in the terms the issue used. -->

Closes #<!-- issue number -->.

## What failure it prevents

<!--
Not what the change enables. What goes wrong without it, concretely enough
that somebody could recognise it happening. If the answer is "nothing yet,
this is groundwork", say that instead of inventing a failure.
-->

## The means

<!--
The language, the format, the tool or the runtime this is made of, and why it
fits. One sentence is enough where the answer is the one the tree already
carries. A new dependency, a new runtime or a new file format needs the
reason written out.
-->

## Evidence

<!--
Every asserted fact carries the command that produced it, run at the commit
being pushed. Paste the command and its output, not a summary of it. A number
without its command is a claim, and writing it as a claim is allowed; passing
it off as a measurement is not.
-->

    $ <!-- command -->
    <!-- output -->

## The guards, proved by deleting them

<!--
No guard ships without proof that it bites, for the reason it names. For each
one added or changed: remove it, run the suite, and paste which tests went
red. A guard whose deletion leaves the suite green is not a guard yet, and
saying so here is worth more than a green tick.

If this change adds no guard, write that and why.
-->

## What is not covered

<!--
What this leaves undone, what it did not measure, and what a green run here is
not evidence of. A negative disclosure never becomes a positive assurance, so
if something was skipped, the admission stays and gets sharper rather than
disappearing in the next edit.
-->

## Who else read it

<!--
Say plainly whether anybody else has read this change. "Nobody else has read
this" is an acceptable answer and the honest one where it is true.
-->
