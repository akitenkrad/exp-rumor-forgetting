**English** | [日本語](model.ja.md)

# Model and implementation

Each person is ignorant (I), a spreader (S), or a stifler (R). Every spreader starts conversations at rate 1. A uniformly selected contact becomes S with probability `lambda` when ignorant; if the contact is S or R, the speaker alone becomes R with probability `sigma`. Forgetting occurs at rate `delta` per eligible person and returns that person to I. The default mode applies forgetting to S and R; `stifler_only` applies it only to R.

The Rust implementation uses continuous-time Gillespie events inside unit-length socsim engine steps. An event crossing a step boundary is retained for the next step. ER networks are generated through `socsim-net` and copied once into CSR arrays; the event loop never calls `Network::neighbors()`.

Network and dynamics random streams are derived separately. An ER network is shared by every parameter condition with the same trial seed. Rayon parallelism does not change trial output ordering or random streams.

The simulation records trial summaries, condition aggregates, and selected time series in a runvault run. Quasi-stationary values are the mean of integer-time samples in `[T/2, T]`, among trials that still contain a spreader at `T`.
