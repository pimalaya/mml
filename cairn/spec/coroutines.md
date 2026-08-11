---
cairn: spec
capability: coroutines
status: current
---

# Coroutines

Compilation and interpretation are I/O-free resumable state machines. They never call a PGP backend, a network, or a clock: they suspend, say what they need, and continue once the caller answers. The backend is therefore the caller's business, which is what lets gpg live behind an off-by-default feature while the pipelines stay pure.

### Requirement: One yield type
Both coroutines SHALL report progress through `coroutine::MmlYield<T>`, generic over what they produce: MIME bytes for the compiler, MML text for the interpreter. It SHALL carry `Ok(T)`, `Err(MmlError)`, and the four inlined PGP steps `WantsPgpEncrypt`, `WantsPgpDecrypt`, `WantsPgpSign` and `WantsPgpVerify`. There is no wrapper enum between the caller and the request.

### Requirement: No backend abstraction
The crate SHALL NOT define a trait, a callback or any other seam a backend plugs into. The caller matches on the yield and answers with `pgp::MmlPgpAnswer`. A trait would smuggle back the assumption that a backend answers synchronously in one call, which is the shape the pre-2.0 async design had.

### Requirement: Coroutines take their input and their options
`MmlCompile::new(mml, MmlCompileOptions)` and `MmlInterpret::new(mime, MmlInterpretOptions)` SHALL take the subject first and its options second. The options types are plain structs of public fields, built with a struct literal and `..Default::default()`; there are no builders and no `with_*` setters.

### Requirement: Resume takes the previous answer
`resume` SHALL take `None` on the first call and the answer to the previous yield on every later one. It returns the yield directly rather than a `Result`, since failure is the `Err` variant. Internally each coroutine keeps a fallible step function so `?` stays usable, folding its error into `MmlYield::Err`.

### Requirement: One implementation per pipeline
`MmlCompileOptions::compile` and `MmlInterpretOptions::interpret` SHALL drive their own coroutine, answering `MmlPgpAnswer::Unavailable`, rather than reimplementing the pipeline. A second implementation is how a crypto directive once compiled silently in the clear.

### Requirement: Public enums stay open
`MmlYield`, `MmlPgpAnswer` and `MmlPgpVerdict` SHALL be `#[non_exhaustive]`, so a later variant is not a breaking change and downstream matches carry a catch-all from the start.
