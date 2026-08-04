# Rule Packages

Rule packages remain the routing contract between a competition context and a model entry.

The public built-in packages contain only:

- model identifier and competition type;
- supported snapshot names;
- external-provider metadata;
- request and response contract identifiers.

They do not contain model coefficients, calibration values, production profiles, fixed fixtures, or algorithm-specific output assumptions.

A private provider may resolve its own parameters by model identifier and rule-package identity. Public callers must treat the `parameters` object as an opaque provider boundary.
