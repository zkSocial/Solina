use solina::BatchSolution;

!// CIRCUIT_DESIGN:
!// 
!// 1. We need a vector commitment (Merkle tree) to the original intent batch.
!//    This should be provided by the Solina service already. 
!// 2. We need to check that the whole set of intents in the solution belongs
!//    belongs to the original batch of intents. Notice that we don't need
!//    to further check that every intent belongs to the batch. Later, we will
!//    check that the total batch solution liquidity matches. This means that
!//    intents which are not provided in the solution, will not contribute to the
!//    total liquidity. Better solutions, will then be picked up. So there is no
!//    incentive for the solver to ommit intents on purpose.
!// 3. We need to check that, for each match, the tokens in each intent are valid,
!//    in reverse order (quote_token, base_token) <--> (base_token, quote_token).
!// 4. Check that the constraints are satisfied.
!// 5. Check that the traded quote token does not exceed the intent one.
!// 6. Notice that each match can originate a single proof. We can then aggregate all these
!//    proofs in a zkTree, and generate a root proof, to be single checked later on.
!//    This will allow for parallelization, and a short small proof.
!// 7. A single intent might be matched more than once, I presume (TODO: check this assertion).
!//    We can route a single intent, intended for a big trade, with multiple shorter intents,
!//    as an example.
!//    That means, we need to have a global swapped amount for each intent.
!//
!//
!// Further remarks:
!//   
!// 1. The circuit generation should be delegated to, possibly, a third party.
!// 2. That means, that we should have a way to query the right circuits from a `Hub`.
!// 3. Each match will have its own small circuit.
!// 4. The total generation of of the full solution circuit, should be possible to achieve via 
!//    zkTree proof aggregation.