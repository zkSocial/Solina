# zkLambda

This is a draft proposal for provable anonymous functions.

## Motivation

The goal of zkLambda is to provide a platform for provable anonymous functions. The platform should be able to provide the following features:
- developer can write a function in a high-level language (in particular Rust) and deploy it to the platform
- developer can invoke the function with a set of inputs
- the function and the inputs should be hashed, such that reinvokation can be a simple look-up
- the function should be compiled to RISC-5 instruction set, such that it can be run in RISC-0 zkVM
- (later counterfactual proving) the execution trace of the function call should be committed to, but the proof itself should not be computed by default
- outputs from previous functions should be able to be used as inputs to the next function
- (later) proofs of a sequence of function calls should be able to be aggregated or recursevely proven (TBD what makes more sense)

## Design

(Inspiration basis from IPVM specification research)

The design of zkLambda is can be structured as follows (below is pseudo-code-semi-json):

1. developers can invoke a function call by submitting a `job` to the zkLambda platform
```ipld
"job": {
    "invocation" {
        "function": QmHashOfFunction,
        "inputs": [
            Input1,
            Input2,
            Input3,
        ]
        },
    "config": someConfigToOverrideDefaults,
}
```

Config can set max cost, cron schedule, whether enqueue to prove the function (for PoC default true), etc.

2. the result is encapsulated in a session when returned
```ipld
"session": {
    "job": QmHashOfJob,
    "result": {
        "invocation": QmHashOfInvocation,
        "output(pure)": QmHashOfPureOutput,
        "effects": [
            EffectFutureForProof,
            Effect1,
            Effect2,
        ]
    },
    "trace": QmHashOfProvableTrace,
    "error": Option(QmHashOfError),
}
```

The effects can be instructions for continued computation, internal error handling, etc. (pure effects), or affect the outside world (impure effects that can't be rolled back) external actions (eg. sending an email, or send on-chain tx with proof).

Pure effects can be job descriptions for future invocations, eg. `job2` to be enqueued. In particular a job to output as an effect is the (lazy) proof calculations of the (current/previous) invocations.

3. chaining together state

To track an event stream, the developer can submit a job with aditional inputs that are pure outputs of a previous job, recursively so. This can be used to track a stateful event stream (over a linear stream or a DAG), eg. a user's account balance.

```ipld
"job": {
    "invocation" {
        "function": QmHashOfFunction,
        "inputs": [
            Input1,
            Input2,
            Input3,
        ],
        "chainedInputs": [
            QmHashOfPureOutput1,
            QmHashOfPureOutput2,
            QmHashOfPureOutput3,
        ]
        },
    "config": someConfigToOverrideDefaults,
}
```