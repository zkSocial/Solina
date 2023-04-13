# zkLambda Accumulator

To convince an external party of a computed result, for many use-cases it suffices to
 - (recursively/aggregated) proof the steps along the path in the DAG that obtained the results,
 - ensure that only valid operations were performed along the path.
 
This suffices for computations where we want to demonstrate that the outcome is valid (according to above), but not necessarily unique.
Think for example about demonstrating the authenticity of a document, eg. the original raw image only has valid transformations
(cropping, resizing, etc.) applied to it. However, many different sets of transformations are all valid paths
and we don't need to enforce a unique outcome.

Often though, we want to convince a third party additionally that all proofs are derived from a single instance of our internal state.
For this reason we introduce the accumulator, which tracks state along a path of the computation DAG.
Eg. for accounting purposes, it is crucial that the accounting books can't be forked, with different histories in different forks.

## Challenge

The challenge is for the accumulator to not be forkable, while it remains hidden in the internal state. 

A solution we don't want, would be to anchor every update of the accumulator onto an L2 system.
This would restrict the throughput to an L2 system, which is still a global mutex across all updates.
Also if we would do this, we come very close to a ZEXE-like system, where every computational step consumes a "computational UTXO".
Note that we are not trying to have an open set of patricipants interact on a shared state; rather we want any participant
to be able to convince any other party of its own internal state, so we want to maintain highly paralellizable computation.

Instead we want to have a system where an entity can have accumulators that track parts of its internal state;
and ensure that different proofs (to different third parties, or over time) are referring to a single history of such an accumulator.

Inevitably we do need an external representation of each accumulator, and a sensible place to have it is on a a L2 system,
but it can equally be on nodes.
However, we want that the accumulator can be updated internally without needing to reflect each update externally (on-chain).

